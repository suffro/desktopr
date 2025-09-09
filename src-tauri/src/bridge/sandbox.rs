// src/bridge/sandbox.rs
// All comments are in English as requested.

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant, SystemTime},
};

use tauri::AppHandle;
use tauri::Manager; // for AppHandle.path()

use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Context, Result};

// Wasmtime/WASI (Preview1 in wasmtime-wasi 19.x)
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtxBuilder, DirPerms, FilePerms, I32Exit};
use wasmtime_wasi::preview1::add_to_linker_sync;
use wasmtime_wasi::WasiP1Ctx; // exported when feature "preview1" is enabled
// Stdio virtual pipes (implement the required traits for WasiCtxBuilder)
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use bytes::Bytes;
// Use wasi_common::sync Dir adapter to wrap cap-std directories
use wasi_common::sync::Dir as WasiSyncDir;

// Helpers
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
// Reuse the file-open helper to pick a .wasm and read bytes
use crate::bridge::files::bd_file_open_with_bytes;

/// Store data: holds WASI context + optional limiter so we can hand
/// a &'static lifetime to wasmtime via a reference into store data.
struct StoreState {
    wasi: WasiP1Ctx,
    limiter: Option<StoreLimiter>,
}

/// ======================================================
/// Global state: concurrency limit, running counter, TTL
/// ======================================================

static CONCURRENCY_LIMIT: AtomicUsize = AtomicUsize::new(2);
static RUNNING_JOBS: AtomicUsize = AtomicUsize::new(0);
static TTL_MINUTES: AtomicU64 = AtomicU64::new(60);
static SERVICES_STARTED: Lazy<std::sync::Mutex<bool>> =
    Lazy::new(|| std::sync::Mutex::new(false));

/// ======================================================
/// Public API types
/// ======================================================
#[derive(Debug, Deserialize, Clone)]
pub struct SandboxCaps {
    pub timeout_ms: Option<u64>,    // hard timeout via epoch interruption
    pub memory_mb: Option<usize>,   // linear memory upper bound
    pub cpu_fuel: Option<u64>,      // (disabled below for 19.x portability)
    pub stdout_max_kb: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SandboxRunInput {
    pub module_path: String,                 // e.g. "_external_modules/my.wasm"
    pub args: Option<Vec<String>>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub stdin: Option<String>,
    pub caps: Option<SandboxCaps>,
    pub working_dir: Option<String>,         // set to "_sandbox/<jobId>"
}

/// Unified call input: send JSON to stdin, expect JSON on stdout.
#[derive(Debug, Deserialize, Clone)]
pub struct SandboxCallInputJson {
    pub module_path: String,                  // e.g. "_external_modules/my.wasm" or "mylib.wasm"
    pub payload: serde_json::Value,           // arbitrary JSON to send to stdin
    pub caps: Option<SandboxCaps>,            // optional limits (timeout/mem/stdout cap)
    pub env: Option<std::collections::HashMap<String, String>>, // optional env
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct RunStats {
    pub wall_ms: u128,
    pub fuel_used: Option<u64>,   // None for now (see note)
    pub peak_mb: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SandboxRunResult {
    pub ok: bool,
    pub job_id: String,
    pub work_dir: String,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub value: Option<serde_json::Value>,    // parsed JSON from stdout (best-effort)
    pub stats: Option<RunStats>,
}

/// ======================================================
/// Paths helpers
/// ======================================================

fn root_dir(app: &AppHandle) -> Result<PathBuf> {
    // Tauri v2: Result<PathBuf, Error>
    let data = app
        .path()
        .app_data_dir()
        .map_err(|_| anyhow!("app_data_dir not available"))?;
    let root = data
        .parent()
        .ok_or_else(|| anyhow!("cannot resolve root parent"))?
        .to_path_buf();
    Ok(root)
}

fn external_modules_dir(app: &AppHandle) -> Result<PathBuf> {
    let p = root_dir(app)?.join("_external_modules");
    let _ = fs::create_dir_all(&p);
    Ok(p)
}

fn sandbox_dir(app: &AppHandle) -> Result<PathBuf> {
    let p = root_dir(app)?.join("_sandbox");
    let _ = fs::create_dir_all(&p);
    Ok(p)
}

fn job_sandbox_dir(app: &AppHandle, job_id: &str) -> Result<PathBuf> {
    let base = sandbox_dir(app)?;
    let p = base.join(job_id);
    fs::create_dir_all(&p)?;
    Ok(p)
}

fn wipe_sandbox_all(app: &AppHandle) -> Result<()> {
    let s = sandbox_dir(app)?;
    if s.exists() {
        for entry in fs::read_dir(&s)? {
            let e = entry?;
            let _ = if e.path().is_dir() {
                fs::remove_dir_all(e.path())
            } else {
                fs::remove_file(e.path())
            };
        }
    }
    Ok(())
}

fn resolve_module_path(app: &AppHandle, module_path: &str) -> Result<PathBuf> {
    let base = external_modules_dir(app)?;
    let p = Path::new(module_path);
    let final_path = if p.is_absolute() { p.to_path_buf() } else { base.join(p) };
    if !final_path.starts_with(&base) {
        return Err(anyhow!("module path escapes _external_modules"));
    }
    if !final_path.exists() {
        return Err(anyhow!("module not found: {}", final_path.display()));
    }
    Ok(final_path)
}

/// ======================================================
/// Resource limiter (memory)
/// ======================================================
struct StoreLimiter {
    max_memory_bytes: usize,
}
impl wasmtime::ResourceLimiter for StoreLimiter {
    fn memory_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>)
        -> Result<bool, anyhow::Error>
    {
        Ok(desired <= self.max_memory_bytes)
    }
    fn table_growing(&mut self, _current: u32, _desired: u32, _maximum: Option<u32>)
        -> Result<bool, anyhow::Error>
    {
        Ok(true)
    }
}

/// ======================================================
/// Helpers
/// ======================================================

fn gen_job_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
fn mtime(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

pub fn sandbox_cleanup_on_boot(app: &AppHandle) -> Result<()> {
    wipe_sandbox_all(app)?;
    Ok(())
}

pub fn start_sandbox_janitor(app: &AppHandle) -> Result<()> {
    let app_path = sandbox_dir(app)?;
    let ttl_minutes = TTL_MINUTES.load(Ordering::SeqCst);
    {
        let mut started = SERVICES_STARTED.lock().unwrap();
        if *started { return Ok(()); }
        *started = true;
    }
    thread::Builder::new()
        .name("bubbledesk-sandbox-janitor".into())
        .spawn(move || {
            loop {
                let ttl = Duration::from_secs(ttl_minutes.saturating_mul(60));
                let now = SystemTime::now();
                if let Ok(entries) = fs::read_dir(&app_path) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if p.is_dir() {
                            let running_flag = p.join(".RUNNING");
                            if running_flag.exists() { continue; }
                            if let Some(mt) = mtime(&p) {
                                if now.duration_since(mt).unwrap_or_default() > ttl {
                                    let _ = fs::remove_dir_all(&p);
                                }
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_secs(60));
            }
        })
        .ok();
    Ok(())
}

/// ======================================================
/// Core run (WASI/_start)
/// ======================================================

fn run_wasi_module(app: &AppHandle, job_id: &str, input: SandboxRunInput) -> Result<SandboxRunResult> {
    let caps = input.caps.clone().unwrap_or(SandboxCaps {
        timeout_ms: Some(15_000),
        memory_mb: Some(256),
        cpu_fuel: Some(5_000_000),   // NOTE: fuel disabled below for portability
        stdout_max_kb: Some(512),
    });
    let args = input.args.clone().unwrap_or_default();
    let env = input.env.clone().unwrap_or_default();

    let module_path = resolve_module_path(app, &input.module_path)?;
    let work_abs = {
        let w = PathBuf::from(
            input.working_dir.clone().ok_or_else(|| anyhow!("missing working_dir"))?,
        );
        let base = sandbox_dir(app)?;
        if !w.starts_with(&base) {
            return Err(anyhow!("working_dir must be under _sandbox"));
        }
        let _ = fs::create_dir_all(&w);
        w
    };

    // Stdio pipes
    let stdin_bytes = Bytes::from(input.stdin.clone().unwrap_or_default().into_bytes());
    let stdin_pipe  = MemoryInputPipe::new(stdin_bytes);
    let stdout_pipe = MemoryOutputPipe::new(1024 * 1024); // 1 MiB
    let stderr_pipe = MemoryOutputPipe::new(512 * 1024);  // 512 KiB

    // Wasmtime config
    let mut cfg = Config::new();
    cfg.consume_fuel(true);
    cfg.wasm_multi_value(true);
    cfg.epoch_interruption(true);

    let engine = Engine::new(&cfg)?;
    let module = Module::from_file(&engine, &module_path)?;

    // WASI ctx
    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder.stdin(stdin_pipe);
    wasi_builder.stdout(stdout_pipe.clone());
    wasi_builder.stderr(stderr_pipe.clone());

    let mut wasi_args = vec![module_path.file_name().unwrap_or_default().to_string_lossy().to_string()];
    wasi_args.extend(args.into_iter());
    wasi_builder.args(&wasi_args);
    for (k, v) in env.iter() { wasi_builder.env(k, v); }

    // Preopen _sandbox/<jobId> at "/"
    let cap_dir  = cap_std::fs::Dir::open_ambient_dir(&work_abs, cap_std::ambient_authority())
        .context("failed to open sandbox dir")?;
    let wasi_dir = WasiSyncDir::reopen_dir(&cap_dir)?;
    // DirPerms::MUTATE is the write-flag for dirs in 19.x
    wasi_builder.preopened_dir(
        wasi_dir,
        DirPerms::READ | DirPerms::MUTATE,
        FilePerms::READ | FilePerms::WRITE,
        "/",
    );

    let wasi_ctx = wasi_builder.build_p1();

    // Linker + instance pre
    let mut linker: Linker<StoreState> = Linker::new(&engine);
    add_to_linker_sync(&mut linker, |cx: &mut StoreState| &mut cx.wasi)?;
    let pre = linker.instantiate_pre(&module)?;

    // Store + memory limiter
    let mut store = Store::new(&engine, StoreState { wasi: wasi_ctx, limiter: None });
    // NOTE: to avoid version-gating issues, we omit `store.add_fuel(...)` here.
    if let Some(mb) = caps.memory_mb {
        let maxb = mb.saturating_mul(1024 * 1024);
        store.data_mut().limiter = Some(StoreLimiter { max_memory_bytes: maxb });
        store.limiter(|data| data.limiter.as_mut().expect("limiter set"));
    }

    // Instantiate and run
    let start = Instant::now();
    let instance = pre.instantiate(&mut store)?;

    // Timeout via epoch interruption
    if let Some(ms) = caps.timeout_ms {
        let engine_clone = engine.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(ms));
            engine_clone.increment_epoch();
        });
    }

    // Call `_start`
    let call_res: Result<i32> = (|| {
        let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
        func.call(&mut store, ())
            .map(|_| 0)
            .map_err(|e| {
                if let Some(exit) = e.downcast_ref::<I32Exit>() {
                    return anyhow!(format!("__EXIT__{}", exit.0));
                }
                anyhow!(e.to_string())
            })
    })();

    // NOTE: fuel accounting disabled for portability
    let fuel_used: Option<u64> = None;
    drop(store);

    // Drain pipes
    let out = if let Some(c) = stdout_pipe.try_into_inner() { c.to_vec() } else { Vec::new() };
    let err = if let Some(c) = stderr_pipe.try_into_inner() { c.to_vec() } else { Vec::new() };
    let stdout_str = String::from_utf8_lossy(&out).to_string();
    let stderr_str = String::from_utf8_lossy(&err).to_string();

    let wall_ms = start.elapsed().as_millis();

    let (ok, ec) = match call_res {
        Ok(code) => (true, Some(code)),
        Err(e) => {
            let s = e.to_string();
            if s.starts_with("__EXIT__") {
                let code: i32 = s.trim_start_matches("__EXIT__").parse().unwrap_or(1);
                (true, Some(code))
            } else {
                (false, None)
            }
        }
    };

    // Limit stdout
    let stdout_limited = if let Some(kb) = caps.stdout_max_kb {
        let limit = kb * 1024;
        if stdout_str.len() > limit { stdout_str[..limit].to_string() } else { stdout_str }
    } else { stdout_str };

    // Best-effort parse JSON
    let value = match serde_json::from_str::<serde_json::Value>(&stdout_limited) {
        Ok(v) => Some(v),
        Err(_) => None,
    };

    Ok(SandboxRunResult {
        ok,
        job_id: job_id.to_string(),
        work_dir: work_abs.to_string_lossy().to_string(),
        exit_code: ec,
        stdout: if value.is_none() { Some(stdout_limited) } else { None },
        stderr: if !stderr_str.is_empty() { Some(stderr_str) } else { None },
        value,
        stats: Some(RunStats { wall_ms, fuel_used, peak_mb: caps.memory_mb }),
    })
}

/// ======================================================
/// Tauri commands
/// ======================================================

/// Single high-level command: send JSON to `_start` and return parsed JSON/stdout/stderr.
#[tauri::command]
pub async fn bd_sandbox_call(app: AppHandle, input: SandboxCallInputJson) -> Result<serde_json::Value, String> {
    // Concurrency gate
    let limit = CONCURRENCY_LIMIT.load(Ordering::SeqCst);
    let prev = RUNNING_JOBS.fetch_add(1, Ordering::SeqCst);
    if prev >= limit {
        RUNNING_JOBS.fetch_sub(1, Ordering::SeqCst);
        return Err(format!("sandbox is busy: {prev} running, limit={limit}"));
    }

    // Ensure base dirs and services
    external_modules_dir(&app).map_err(|e| e.to_string())?;
    sandbox_dir(&app).map_err(|e| e.to_string())?;
    if let Err(e) = start_sandbox_janitor(&app) {
        eprintln!("[sandbox] janitor start failed: {e}");
    }

    // Create job workspace
    let job_id = gen_job_id();
    let work_abs = job_sandbox_dir(&app, &job_id).map_err(|e| e.to_string())?;
    let _ = fs::write(work_abs.join(".RUNNING"), b"");

    // Build SandboxRunInput from JSON payload
    let stdin_str = serde_json::to_string(&input.payload).map_err(|e| e.to_string())?;
    let run_input = SandboxRunInput {
        module_path: input.module_path.clone(),
        args: Some(vec![]),
        env: input.env.clone(),
        stdin: Some(stdin_str),
        caps: input.caps.clone(),
        working_dir: Some(work_abs.to_string_lossy().to_string()),
    };

    // Execute
    let res = run_wasi_module(&app, &job_id, run_input);

    // Cleanup ONLY this job workspace
    let _ = fs::remove_file(work_abs.join(".RUNNING"));
    let _ = fs::remove_dir_all(&work_abs);

    // Decrement running counter
    RUNNING_JOBS.fetch_sub(1, Ordering::SeqCst);

    // Return
    let out = res.map_err(|e| e.to_string())?;
    serde_json::to_value(out).map_err(|e| e.to_string())
}

/// Legacy command kept for backward-compat (optional).
#[tauri::command]
pub async fn bd_sandbox_run(app: AppHandle, input: serde_json::Value) -> Result<serde_json::Value, String> {
    let limit = CONCURRENCY_LIMIT.load(Ordering::SeqCst);
    let prev = RUNNING_JOBS.fetch_add(1, Ordering::SeqCst);
    if prev >= limit {
        RUNNING_JOBS.fetch_sub(1, Ordering::SeqCst);
        return Err(format!("sandbox is busy: {prev} running, limit={limit}"));
    }

    external_modules_dir(&app).map_err(|e| e.to_string())?;
    sandbox_dir(&app).map_err(|e| e.to_string())?;
    if let Err(e) = start_sandbox_janitor(&app) {
        eprintln!("[sandbox] janitor start failed: {e}");
    }

    let mut run_input: SandboxRunInput = serde_json::from_value(input).map_err(|e| e.to_string())?;
    let job_id = gen_job_id();
    let work_abs = job_sandbox_dir(&app, &job_id).map_err(|e| e.to_string())?;
    run_input.working_dir = Some(work_abs.to_string_lossy().to_string());
    let _ = fs::write(work_abs.join(".RUNNING"), b"");

    let res = run_wasi_module(&app, &job_id, run_input);

    let _ = fs::remove_file(work_abs.join(".RUNNING"));
    let _ = fs::remove_dir_all(&work_abs);

    RUNNING_JOBS.fetch_sub(1, Ordering::SeqCst);

    let out = res.map_err(|e| e.to_string())?;
    serde_json::to_value(out).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_sandbox_list_modules(app: AppHandle) -> Result<Vec<String>, String> {
    let dir = external_modules_dir(&app).map_err(|e| e.to_string())?;
    let mut out = vec![];
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let e = entry.map_err(|e| e.to_string())?;
        if e.path().is_file() && e.path().extension().map(|x| x == "wasm").unwrap_or(false) {
            out.push(e.file_name().to_string_lossy().to_string());
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn bd_sandbox_save_module(app: AppHandle, name: String, contents: Vec<u8>) -> Result<bool, String> {
    let base = external_modules_dir(&app).map_err(|e| e.to_string())?;
    if name.contains(std::path::is_separator) { return Err("Invalid module name".into()); }
    let target = base.join(&name);
    if let Some(parent) = target.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    fs::write(&target, &contents).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Opens a file dialog (filtered to .wasm), reads bytes via `bd_file_open_with_bytes`,
/// and saves (overwrites if exists) the selected module into `_external_modules`.
/// If `default_name` is None, it uses the picked file's basename.
#[tauri::command]
pub async fn bd_sandbox_pick_and_save_module(
    app: AppHandle,
    default_name: Option<String>,
    max_bytes: Option<u64>,
) -> Result<serde_json::Value, String> {
    // 1) Ask user to pick exactly one .wasm file and read it as bytes
    let allowed = Some(vec!["wasm".to_string()]);
    let picked = bd_file_open_with_bytes(app.clone(), /* multi */ false, allowed, max_bytes)
        .await?;

    let file = match picked.files.into_iter().next() {
        Some(f) => f,
        None => return Err("no file selected".into()),
    };

    // 2) Determine target name: default_name or basename of picked path
    let name = match default_name {
        Some(n) if !n.trim().is_empty() => n,
        _ => {
            Path::new(&file.path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "cannot derive file name".to_string())?
        }
    };

    // 3) Save/overwrite into _external_modules
    let base = external_modules_dir(&app).map_err(|e| e.to_string())?;
    if name.contains(std::path::is_separator) {
        return Err("Invalid module name".into());
    }
    let target = base.join(&name);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&target, &file.bytes).map_err(|e| e.to_string())?;

    // 4) Return a small JSON summary
    Ok(serde_json::json!({
        "saved": true,
        "name": name,
        "path": target.to_string_lossy(),
        "bytes": file.bytes.len(),
    }))
}

#[tauri::command]
pub fn bd_sandbox_delete_module(app: AppHandle, name: String) -> Result<bool, String> {
    let base = external_modules_dir(&app).map_err(|e| e.to_string())?;
    if name.contains(std::path::is_separator) { return Err("Invalid module name".into()); }
    let target = base.join(&name);
    if target.exists() { fs::remove_file(&target).map_err(|e| e.to_string())?; }
    Ok(true)
}

#[tauri::command]
pub fn bd_sandbox_remove_module(app: AppHandle, name: String) -> Result<bool, String> {
    bd_sandbox_delete_module(app, name)
}

#[tauri::command]
pub fn bd_sandbox_paths(app: AppHandle) -> Result<serde_json::Value, String> {
    let m = external_modules_dir(&app).map_err(|e| e.to_string())?;
    let s = sandbox_dir(&app).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "externalModules": m.to_string_lossy(),
        "sandbox": s.to_string_lossy(),
    }))
}

#[tauri::command]
pub fn bd_sandbox_clear_all(app: AppHandle) -> Result<bool, String> {
    wipe_sandbox_all(&app).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn bd_sandbox_cleanup_on_boot(app: AppHandle) -> Result<bool, String> {
    sandbox_cleanup_on_boot(&app).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn bd_sandbox_list_active(app: AppHandle) -> Result<Vec<String>, String> {
    let base = sandbox_dir(&app).map_err(|e| e.to_string())?;
    let mut out = vec![];
    if base.exists() {
        for e in fs::read_dir(base).map_err(|e| e.to_string())? {
            let e = e.map_err(|e| e.to_string())?;
            let path = e.path();
            if path.is_dir() && path.join(".RUNNING").exists() {
                if let Some(name) = path.file_name() { out.push(name.to_string_lossy().to_string()); }
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn bd_sandbox_sweep(app: AppHandle) -> Result<u32, String> {
    let base = sandbox_dir(&app).map_err(|e| e.to_string())?;
    let ttl = Duration::from_secs(TTL_MINUTES.load(Ordering::SeqCst).saturating_mul(60));
    let now = SystemTime::now();
    let mut removed = 0u32;

    if base.exists() {
        for e in fs::read_dir(&base).map_err(|e| e.to_string())? {
            let e = e.map_err(|e| e.to_string())?;
            let p = e.path();
            if p.is_dir() && !p.join(".RUNNING").exists() {
                if let Some(mt) = mtime(&p) {
                    if now.duration_since(mt).unwrap_or_default() > ttl {
                        if fs::remove_dir_all(&p).is_ok() { removed += 1; }
                    }
                }
            }
        }
    }
    Ok(removed)
}

#[tauri::command]
pub fn bd_sandbox_set_concurrency_limit(limit: u32) -> Result<u32, String> {
    let lim = limit.clamp(1, 32);
    CONCURRENCY_LIMIT.store(lim as usize, Ordering::SeqCst);
    Ok(lim)
}

#[tauri::command]
pub fn bd_sandbox_get_concurrency() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "limit": CONCURRENCY_LIMIT.load(Ordering::SeqCst),
        "running": RUNNING_JOBS.load(Ordering::SeqCst)
    }))
}

#[tauri::command]
pub fn bd_sandbox_set_ttl_minutes(minutes: u64) -> Result<u64, String> {
    let m = minutes.clamp(5, 24 * 60);
    TTL_MINUTES.store(m, Ordering::SeqCst);
    Ok(m)
}

#[tauri::command]
pub fn bd_sandbox_get_ttl_minutes() -> Result<u64, String> {
    Ok(TTL_MINUTES.load(Ordering::SeqCst))
}
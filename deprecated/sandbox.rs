// src/bridge/sandbox.rs
// All comments are in English as requested.

use std::{
    fs,
    env,
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

// Wasmtime/WASI (Preview1 in wasmtime-wasi v21)
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtxBuilder, DirPerms, FilePerms, I32Exit};
use wasmtime_wasi::preview1::add_to_linker_sync;
use wasmtime_wasi::preview1::WasiP1Ctx;

// ✅ Stdio pipes that implement StdinStream/StdoutStream in wasmtime-wasi v21
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe, ClosedInputStream, SinkOutputStream};
use bytes::Bytes;

// Helpers
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
// Reuse your helper
use crate::bridge::files::bd_file_open_with_bytes;

/// Store data: WASI ctx + optional limiter
struct StoreState {
    wasi: WasiP1Ctx,
    limiter: Option<StoreLimiter>,
}

/// ======================================================
/// Global state
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
    pub timeout_ms: Option<u64>,    // epoch interruption timeout
    pub memory_mb: Option<usize>,   // linear memory limit
    pub cpu_fuel: Option<u64>,      // not used here (kept for compat)
    pub stdout_max_kb: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SandboxRunInput {
    pub module_path: String,                 // e.g. "_external_modules/my.wasm"
    pub args: Option<Vec<String>>,           // kept for compat; not used (we do stdin-only)
    pub env: Option<std::collections::HashMap<String, String>>,
    pub stdin: Option<String>,               // JSON to stdin (newline appended)
    pub caps: Option<SandboxCaps>,
    pub working_dir: Option<String>,         // "_sandbox/<jobId>"
}

/// High-level JSON call
#[derive(Debug, Deserialize, Clone)]
pub struct SandboxCallInputJson {
    pub module_path: String,
    pub payload: serde_json::Value,
    pub caps: Option<SandboxCaps>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct RunStats {
    pub wall_ms: u128,
    pub fuel_used: Option<u64>,
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
    pub value: Option<serde_json::Value>,
    pub stats: Option<RunStats>,
}

/// ======================================================
/// Paths helpers
/// ======================================================

fn root_dir(app: &AppHandle) -> Result<PathBuf> {
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

// ---- WASM validators -------------------------------------------------------
const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D]; // "\0asm"

fn is_wasm_extension(path: &Path) -> bool {
    path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.eq_ignore_ascii_case("wasm"))
        .unwrap_or(false)
}

fn validate_wasm_file(path: &Path) -> Result<()> {
    if !is_wasm_extension(path) {
        return Err(anyhow!(
            "invalid module: expected .wasm file (got {})",
            path.display()
        ));
    }
    let bytes = fs::read(path).with_context(|| format!("cannot read module: {}", path.display()))?;
    if bytes.len() < 4 || &bytes[..4] != WASM_MAGIC {
        return Err(anyhow!("invalid module: missing WASM magic header (\\0asm)"));
    }
    Ok(())
}

fn validate_wasm_name_and_bytes(name: &str, bytes: &[u8]) -> Result<()> {
    let as_path = Path::new(name);
    if !is_wasm_extension(as_path) {
        return Err(anyhow!("invalid module name: expected .wasm extension"));
    }
    if bytes.len() < 4 || &bytes[..4] != WASM_MAGIC {
        return Err(anyhow!("invalid module: missing WASM magic header (\\0asm)"));
    }
    Ok(())
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
/// Core run (WASI/_start) — stdin-only JSON
/// ======================================================

fn run_wasi_module(app: &AppHandle, job_id: &str, input: SandboxRunInput) -> Result<SandboxRunResult> {
    let caps = input.caps.clone().unwrap_or(SandboxCaps {
        timeout_ms: Some(15_000),
        memory_mb: Some(256),
        cpu_fuel: Some(5_000_000),
        stdout_max_kb: Some(512),
    });
    // Keep env; ignore args (stdin-only)
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

    // Debug artifacts
    let _ = fs::write(work_abs.join("_stdin.json"), input.stdin.clone().unwrap_or_default());
    if let Some(s) = input.stdin.as_ref() {
        let _ = fs::write(work_abs.join("_stdin_len.txt"), s.len().to_string());
    } else {
        let _ = fs::write(work_abs.join("_stdin_len.txt"), "0");
    }
    let _ = fs::write(
        work_abs.join("_meta.txt"),
        format!(
            "module_path={}\nmode=stdin-only-json\nenv-keys={:?}\n",
            module_path.display(),
            input.env.as_ref().map(|m| m.keys().cloned().collect::<Vec<_>>())
        ),
    );
    let stage_path = work_abs.join("_host_stage.txt");
    let mut _stage = String::new();
    _stage.push_str("1: begin\n");
    let _ = fs::write(&stage_path, &_stage);

    // --- stdio pipes (Memory* from wasmtime_wasi::pipe) ---
    let stdin_is_some = input.stdin.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
    let stdout_pipe = MemoryOutputPipe::new(4 * 1024 * 1024); // 4 MiB
    let stderr_pipe = MemoryOutputPipe::new(1 * 1024 * 1024); // 1 MiB

    // Decide timeout now so we can tune engine/store accordingly
    let timeout_ms = caps.timeout_ms.unwrap_or(0);
    // Wasmtime engine config
    let mut cfg = Config::new();
    cfg.wasm_multi_value(true);
    // Enable epoch interruption ONLY when a timeout is requested
    if timeout_ms > 0 {
        cfg.epoch_interruption(true);
    }

    let engine = Engine::new(&cfg)?;
    let module = Module::from_file(&engine, &module_path)?;
    _stage.push_str("2: module_loaded\n");
    let _ = fs::write(&stage_path, &_stage);
    // === DIAGNOSTIC: dump module imports/exports ===
    {
        use std::fmt::Write as _;
        let mut s = String::new();
        s.push_str("== IMPORTS ==\n");
        for imp in module.imports() {
            let _ = writeln!(
                &mut s,
                "{}.{} : {:?}",
                imp.module(),
                imp.name(),
                imp.ty()
            );
        }
        s.push_str("== EXPORTS ==\n");
        for exp in module.exports() {
            let _ = writeln!(
                &mut s,
                "{} : {:?}",
                exp.name(),
                exp.ty()
            );
        }
        let _ = fs::write(work_abs.join("_module_introspection.txt"), s);
    }

    // --- WASI ctx builder (Preview1) ---
    let mut wasi_builder = WasiCtxBuilder::new();
    // Send JSON on stdin (newline-terminated) then EOF; or closed stdin if none.
    if stdin_is_some {
        let mut payload = input.stdin.clone().unwrap();
        if !payload.ends_with('\n') { payload.push('\n'); }   // line-friendly
        let stdin_bytes = Bytes::from(payload.into_bytes());
        let stdin_pipe  = MemoryInputPipe::new(stdin_bytes);   // finite buffer -> EOF
        wasi_builder.stdin(stdin_pipe);
    } else {
        wasi_builder.stdin(ClosedInputStream);                 // immediate EOF
    }
    // Allow disabling stdout/stderr buffering to rule out backpressure deadlocks
    let sink_stdio = std::env::var("BUBBLEDESK_SINK_STDIO").map(|v| v == "1").unwrap_or(false);
    if sink_stdio {
        wasi_builder.stdout(SinkOutputStream); // drops all writes
        wasi_builder.stderr(SinkOutputStream);
    } else {
        wasi_builder.stdout(stdout_pipe.clone());
        wasi_builder.stderr(stderr_pipe.clone());
    }

    // argv: only argv[0] is needed; we do stdin-only
    let argv0 = module_path.file_name().unwrap_or_default().to_string_lossy().to_string();
    wasi_builder.args(&[argv0]);

    // env passthrough
    for (k, v) in env.iter() { wasi_builder.env(k, v); }

    // Preopen job dir as "/"
    wasi_builder.preopened_dir(
        &work_abs,
        "/",
        DirPerms::READ | DirPerms::MUTATE,
        FilePerms::READ | FilePerms::WRITE,
    );

    let wasi_ctx = wasi_builder.build_p1();

    // --- Linker and instantiate_pre ---
    let mut linker: Linker<StoreState> = Linker::new(&engine);
    add_to_linker_sync(&mut linker, |cx: &mut StoreState| &mut cx.wasi)?;
    _stage.push_str("3: wasi_linked\n");
    let _ = fs::write(&stage_path, &_stage);

    // --- Store and limits ---
    let mut store = Store::new(&engine, StoreState { wasi: wasi_ctx, limiter: None });
    if timeout_ms > 0 {
        // Set a small deadline so the next epoch bump after timeout traps the guest
        store.set_epoch_deadline(1);
    }

    // DIAG: begin disable limiter
    /*
    if let Some(mb) = caps.memory_mb {
        let maxb = mb.saturating_mul(1024 * 1024);
        store.data_mut().limiter = Some(StoreLimiter { max_memory_bytes: maxb });
        store.limiter(|data| data.limiter.as_mut().expect("limiter set"));
    }
    */
    // DIAG: end disable limiter

    // Instantiate (no pre-instantiate, for diagnosis)
    let start_instant = Instant::now();
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(i) => i,
        Err(e) => {
            let _ = fs::write(work_abs.join("_host_error.txt"), format!("instantiate error: {e:?}"));
            return Err(anyhow!("instantiate failed: {e}"));
        }
    };
    _stage.push_str("4: instantiated\n");
    let _ = fs::write(&stage_path, &_stage);

    // Timeout via epoch interruption
    if timeout_ms > 0 {
        let engine_clone = engine.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(timeout_ms));
            // Burst increments to guarantee an interrupt is observed soon after deadline
            for _ in 0..256 {
                engine_clone.increment_epoch();
                std::hint::spin_loop();
            }
        });
    }

    // Resolve and call `_start`
    let start_func = match instance.get_typed_func::<(), ()>(&mut store, "_start") {
        Ok(f) => f,
        Err(e) => {
            let _ = fs::write(work_abs.join("_host_error.txt"), format!("get _start error: {e:?}"));
            return Err(anyhow!("_start not found/typed: {e}"));
        }
    };
    _stage.push_str("5: start_resolved\n");
    let _ = fs::write(&stage_path, &_stage);

    _stage.push_str("6: before_start_call\n");
    let _ = fs::write(&stage_path, &_stage);
    // DIAGNOSTIC breadcrumb
    let _ = fs::write(work_abs.join("_reached_start_call.txt"), b"1");
    // Drop the JSON payload into the preopened "/" for guest-side manual tests
    if let Some(s) = input.stdin.as_ref() {
        let _ = fs::write(work_abs.join("stdin_payload.json"), s);
    }

    let call_res: Result<i32> = match start_func.call(&mut store, ()) {
        Ok(()) => {
            let _ = fs::write(work_abs.join("_start_ok.txt"), b"1");
            _stage.push_str("7: start_returned_ok\n");
            let _ = fs::write(&stage_path, &_stage);
            Ok(0)
        }
        Err(e) => {
            let _ = fs::write(work_abs.join("_start_err.txt"), format!("{e:?}"));
            _stage.push_str("7: start_trapped\n");
            let _ = fs::write(&stage_path, &_stage);
            if let Some(exit) = e.downcast_ref::<I32Exit>() {
                Err(anyhow!(format!("__EXIT__{}", exit.0)))
            } else {
                let _ = fs::write(work_abs.join("_host_error.txt"), format!("start trap: {e:?}"));
                Err(anyhow!(e.to_string()))
            }
        }
    };

    // No fuel accounting here
    let fuel_used: Option<u64> = None;
    drop(store);

    // --- Drain stdout/stderr from MemoryOutputPipe ---
    let out = if let Some(buf) = stdout_pipe.try_into_inner() { buf.to_vec() } else { Vec::new() };
    let err = if let Some(buf) = stderr_pipe.try_into_inner() { buf.to_vec() } else { Vec::new() };
    let stdout_str = String::from_utf8_lossy(&out).to_string();
    let stderr_str = String::from_utf8_lossy(&err).to_string();

    let wall_ms = start_instant.elapsed().as_millis();

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

    // Cap stdout length in KB
    let stdout_capped = if let Some(kb) = caps.stdout_max_kb {
        let limit = kb * 1024;
        if stdout_str.len() > limit { stdout_str[..limit].to_string() } else { stdout_str }
    } else { stdout_str };

    // Best-effort JSON parse
    let value = match serde_json::from_str::<serde_json::Value>(&stdout_capped) {
        Ok(v) => Some(v),
        Err(_) => None,
    };

    Ok(SandboxRunResult {
        ok,
        job_id: job_id.to_string(),
        work_dir: work_abs.to_string_lossy().to_string(),
        exit_code: ec,
        stdout: if value.is_none() { Some(stdout_capped) } else { None },
        stderr: if !stderr_str.is_empty() { Some(stderr_str) } else { None },
        value,
        stats: Some(RunStats { wall_ms, fuel_used, peak_mb: caps.memory_mb }),
    })
}

/// ======================================================
/// Tauri commands (unchanged in behavior)
/// ======================================================

#[tauri::command]
pub async fn bd_sandbox_call(app: AppHandle, input: SandboxCallInputJson) -> Result<serde_json::Value, String> {
    // Concurrency gate
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

    // Validate module
    let mod_abs = match resolve_module_path(&app, &input.module_path) {
        Ok(p) => p,
        Err(e) => {
            RUNNING_JOBS.fetch_sub(1, Ordering::SeqCst);
            return Err(e.to_string());
        }
    };
    if let Err(e) = validate_wasm_file(&mod_abs) {
        RUNNING_JOBS.fetch_sub(1, Ordering::SeqCst);
        return Err(e.to_string());
    }

    // Create job workspace
    let job_id = gen_job_id();
    let work_abs = job_sandbox_dir(&app, &job_id).map_err(|e| e.to_string())?;
    let _ = fs::write(work_abs.join(".RUNNING"), b"");

    // Always send stdin JSON (newline appended later in runner)
    let payload_json = if let Some(s) = input.payload.as_str() {
        serde_json::from_str::<serde_json::Value>(s).unwrap_or(input.payload.clone())
    } else {
        input.payload.clone()
    };
    let stdin_body = serde_json::to_string(&payload_json).map_err(|e| e.to_string())?;

    let run_input = SandboxRunInput {
        module_path: input.module_path.clone(),
        args: None, // stdin-only
        env: input.env.clone(),
        stdin: Some(stdin_body),
        caps: input.caps.clone(),
        working_dir: Some(work_abs.to_string_lossy().to_string()),
    };

    let res = run_wasi_module(&app, &job_id, run_input);

    // Cleanup this job unless KEEP flag is set
    let keep = std::env::var("BUBBLEDESK_KEEP_SANDBOX").map(|v| v == "1").unwrap_or(false);
    if !keep {
        let _ = fs::remove_file(work_abs.join(".RUNNING"));
        let _ = fs::remove_dir_all(&work_abs);
    } else {
        eprintln!("[sandbox] KEEP enabled; job dir: {}", work_abs.display());
    }

    RUNNING_JOBS.fetch_sub(1, Ordering::SeqCst);

    let out = res.map_err(|e| e.to_string())?;
    serde_json::to_value(out).map_err(|e| e.to_string())
}

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

    let keep = std::env::var("BUBBLEDESK_KEEP_SANDBOX").map(|v| v == "1").unwrap_or(false);
    if !keep {
        let _ = fs::remove_file(work_abs.join(".RUNNING"));
        let _ = fs::remove_dir_all(&work_abs);
    } else {
        eprintln!("[sandbox] KEEP enabled; job dir: {}", work_abs.display());
    }

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

    if let Err(e) = validate_wasm_name_and_bytes(&name, &contents) {
        return Err(e.to_string());
    }

    let target = base.join(&name);
    if let Some(parent) = target.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    fs::write(&target, &contents).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn bd_sandbox_pick_and_save_module(
    app: AppHandle,
    default_name: Option<String>,
    max_bytes: Option<u64>,
) -> Result<serde_json::Value, String> {
    let allowed = Some(vec!["wasm".to_string()]);
    let picked = bd_file_open_with_bytes(app.clone(), false, allowed, max_bytes).await?;

    let file = match picked.files.into_iter().next() {
        Some(f) => f,
        None => return Err("no file selected".into()),
    };

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

    let base = external_modules_dir(&app).map_err(|e| e.to_string())?;
    if name.contains(std::path::is_separator) {
        return Err("Invalid module name".into());
    }
    let target = base.join(&name);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&target, &file.bytes).map_err(|e| e.to_string())?;

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
use tauri::{AppHandle, Manager, WebviewWindowBuilder, WebviewUrl, Emitter, Listener};
use std::{
  fs,
  io::{Read, Write},
  path::{Path, PathBuf},
  collections::HashMap,
  sync::atomic::{AtomicU64, AtomicUsize, Ordering, AtomicBool},
  sync::{Arc, Mutex}
};
use serde_json::json;
use anyhow::{anyhow, Context, Result};

// Reuse your helper
use crate::bridge::files::bd_file_open_with_bytes;

// One-shot mailbox used by the webview to signal readiness
static READY_TX: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>> = std::sync::Mutex::new(None);

static REQ_COUNTER: AtomicU64 = AtomicU64::new(1);
static WORKER_READY: AtomicBool = AtomicBool::new(false);
static RESP_LISTENER: AtomicBool = AtomicBool::new(false);

// Queue/backpressure limit for pending jobs
const MAX_PENDING: usize = 64; // tune if needed

fn pending_len() -> usize {
  let p = pending();
  let guard = p.map.lock().unwrap();
  guard.len()
}

fn gen_id() -> String {
  // Simple monotonic id without extra crates
  let n = REQ_COUNTER.fetch_add(1, Ordering::SeqCst);
  format!("req{:x}", n)
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct WorkerRequest {
  id: String,
  // raw wasm bytes to avoid FS/fetch in the worker
  wasm_bytes: Vec<u8>,
  // arbitrary JSON payload; your contract is stdin-like JSON
  payload: serde_json::Value,
  wasi: bool,
  timeout_ms: u64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct WorkerResponse {
  id: String,
  ok: bool,
  stdout: Option<String>,
  stderr: Option<String>,
  value: Option<serde_json::Value>,
  error: Option<String>,
}

// Pending map to await replies
struct Pending {
  map: Mutex<HashMap<String, tokio::sync::oneshot::Sender<serde_json::Value>>>,
}
static mut PENDING: Option<Arc<Pending>> = None;

fn pending() -> Arc<Pending> {
  unsafe {
    PENDING.get_or_insert_with(|| Arc::new(Pending { map: Mutex::new(HashMap::new()) })).clone()
  }
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

fn ensure_response_listener(app: &AppHandle) {
  // Register only once
  if RESP_LISTENER
    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
    .is_ok()
  {
    let _unlisten = app.listen("bd:worker:resp", move |ev| {
      // ev.payload() is a &str with serialized JSON
      let payload = ev.payload();
      if let Ok(val) = serde_json::from_str::<serde_json::Value>(payload) {
        let id = val.get("id").and_then(|x| x.as_str()).unwrap_or_default().to_string();
        if !id.is_empty() {
          let tx_opt = {
            let p = pending();
            let mut g = p.map.lock().unwrap();
            g.remove(&id)
          };
          if let Some(tx) = tx_opt {
            let _ = tx.send(val);
          }
        }
      }
    });
    // _unlisten intentionally dropped at app shutdown; keep listener alive
    let _ = _unlisten; 
  }
}

#[tauri::command]
pub fn bd_worker_status() -> serde_json::Value {
  serde_json::json!({
    "ready": WORKER_READY.load(std::sync::atomic::Ordering::SeqCst),
    "pending": pending_len()
  })
}


#[tauri::command]
pub async fn bd_worker_restart(app: AppHandle) -> Result<bool, String> {
  // All comments in English.
  // 1) Close existing window if present and wait until it's actually gone
  if let Some(win) = app.get_webview_window("bd-worker") {
    let _ = win.close();
    // Poll until manager no longer returns the window (up to ~2s)
    for _ in 0..40 {
      if app.get_webview_window("bd-worker").is_none() { break; }
      tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
  }

  // 2) Reset readiness flags and mailbox
  WORKER_READY.store(false, std::sync::atomic::Ordering::SeqCst);
  *READY_TX.lock().unwrap() = None;

  // 3) Prepare a fresh mailbox and spawn
  let (tx, rx) = tokio::sync::oneshot::channel::<()>();
  *READY_TX.lock().unwrap() = Some(tx);

  spawn_hidden_worker_window(&app).map_err(|e| e.to_string())?;

  // 4) Await readiness (up to 10s) so restart is complete in a single call
  let res = tokio::time::timeout(std::time::Duration::from_millis(10_000), rx).await;
  match res {
    Ok(Ok(())) => Ok(true),
    _ => Err("worker did not become ready after restart".into()),
  }
}

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

fn worker_sandbox_dir(app: &AppHandle) -> Result<PathBuf> {
    let p = root_dir(app)?.join("_sandbox");
    let _ = fs::create_dir_all(&p);
    Ok(p)
}
fn external_modules_dir(app: &AppHandle) -> Result<PathBuf> {
    let p = root_dir(app)?.join("_external_modules");
    let _ = fs::create_dir_all(&p);
    Ok(p)
}

fn wipe_worker_sandbox_all(app: &AppHandle) -> Result<()> {
    let s = worker_sandbox_dir(app)?;
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

#[tauri::command]
pub async fn bd_worker_ready(_app: tauri::AppHandle) -> Result<bool, String> {
  if let Some(tx) = READY_TX.lock().unwrap().take() {
    let _ = tx.send(());
    WORKER_READY.store(true, Ordering::SeqCst);
    Ok(true)
  } else {
    Ok(false)
  }
}

async fn ensure_worker_ready(app: &AppHandle) -> Result<(), String> {
  // Ensure the response listener is installed once
  ensure_response_listener(app);

  // Fast path
  if WORKER_READY.load(Ordering::SeqCst) {
    return Ok(());
  }

  // Install a mailbox for this attempt BEFORE spawning the window
  let (tx, rx) = tokio::sync::oneshot::channel::<()>();
  *READY_TX.lock().unwrap() = Some(tx);

  // Spawn the window (no-op if already present). This also injects the
  // script and host-side `eval` that will call `bd_worker_ready` repeatedly.
  spawn_hidden_worker_window(app).map_err(|e| e.to_string())?;

  // Wait for the ready signal (with safety timeout)
  let res = tokio::time::timeout(std::time::Duration::from_millis(10_000), rx).await;
  match res {
    Ok(Ok(())) => Ok(()),
    _ => Err("bd-worker not ready".into()),
  }
}

// Eager-ready kick-off at app startup: primes the READY_TX mailbox, spawns the window,
// and waits for the ready signal in the background so the first call won't 404 on readiness.
pub fn kickoff_worker_readiness(app: &AppHandle) {
  // If already ready, nothing to do.
  if WORKER_READY.load(Ordering::SeqCst) {
    return;
  }
  // Ensure the response listener is installed once
  ensure_response_listener(app);

  // Prepare a fresh mailbox so the webview's invoke("bd_worker_ready") has somewhere to send to.
  let (tx, rx) = tokio::sync::oneshot::channel::<()>();
  *READY_TX.lock().unwrap() = Some(tx);

  // Spawn (or no-op if already present). This injects the script and eval that will invoke bd_worker_ready repeatedly.
  if let Err(e) = spawn_hidden_worker_window(app) { eprintln!("[bd-worker] eager spawn failed: {e}"); }

  // Wait in background; don't block setup.
  let _app = app.clone();
  tauri::async_runtime::spawn(async move {
    let _ = tokio::time::timeout(std::time::Duration::from_millis(10_000), rx).await;
    // Note: on success, bd_worker_ready() sets WORKER_READY itself. On timeout, we simply log next call will retry.
    let _ = _app; // keep handle owned for lifetime symmetry
  });
}

// Call this at setup (once) to spawn the hidden webview
pub fn spawn_hidden_worker_window(app: &AppHandle) -> tauri::Result<()> {
  if app.get_webview_window("bd-worker").is_some() {
    return Ok(());
  }

  // Embed the bundled worker code (ensure `pnpm build:worker` ran so this file exists)
  const WORKER_BUNDLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/worker/bd-wasm-runner.worker.js"));
  // JSON-stringify the code to produce a safe JS string literal
  let worker_code_js_literal = serde_json::to_string(WORKER_BUNDLE).unwrap_or_else(|_| "\"\"".into());

  // Build an initialization script that:
  // - announces readiness
  // - listens for bd:worker:req
  // - creates a Worker from an in-memory Blob of the bundled JS (so we don't rely on asset URLs)
  let init_js = format!(r#"
    (() => {{
      try {{
        console.log("[bd-worker bootstrap] injected script running");
        const T = window.__TAURI__;
        // Proactively invoke the Rust-side ready command a few times to guarantee handshake
        try {{ T?.core?.invoke("bd_worker_ready"); }} catch (_) {{}}
        setTimeout(() => {{ try {{ T?.core?.invoke("bd_worker_ready"); }} catch (_) {{}} }}, 50);
        setTimeout(() => {{ try {{ T?.core?.invoke("bd_worker_ready"); }} catch (_) {{}} }}, 200);
        console.log("[bd-worker bootstrap] typeof window.__TAURI__ =", typeof T);
        if (!T || !T.event || !T.core) {{
          console.warn("[bd-worker] __TAURI__ not ready at injection time; scheduling delayed ready emits");
        }}
        const WORKER_CODE = {worker_code};

        let __bd_worker;
        function getWorker() {{
          if (__bd_worker) return __bd_worker;
          const blob = new Blob([WORKER_CODE], {{ type: "text/javascript" }});
          const url = URL.createObjectURL(blob);
          __bd_worker = new Worker(url, {{ type: "module" }});
          console.log("[bd-worker bootstrap] worker created via Blob URL");
          return __bd_worker;
        }}

        // Tell Rust we're ready to receive jobs (emit repeatedly to avoid race)
        const emitReady = () => {{
          try {{
            const TT = window.__TAURI__;
            if (!TT || !TT.event) {{ console.warn("[bd-worker] emitReady: __TAURI__.event missing"); return; }}
            TT.event.emit("bd:worker:ready");
            console.log("[bd-worker bootstrap] emitted bd:worker:ready");
          }} catch (e) {{
            console.error("[bd-worker] emitReady error:", e);
          }}
        }};
        emitReady();
        setTimeout(emitReady, 0);
        setTimeout(emitReady, 50);
        setTimeout(emitReady, 100);
        setTimeout(emitReady, 300);
        setTimeout(emitReady, 500);

        // Also try after DOMContentLoaded just in case
        window.addEventListener("DOMContentLoaded", () => {{
          console.log("[bd-worker bootstrap] DOMContentLoaded -> emit ready");
          emitReady();
        }});

        // Listen for jobs from Rust
        const attachListener = () => {{
          try {{
            const TT = window.__TAURI__;
            if (!TT || !TT.event || !TT.core) {{
              console.warn("[bd-worker] attachListener: __TAURI__ not ready yet");
              setTimeout(attachListener, 100);
              return;
            }}
            TT.event.listen("bd:worker:req", (ev) => {{
              try {{
                const req = ev.payload;
                const w = getWorker();

                const chan = new MessageChannel();
                chan.port1.onmessage = async (msg) => {{
                  const out = Object.assign({{ id: req.id }}, msg.data || {{}});
                  TT.event.emit("bd:worker:resp", out);
                  chan.port1.close();
                }};

                console.log("[bd-worker bootstrap] received job", req?.id);
                w.postMessage(
                  {{
                    id: req.id,
                    wasmBytes: new Uint8Array(req.wasm_bytes),
                    payload: req.payload,
                    wasi: !!req.wasi,
                    timeoutMs: req.timeout_ms
                  }},
                  [chan.port2]
                );
              }} catch (e) {{
                console.error("[bd-worker] job handler error:", e);
                TT.event.emit("bd:worker:resp", {{ id: "unknown", ok: false, error: String(e) }});
              }}
            }});
            console.log("[bd-worker bootstrap] listener attached");
          }} catch (e) {{
            console.error("[bd-worker] attachListener error:", e);
          }}
        }};
        attachListener();
        console.log("[bd-worker bootstrap] init script end");
      }} catch (e) {{
        console.error("[bd-worker] init error:", e);
      }}
    }})();
  "#, worker_code = worker_code_js_literal);

  // Visibility/devtools based on build profile
  #[cfg(debug_assertions)]
  let (visible, devtools) = (false, false); // dev environment
  #[cfg(not(debug_assertions))]
  let (visible, devtools) = (false, false); // prod environment

  let win = WebviewWindowBuilder::new(app, "bd-worker", WebviewUrl::App("blank.html".into()))
    .title("Bubbledesk Worker")
    .visible(visible)
    .resizable(false)
    .devtools(devtools)
    .initialization_script(init_js)
    .build()?;

  #[cfg(debug_assertions)]
  {
    let _ = win.open_devtools();
  }

  // Also poke from host side: call bd_worker_ready a few times from inside the webview
  let _ = win.eval(r#"
    (function pokeReady(){
      let n = 0;
      const tick = () => {
        try { window.__TAURI__?.core?.invoke('bd_worker_ready'); } catch (e) {}
        if (++n < 40) setTimeout(tick, 50);
      };
      tick();
    })();
  "#);

  Ok(())
}

// Utility: read wasm bytes from _external_modules
fn read_wasm_module(app: &AppHandle, name: &str) -> anyhow::Result<Vec<u8>> {
  // Reuse your existing helper/path logic if you have it
  let root = app.path().app_data_dir().map_err(|_| anyhow::anyhow!("no app_data_dir"))?;
  let base = root.parent().ok_or_else(|| anyhow::anyhow!("no parent"))?.join("_external_modules");
  let p = if std::path::Path::new(name).is_absolute() { std::path::PathBuf::from(name) } else { base.join(name) };
  let bytes = std::fs::read(&p).map_err(|e| anyhow::anyhow!("read {}: {}", p.display(), e))?;
  Ok(bytes)
}

// Command: Rust -> hidden window -> worker; await response
#[tauri::command]
pub async fn bd_worker_call(
  app: AppHandle,
  module_path: String,
  payload: serde_json::Value,
  wasi: Option<bool>,
  timeout_ms: Option<u64>,
) -> Result<serde_json::Value, String> {
  let t0 = std::time::Instant::now();
  // ensure hidden window exists and is ready
  ensure_worker_ready(&app).await?;

  // Backpressure: refuse if too many pending jobs
  if pending_len() >= MAX_PENDING {
    return Err("queue_full".into());
  }

  // prepare request
  let id = gen_id();
  let wasm_bytes = read_wasm_module(&app, &module_path).map_err(|e| e.to_string())?;
  let req = WorkerRequest {
    id: id.clone(),
    wasm_bytes,
    payload,
    wasi: wasi.unwrap_or(true),        // default to WASI polyfill
    timeout_ms: timeout_ms.unwrap_or(2000),
  };

  // register pending
  let (tx, rx) = tokio::sync::oneshot::channel();
  {
    let p = pending();
    p.map.lock().unwrap().insert(id.clone(), tx);
  }

  // emit to hidden window
  app.emit_to("bd-worker", "bd:worker:req", req.clone()).map_err(|e| e.to_string())?;

  // wait for response or timeout
  let dur = std::time::Duration::from_millis(req.timeout_ms + 5000);
  let mut resp = tokio::time::timeout(dur, rx).await.map_err(|_| "worker timeout".to_string())?
    .map_err(|_| "worker cancelled".to_string())?;

  let elapsed_ms = t0.elapsed().as_millis() as u64;
  if let serde_json::Value::Object(ref mut m) = resp {
    m.insert("duration_ms".into(), serde_json::json!(elapsed_ms));
  }
  Ok(resp)
}

// Command: called from hidden window to deliver result
#[tauri::command]
pub async fn bd_worker_delivery(_app: AppHandle, id: String, result: serde_json::Value) -> Result<bool, String> {
  let p = pending();
  let tx_opt = {
    let mut guard = p.map.lock().unwrap();
    guard.remove(&id)
  };
  if let Some(tx) = tx_opt {
    let _ = tx.send(result);
    Ok(true)
  } else {
    Err("unknown request id".into())
  }
}

// Convenience for setup(|app| ...) where you receive &mut tauri::App
pub fn spawn_hidden_worker_window_from_app(app: &mut tauri::App) -> tauri::Result<()> {
  spawn_hidden_worker_window(&app.handle())
}

#[tauri::command]
pub fn bd_worker_add_module(app: AppHandle, name: String, contents: Vec<u8>) -> Result<bool, String> {
    // ---- size limit: 20 MB default
    let max_size: usize = 20 * 1024 * 1024;
    if contents.len() > max_size {
        return Err(format!(
            "module too large ({} bytes > {} bytes)",
            contents.len(), max_size
        ).into());
    }
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
pub async fn bd_worker_pick_and_add_module(
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
pub fn bd_worker_remove_module(app: AppHandle, name: String) -> Result<bool, String> {
    let base = external_modules_dir(&app).map_err(|e| e.to_string())?;
    if name.contains(std::path::is_separator) { return Err("Invalid module name".into()); }
    let target = base.join(&name);
    if target.exists() { fs::remove_file(&target).map_err(|e| e.to_string())?; }
    Ok(true)
}

#[tauri::command]
pub fn bd_worker_paths(app: AppHandle) -> Result<serde_json::Value, String> {
    let m = external_modules_dir(&app).map_err(|e| e.to_string())?;
    let s = worker_sandbox_dir(&app).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "externalModules": m.to_string_lossy(),
        "sandbox": s.to_string_lossy(),
    }))
}

#[tauri::command]
pub fn bd_worker_clear_all(app: AppHandle) -> Result<bool, String> {
    wipe_worker_sandbox_all(&app).map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn worker_sandbox_cleanup_on_boot(app: &AppHandle) -> Result<()> {
    wipe_worker_sandbox_all(app)?;
    Ok(())
}

#[tauri::command]
pub fn bd_worker_list_modules(app: AppHandle) -> Result<Vec<String>, String> {
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
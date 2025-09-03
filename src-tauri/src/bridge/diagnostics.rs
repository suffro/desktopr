use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
    thread,
    sync::atomic::{AtomicBool, Ordering},
};

use chrono::Utc;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use crate::bridge::app::{bd_app_info, AppInfo as AppInfoStruct}; // adatta se il path è diverso

// ==============================
// Heartbeat / runtime markers
// ==============================

static HEARTBEAT_STOP: AtomicBool = AtomicBool::new(false);

fn data_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().expect("app_data_dir not available")
}

fn runtime_dir(app: &AppHandle) -> PathBuf {
    data_dir(app).join("runtime")
}

/// Start a background heartbeat that writes "runtime/heartbeat.json" every few seconds.
/// On next startup, if heartbeat exists and no "shutdown.ok" is present, we record a dirty shutdown.
pub fn start_heartbeat(app: AppHandle) {
    let dir = runtime_dir(&app);
    let _ = fs::create_dir_all(&dir);
    let hb = dir.join("heartbeat.json");
    let ok = dir.join("shutdown.ok");

    // Startup check: previous dirty shutdown?
    let dirty = hb.exists() && !ok.exists();
    if dirty {
        let entry = serde_json::json!({
          "record_type": "dirty_shutdown",
          "timestamp": Utc::now().timestamp_millis() as u64,
          "note": "previous run did not shut down cleanly"
        });
        let crash_path = data_dir(&app).join(format!(
            "crashes/crash-{}.json",
            Utc::now().format("%Y-%m-%dT%H-%M-%SZ")
        ));
        let _ = fs::create_dir_all(crash_path.parent().unwrap());
        let _ = fs::write(&crash_path, serde_json::to_vec_pretty(&entry).unwrap());

        let log_path = data_dir(&app).join(format!(
            "logs/record-{}.jsonl",
            Utc::now().format("%Y-%m")
        ));
        let _ = append_jsonl(log_path, &entry);
    }

    // Reset markers for this new run
    let _ = fs::remove_file(&ok);

    // Background ticker
    thread::spawn(move || {
        while !HEARTBEAT_STOP.load(Ordering::Relaxed) {
            let _ = fs::write(
                &hb,
                serde_json::to_vec(&serde_json::json!({
                    "ts": Utc::now().to_rfc3339()
                }))
                .unwrap(),
            );
            thread::sleep(Duration::from_secs(5));
        }
        // On stop, write clean shutdown marker and remove heartbeat
        let _ = fs::write(runtime_dir(&app).join("shutdown.ok"), b"ok");
        let _ = fs::remove_file(runtime_dir(&app).join("heartbeat.json"));
    });
}

/// Stop heartbeat and write clean shutdown marker right now.
pub fn mark_clean_shutdown_now(app: &AppHandle) {
    HEARTBEAT_STOP.store(true, Ordering::Relaxed);
    let dir = runtime_dir(app);
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(dir.join("shutdown.ok"), b"ok");
    let _ = fs::remove_file(dir.join("heartbeat.json"));
}

// ==============================
// Settings
// ==============================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivacySettings {
    pub analytics_enabled: bool,
    pub crash_reports_enabled: bool,
    /// Retention (days) per area
    pub retention_days_logs: u32,      // default 180
    pub retention_days_analytics: u32, // default 180
    pub retention_days_crashes: u32,   // default 365
}

fn default_privacy() -> PrivacySettings {
    PrivacySettings {
        analytics_enabled: false,
        crash_reports_enabled: true,
        retention_days_logs: 180,
        retention_days_analytics: 180,
        retention_days_crashes: 365,
    }
}

fn privacy_path(app: &AppHandle) -> PathBuf {
    data_dir(app).join("settings/privacy.json")
}

fn ensure_dir(p: &PathBuf) -> std::io::Result<()> {
    if !p.exists() {
        fs::create_dir_all(p)?;
    }
    Ok(())
}

fn read_privacy(app: &AppHandle) -> PrivacySettings {
    let p = privacy_path(app);
    if let Ok(b) = fs::read(&p) {
        if let Ok(v) = serde_json::from_slice::<PrivacySettings>(&b) {
            return v;
        }
    }
    default_privacy()
}

fn write_privacy(app: &AppHandle, s: &PrivacySettings) -> Result<(), String> {
    let p = privacy_path(app);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?;
    fs::write(p, bytes).map_err(|e| e.to_string())
}

// ==============================
// Types for logs
// ==============================

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineno: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colno: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsRecord {
    pub name: String,
    #[serde(default)]
    pub props: serde_json::Value, // object
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    Error(ErrorPayload),
    Analytics(AnalyticsRecord),
    Any(Value), // fallback
}

// AppInfo: libero (qualsiasi JSON)
pub type AppInfoJson = Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogData {
    pub record_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>, // millis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    pub payload: Payload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_info: Option<AppInfoJson>,
}

#[derive(Debug, Serialize)]
pub struct ListedFile {
    pub rel_path: String, // e.g., "logs/record-2025-09.jsonl"
    pub bytes: u64,
    pub modified_ms: u64,
}

// ==============================
// Low-level logging helpers
// ==============================

static ROTATE_MAX_BYTES: usize = 10 * 1024 * 1024; // 10 MB
static LOG_MUTEX: Mutex<()> = Mutex::new(());

/// Rename "file.jsonl" -> "file.partN.jsonl" when size threshold is exceeded.
fn rotate_with_part_suffix(path: &Path) -> std::io::Result<()> {
    if let Ok(meta) = fs::metadata(path) {
        if meta.len() as usize >= ROTATE_MAX_BYTES {
            // Build "{stem}.partN.jsonl"
            let (stem, ext) = if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
                (stem.to_string(), format!(".{}", ext))
            } else {
                ("file".to_string(), String::new())
            };
            let mut n = 2;
            loop {
                let rotated_name = format!("{}.part{}{}", stem, n, ext);
                let rotated_path = path.with_file_name(rotated_name);
                if !rotated_path.exists() {
                    fs::rename(path, rotated_path)?;
                    break;
                }
                n += 1;
            }
        }
    }
    Ok(())
}

fn append_jsonl(path: PathBuf, value: &serde_json::Value) -> std::io::Result<()> {
    let _guard = LOG_MUTEX.lock().unwrap();
    if let Some(parent) = path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }
    if !path.exists() {
        fs::File::create(&path)?;
    }
    rotate_with_part_suffix(&path)?;
    let mut f = fs::OpenOptions::new().append(true).open(&path)?;
    let line = serde_json::to_string(value).unwrap();
    writeln!(f, "{}", line)?;
    Ok(())
}

fn new_log_data(
    app: &tauri::AppHandle,
    payload: Payload,
    record_type: String,
    env: Option<String>,
    app_version: Option<String>,
) -> LogData {
    // bd_app_info(AppHandle) -> Result<AppInfoStruct, String>
    let app_info_json: Option<Value> = bd_app_info(app.clone())
        .ok()
        .and_then(|ai: AppInfoStruct| serde_json::to_value(ai).ok());

    LogData {
        record_type,
        env,
        timestamp: Some(Utc::now().timestamp_millis() as u64),
        app_version,
        payload,
        app_info: app_info_json,
    }
}


// ==============================
// Public Tauri commands
// ==============================

/// Generic error record (also writes a crash file if enabled).
#[tauri::command]
pub fn bd_logs_record_error(
    app: AppHandle,
    payload: ErrorPayload,
    env: String,            // <-- Opzione A: String
    app_version: String,
) -> Result<(), String> {
    let settings = read_privacy(&app);

    // General application log (JSONL)
    let log_path = data_dir(&app).join(format!(
        "logs/record-{}.jsonl",
        Utc::now().format("%Y-%m")
    ));
    let log_data = new_log_data(
        &app,
        Payload::Error(payload),
        "error".to_string(),
        Some(env),                 // <-- passiamo Some(env)
        Some(app_version.clone()),
    );
    let entry = serde_json::to_value(&log_data).unwrap();
    append_jsonl(log_path, &entry).map_err(|e| e.to_string())?;

    // Crash store (if enabled)
    if settings.crash_reports_enabled {
        let crashes_dir = data_dir(&app).join("crashes");
        ensure_dir(&crashes_dir).map_err(|e| e.to_string())?;
        let fname = format!("crash-{}.json", Utc::now().format("%Y-%m-%dT%H-%M-%SZ"));
        let fpath = crashes_dir.join(fname);
        fs::write(fpath, serde_json::to_vec_pretty(&entry).unwrap()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Shortcut for JS errors.
#[tauri::command]
pub fn bd_logs_record_js_error(
    app: AppHandle,
    payload: ErrorPayload,
    app_version: String,
) -> Result<(), String> {
    bd_logs_record_error(app, payload, "js".to_string(), app_version)
}

/// Shortcut for native errors.
#[tauri::command]
pub fn bd_logs_record_native_error(
    app: AppHandle,
    payload: ErrorPayload,
    app_version: String,
) -> Result<(), String> {
    bd_logs_record_error(app, payload, "native".to_string(), app_version)
}

/// Append one analytics record (if analytics are enabled).
#[tauri::command]
pub fn bd_logs_new_record(
    app: AppHandle,
    record_type: String,
    payload: AnalyticsRecord,
    env: String,
    app_version: String,
) -> Result<(), String> {
    let settings = read_privacy(&app);
    if !settings.analytics_enabled {
        return Ok(());
    }

    let log_path = data_dir(&app).join(format!(
        "logs/record-{}.jsonl",
        Utc::now().format("%Y-%m")
    ));
    let log_data = new_log_data(
        &app,
        Payload::Analytics(payload),
        record_type,
        Some(env),
        Some(app_version),
    );
    let entry = serde_json::to_value(&log_data).unwrap();
    append_jsonl(log_path, &entry).map_err(|e| e.to_string())
}

/// Export logs/ and crashes/ as a zip file at `target_zip_path`.
#[tauri::command]
pub fn bd_logs_export_zip(app: AppHandle, target_zip_path: String) -> Result<(), String> {
    let base = data_dir(&app);
    let file = std::fs::File::create(&target_zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let dirs = ["logs", "crashes"];
    for d in dirs {
        let dir_path = base.join(d);
        if !dir_path.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir_path).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(&base)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                zip.start_file(rel, options).map_err(|e| e.to_string())?;
                let bytes = fs::read(entry.path()).map_err(|e| e.to_string())?;
                zip.write_all(&bytes).map_err(|e| e.to_string())?;
            }
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Install a Rust panic hook that writes JSON and text crash files.
pub fn install_panic_hook(app: AppHandle, app_version: String) {
    std::panic::set_hook(Box::new(move |info| {
        let ts = Utc::now().format("%Y-%m-%dT%H-%M-%SZ").to_string();
        let base = app.path().app_data_dir().expect("app_data_dir not available");
        let crashes = base.join("crashes");
        let _ = fs::create_dir_all(&crashes);
        let cause = info.to_string();
        let bt = std::backtrace::Backtrace::force_capture().to_string();

        let json = serde_json::json!({
          "record_type": "panic",
          "timestamp": Utc::now().timestamp_millis() as u64,
          "appVersion": app_version,
          "cause": cause,
          "backtrace": bt
        });
        let _ = fs::write(
            crashes.join(format!("crash-{}.json", ts)),
            serde_json::to_vec_pretty(&json).unwrap(),
        );
        let _ = fs::write(
            crashes.join(format!("crash-{}.log", ts)),
            format!("{}\n\n{}", cause, bt),
        );
    }));
}

// ==============================
// Retention
// ==============================

fn purge_older_than(dir: &Path, max_age_days: u32) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let now = std::time::SystemTime::now();
    let cutoff = now - Duration::from_secs((max_age_days as u64) * 24 * 3600);
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let meta = entry.metadata()?;
            if let Ok(modified) = meta.modified() {
                if modified < cutoff {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    Ok(())
}

/// Runs retention for logs/, analytics/, crashes/ according to current settings.
fn run_retention(app: &AppHandle) -> Result<(), String> {
    let s = read_privacy(app);
    let base = data_dir(app);
    purge_older_than(&base.join("logs"), s.retention_days_logs).map_err(|e| e.to_string())?;
    // opzionale (se non usi più analytics/ puoi rimuoverla)
    purge_older_than(&base.join("analytics"), s.retention_days_analytics).map_err(|e| e.to_string())?;
    purge_older_than(&base.join("crashes"), s.retention_days_crashes).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn bd_logs_run_retention(app: AppHandle) -> Result<(), String> {
    run_retention(&app)
}

// ==============================
// File listing/reading (for web app)
// ==============================

#[tauri::command]
pub fn bd_logs_list_files(app: AppHandle, area: String) -> Result<Vec<ListedFile>, String> {
    // area: "logs" | "crashes"
    let base = data_dir(&app);
    let allowed = ["logs", "crashes"];
    if !allowed.contains(&area.as_str()) {
        return Err("Invalid area".into());
    }
    let dir = base.join(&area);
    let mut out = vec![];
    if dir.exists() {
        for entry in WalkDir::new(&dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let meta = entry.metadata().map_err(|e| e.to_string())?;
                let m = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let modified_ms = m
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let rel = entry
                    .path()
                    .strip_prefix(&base)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                out.push(ListedFile {
                    rel_path: rel,
                    bytes: meta.len(),
                    modified_ms,
                });
            }
        }
    }
    out.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    Ok(out)
}

#[tauri::command]
pub fn bd_logs_read_file(
    app: AppHandle,
    rel_path: String,
    max_bytes: Option<u64>,
) -> Result<Vec<u8>, String> {
    // Constrain reads to app data dir.
    let base = data_dir(&app);
    let p = base.join(&rel_path);
    let canon_base = base.canonicalize().map_err(|e| e.to_string())?;
    let canon_file = p.canonicalize().map_err(|e| e.to_string())?;
    if !canon_file.starts_with(&canon_base) {
        return Err("Path escapes data directory".into());
    }
    let data = fs::read(&canon_file).map_err(|e| e.to_string())?;
    if let Some(limit) = max_bytes {
        if data.len() as u64 > limit {
            return Err(format!("File too large ({} > {})", data.len(), limit));
        }
    }
    Ok(data)
}

// ==============================
// Privacy toggle commands
// ==============================

#[tauri::command]
pub fn bd_logs_get_privacy(app: AppHandle) -> Result<PrivacySettings, String> {
    Ok(read_privacy(&app))
}

#[tauri::command]
pub fn bd_logs_set_privacy(
    app: AppHandle,
    analytics_enabled: Option<bool>,
    crash_reports_enabled: Option<bool>,
    retention_days_logs: Option<u32>,
    retention_days_analytics: Option<u32>,
    retention_days_crashes: Option<u32>,
) -> Result<PrivacySettings, String> {
    let mut s = read_privacy(&app);
    if let Some(v) = analytics_enabled {
        s.analytics_enabled = v;
    }
    if let Some(v) = crash_reports_enabled {
        s.crash_reports_enabled = v;
    }
    if let Some(v) = retention_days_logs {
        s.retention_days_logs = v.max(30); // minimum safety
    }
    if let Some(v) = retention_days_analytics {
        s.retention_days_analytics = v.max(30);
    }
    if let Some(v) = retention_days_crashes {
        s.retention_days_crashes = v.max(90); // crashes often valuable longer
    }
    write_privacy(&app, &s)?;
    Ok(s)
}

// ==============================
// Test commands (dev only)
// ==============================

#[cfg(debug_assertions)]
#[tauri::command]
pub fn bd_logs_test_record_n(app: AppHandle, n: u32) -> Result<(), String> {
    for i in 0..n {
        let rec = AnalyticsRecord {
            name: "test_event".into(),
            props: serde_json::json!({ "i": i, "blob": "x".repeat((i % 5 + 1) as usize * 2000) }),
        };
        bd_logs_new_record(
            app.clone(),
            "test".into(),
            rec,
            "dev".into(),
            "0.0.0-dev".into(),
        )?;
    }
    Ok(())
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn bd_logs_test_panic() {
    panic!("Intentional panic for testing crash pipeline");
}

#[cfg(debug_assertions)]
#[tauri::command]
pub fn bd_logs_test_force_retention(app: AppHandle, area: String) -> Result<(), String> {
    // Purge TUTTO nell’area indicata (logs|analytics|crashes). Utile per verificare la retention.
    let base = data_dir(&app);
    let allowed = ["logs", "analytics", "crashes"];
    if !allowed.contains(&area.as_str()) {
        return Err("Invalid area".into());
    }
    let dir = base.join(area);
    if dir.exists() {
        for entry in WalkDir::new(&dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}

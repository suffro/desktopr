use std::fs;
use std::path::PathBuf;
use std::thread;

use uuid::Uuid;

#[cfg(debug_assertions)]
use std::process::Command as DevCommand;

#[cfg(not(debug_assertions))]
use std::process::{Command, Stdio};

use tauri::{Emitter, Manager};

const BUBBLEDESK_COMPANION_DEV_BIN: &str = "bubbledesk-companion";

fn log_debug(app: &tauri::AppHandle, msg: &str) {
    println!("[Bubbledesk][DEBUG] {msg}");
    if let Err(e) = app.emit("bubbledesk:debug", msg.to_string()) {
        println!("[Bubbledesk][DEBUG] Failed to emit debug event: {e}");
    }
}

fn resolve_companion_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to resolve resource_dir: {e}"))?;

    // Try common layouts: Resources/resources/companion and Resources/companion
    let candidates = [
        resource_dir.join("resources").join("companion"),
        resource_dir.join("companion"),
    ];

    let companion_dir = candidates
        .into_iter()
        .find(|p| p.exists())
        .ok_or_else(|| {
            format!(
                "Companion directory not found under resource_dir {:?}",
                resource_dir
            )
        })?;

    #[cfg(target_os = "windows")]
    let companion_path = companion_dir.join("bubbledesk-companion.exe");

    #[cfg(not(target_os = "windows"))]
    let companion_path = companion_dir.join("bubbledesk-companion");

    if !companion_path.exists() {
        return Err(format!("Companion binary not found at {:?}", companion_path));
    }

    Ok(companion_path)
}

/// Launch the Bubbledesk companion application.
///
/// - In **dev builds** (debug assertions enabled), this spawns `cargo tauri dev -- --bin bubbledesk-companion`.
///   This assumes the developer has the full Rust toolchain and project layout available.
///
/// - In **release builds**, this launches a bundled sidecar binary named `bubbledesk-companion`,
///   which must be configured via `tauri.conf.json > tauri.bundle.externalBin`.
///
/// In both cases, a per-session sandbox directory is created (under `/tmp` on Unix or the
/// system temp dir elsewhere). The given `app_config` is written as
/// `tauri.conf.companion.json` inside that sandbox, and the companion receives the path via
/// the `COMPANION_APP_CONFIG_PATH` environment variable, plus a `BUBBLEDESK_COMPANION_SESSION_ID`.
#[tauri::command]
pub async fn bd_launch_companion(app: tauri::AppHandle, app_config: serde_json::Value) -> Result<(), String> {
    // Derive the package name at compile time so we can namespace the sandbox per app.
    let package_name: &str = env!("CARGO_PKG_NAME");

    // Create sandbox path: /tmp/<package_name>/companions/<session_id>
    let session_id = Uuid::new_v4().to_string();

    #[cfg(unix)]
    let mut sandbox_path = PathBuf::from("/tmp");
    #[cfg(not(unix))]
    let mut sandbox_path = std::env::temp_dir();

    sandbox_path.push(package_name);
    sandbox_path.push("companions");
    sandbox_path.push(&session_id);

    log_debug(&app, &format!("Companion sandbox will be created at: {:?}", sandbox_path));

    if let Err(e) = fs::create_dir_all(&sandbox_path) {
        log_debug(&app, &format!("[ERROR] Failed to create companion sandbox {:?}: {}", sandbox_path, e));
        return Err(format!("Failed to create companion sandbox: {e}"));
    }
    log_debug(&app, &format!("Sandbox created successfully: {:?}", sandbox_path));

    // Persist app config into the sandbox so the companion can read it
    let app_config_path = sandbox_path.join("tauri.conf.companion.json");
    let cfg_json = serde_json::to_string_pretty(&app_config)
        .map_err(|e| format!("Failed to serialize companion config: {e}"))?;

    log_debug(&app, &format!("Writing companion config to {:?}", app_config_path));

    if let Err(e) = fs::write(&app_config_path, cfg_json) {
        log_debug(&app, &format!("[ERROR] Failed to write companion config at {:?}: {}", app_config_path, e));
        return Err(format!("Failed to write companion config: {e}"));
    }
    log_debug(&app, "Companion config written successfully.");

    log_debug(&app, &format!(
        "Launching companion session {} in sandbox {:?}",
        session_id, sandbox_path
    ));

    // Development path: use `cargo tauri dev -- --bin bubbledesk-companion`.
    #[cfg(debug_assertions)]
    {
        log_debug(
            &app,
            &format!(
                "Dev companion command: cargo tauri dev -- --bin {}",
                BUBBLEDESK_COMPANION_DEV_BIN
            ),
        );

        log_debug(&app, "Spawning dev companion via Cargo...");

        let mut child = DevCommand::new("cargo")
            .args(&["tauri", "dev", "--", "--bin", BUBBLEDESK_COMPANION_DEV_BIN])
            .env(
                "COMPANION_APP_CONFIG_PATH",
                app_config_path.to_str().unwrap_or_default(),
            )
            .env("BUBBLEDESK_COMPANION_SESSION_ID", &session_id)
            .current_dir(".")
            .spawn()
            .map_err(|e| format!("Failed to spawn dev companion: {e}"))?;

        log_debug(&app, &format!("Dev companion spawned successfully with PID: {:?}", child.id()));

        let sandbox_for_cleanup = sandbox_path.clone();
        thread::spawn(move || {
            if let Ok(status) = child.wait() {
                println!(
                    "[Bubbledesk] Companion session {} exited with status {:?}",
                    session_id, status
                );
            }
            if let Err(e) = fs::remove_dir_all(&sandbox_for_cleanup) {
                eprintln!(
                    "[Bubbledesk] Failed to remove companion sandbox {:?}: {}",
                    sandbox_for_cleanup, e
                );
            }
        });
    }

    // Release path: launch a bundled companion binary from the resources directory.
    #[cfg(not(debug_assertions))]
    {
        log_debug(
            &app,
            &format!(
                "Launching companion (manual) in release mode, session {} in sandbox {:?}",
                session_id, sandbox_path
            ),
        );

        let companion_path = resolve_companion_path(&app)?;

        log_debug(
            &app,
            &format!("Resolved companion path: {:?}", companion_path),
        );

        let sandbox_for_cleanup = sandbox_path.clone();

        let mut child = Command::new(&companion_path)
            .env(
                "COMPANION_APP_CONFIG_PATH",
                app_config_path.to_str().unwrap_or_default(),
            )
            .env("BUBBLEDESK_COMPANION_SESSION_ID", &session_id)
            .current_dir(&sandbox_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn companion process: {e}"))?;

        log_debug(
            &app,
            &format!("Companion process spawned with PID: {:?}", child.id()),
        );

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let app_for_events = app.clone();
            thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("[bubbledesk-companion stdout] {}", line);
                        let _ = app_for_events.emit(
                            "bubbledesk:debug",
                            format!("companion stdout: {}", line),
                        );
                    }
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = child.stderr.take() {
            let app_for_events = app.clone();
            thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        eprintln!("[bubbledesk-companion stderr] {}", line);
                        let _ = app_for_events.emit(
                            "bubbledesk:debug",
                            format!("companion stderr: {}", line),
                        );
                    }
                }
            });
        }

        // Wait for process and cleanup sandbox
        thread::spawn(move || {
            if let Ok(status) = child.wait() {
                println!(
                    "[Bubbledesk] Companion session {} exited with status {:?}",
                    session_id, status
                );
            }
            if let Err(e) = fs::remove_dir_all(&sandbox_for_cleanup) {
                eprintln!(
                    "[Bubbledesk] Failed to remove companion sandbox {:?}: {}",
                    sandbox_for_cleanup, e
                );
            }
        });
    }

    Ok(())
}
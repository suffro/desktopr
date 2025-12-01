use std::fs;
use std::path::PathBuf;
use std::thread;

use uuid::Uuid;

#[cfg(debug_assertions)]
use std::process::Command as DevCommand;

#[cfg(not(debug_assertions))]
use tauri_plugin_shell::ShellExt;
#[cfg(not(debug_assertions))]
use tauri_plugin_shell::process::CommandEvent;

use tauri::{Emitter, Manager};

const BUBBLEDESK_SIDECAR_DEV_BIN: &str = "bubbledesk-companion";
const BUBBLEDESK_SIDECAR_NAME: &str = "binaries/bubbledesk-companion";

fn log_debug(app: &tauri::AppHandle, msg: &str) {
    println!("[Bubbledesk][DEBUG] {msg}");
    if let Err(e) = app.emit("bubbledesk:debug", msg.to_string()) {
        println!("[Bubbledesk][DEBUG] Failed to emit debug event: {e}");
    }
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
                BUBBLEDESK_SIDECAR_DEV_BIN
            ),
        );

        log_debug(&app, "Spawning dev companion via Cargo...");

        let mut child = DevCommand::new("cargo")
            .args(&["tauri", "dev", "--", "--bin", BUBBLEDESK_SIDECAR_DEV_BIN])
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

    // Release path: use a bundled sidecar named `bubbledesk-companion`.
    #[cfg(not(debug_assertions))]
    {
        log_debug(
            &app,
            &format!("Launching companion sidecar '{}' in release mode", BUBBLEDESK_SIDECAR_NAME),
        );

        let sandbox_for_cleanup = sandbox_path.clone();

        let sidecar_cmd = match app.shell().sidecar(BUBBLEDESK_SIDECAR_NAME) {
            Ok(cmd) => cmd,
            Err(e) => {
                log_debug(
                    &app,
                    &format!(
                        "[ERROR] Failed to create companion sidecar '{}': {e}",
                        BUBBLEDESK_SIDECAR_NAME
                    ),
                );
                return Err(format!(
                    "Failed to create companion sidecar '{}': {e}",
                    BUBBLEDESK_SIDECAR_NAME
                ));
            }
        }
        .env(
            "COMPANION_APP_CONFIG_PATH",
            app_config_path.to_str().unwrap_or_default(),
        )
        .env("BUBBLEDESK_COMPANION_SESSION_ID", &session_id);

        log_debug(
            &app,
            &format!("Attempting to spawn bundled sidecar '{}'.", BUBBLEDESK_SIDECAR_NAME),
        );

        let (mut rx, mut child) = sidecar_cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn companion sidecar '{}': {e}", BUBBLEDESK_SIDECAR_NAME))?;

        log_debug(
            &app,
            &format!("Sidecar spawned successfully with PID: {:?}", child.pid()),
        );

        let app_for_events = app.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        let line_str = String::from_utf8_lossy(&line).to_string();
                        println!("[bubbledesk-companion stdout] {}", line_str);
                        let _ = app_for_events.emit(
                            "bubbledesk:debug",
                            format!("sidecar stdout: {}", line_str),
                        );
                    }
                    CommandEvent::Stderr(line) => {
                        let line_str = String::from_utf8_lossy(&line).to_string();
                        eprintln!("[bubbledesk-companion stderr] {}", line_str);
                        let _ = app_for_events.emit(
                            "bubbledesk:debug",
                            format!("sidecar stderr: {}", line_str),
                        );
                    }
                    CommandEvent::Terminated(status) => {
                        println!("[bubbledesk-companion terminated] status: {:?}", status);
                        let _ = app_for_events.emit(
                            "bubbledesk:debug",
                            format!("sidecar terminated with status: {:?}", status),
                        );
                    }
                    other => {
                        println!("[bubbledesk-companion event] {:?}", other);
                        let _ = app_for_events.emit(
                            "bubbledesk:debug",
                            format!("sidecar event: {:?}", other),
                        );
                    }
                }
            }
            let _ = fs::remove_dir_all(&sandbox_for_cleanup);
        });
    }

    Ok(())
}
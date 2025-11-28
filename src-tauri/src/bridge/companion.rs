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

const BUBBLEDESK_SIDECAR_NAME: &str = "binaries/bubbledesk-sidecar";

/// Launch the Bubbledesk companion application.
///
/// - In **dev builds** (debug assertions enabled), this spawns `cargo tauri dev -- --bin bubbledesk-sidecar`.
///   This assumes the developer has the full Rust toolchain and project layout available.
///
/// - In **release builds**, this launches a bundled sidecar binary named `bubbledesk-sidecar`,
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

    fs::create_dir_all(&sandbox_path)
        .map_err(|e| format!("Failed to create companion sandbox: {e}"))?;

    // Persist app config into the sandbox so the companion can read it
    let app_config_path = sandbox_path.join("tauri.conf.companion.json");
    let cfg_json = serde_json::to_string_pretty(&app_config)
        .map_err(|e| format!("Failed to serialize companion config: {e}"))?;
    fs::write(&app_config_path, cfg_json)
        .map_err(|e| format!("Failed to write companion config: {e}"))?;

    println!(
        "[Bubbledesk] Launching companion session {} in sandbox {:?}",
        session_id, sandbox_path
    );

    // Development path: use `cargo tauri dev -- --bin bubbledesk-sidecar`.
    #[cfg(debug_assertions)]
    {
        println!(
            "[Bubbledesk] Dev companion command: cargo tauri dev -- --bin {}",
            BUBBLEDESK_SIDECAR_NAME
        );

        let mut child = DevCommand::new("cargo")
            .args(&["tauri", "dev", "--", "--bin", BUBBLEDESK_SIDECAR_NAME])
            .env(
                "COMPANION_APP_CONFIG_PATH",
                app_config_path.to_str().unwrap_or_default(),
            )
            .env("BUBBLEDESK_COMPANION_SESSION_ID", &session_id)
            .current_dir(".")
            .spawn()
            .map_err(|e| format!("Failed to spawn dev companion: {e}"))?;

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

    // Release path: use a bundled sidecar named `bubbledesk-sidecar`.
    #[cfg(not(debug_assertions))]
    {
        println!(
            "[Bubbledesk] Launching companion sidecar '{}' in release mode",
            BUBBLEDESK_SIDECAR_NAME
        );

        let sandbox_for_cleanup = sandbox_path.clone();

        let sidecar_cmd = app
            .shell()
            .sidecar(BUBBLEDESK_SIDECAR_NAME)
            .map_err(|e| format!("Failed to create companion sidecar '{}': {e}", BUBBLEDESK_SIDECAR_NAME))?
            .env(
                "COMPANION_APP_CONFIG_PATH",
                app_config_path.to_str().unwrap_or_default(),
            )
            .env("BUBBLEDESK_COMPANION_SESSION_ID", &session_id);

        let (mut rx, mut child) = sidecar_cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn companion sidecar '{}': {e}", BUBBLEDESK_SIDECAR_NAME))?;

        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let CommandEvent::Stdout(line) = event {
                    println!("[bubbledesk-sidecar stdout] {}", String::from_utf8_lossy(&line));
                }
            }
            let _ = fs::remove_dir_all(&sandbox_for_cleanup);
        });
    }

    Ok(())
}
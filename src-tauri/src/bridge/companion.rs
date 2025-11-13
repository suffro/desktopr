use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

use toml::Value;
use uuid::Uuid;

#[tauri::command]
pub fn bd_launch_companion(app_config: serde_json::Value) -> Result<(), String> {
    // Read package name from Cargo.toml to derive the companion bin name
    let cargo_toml_content = fs::read_to_string("Cargo.toml").map_err(|e| e.to_string())?;
    let cargo_toml: Value = toml::from_str(&cargo_toml_content).map_err(|e| e.to_string())?;
    let package_name = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or("Failed to read package name from Cargo.toml")?;

    let companion_bin_name = format!("{}-companion", package_name);

    // Create sandbox path: /tmp/<package_name>/companions/<session_id>
    let session_id = Uuid::new_v4().to_string();

    #[cfg(unix)]
    let mut sandbox_path = PathBuf::from("/tmp");
    #[cfg(not(unix))]
    let mut sandbox_path = std::env::temp_dir();

    sandbox_path.push(package_name);
    sandbox_path.push("companions");
    sandbox_path.push(&session_id);

    fs::create_dir_all(&sandbox_path).map_err(|e| e.to_string())?;

    // Persist app config into the sandbox so the companion can read it
    let app_config_path = sandbox_path.join("tauri.conf.companion.json");
    let cfg_json = serde_json::to_string_pretty(&app_config).map_err(|e| e.to_string())?;
    fs::write(&app_config_path, cfg_json).map_err(|e| e.to_string())?;

    println!(
        "[Bubbledesk] Launching companion session {} in sandbox {:?}",
        session_id, sandbox_path
    );
    println!(
        "[Bubbledesk] Command: cargo tauri dev -- --bin {}",
        companion_bin_name
    );

    // Spawn the companion process in dev mode, passing the app config path
    let mut child = Command::new("cargo")
        .args(&["tauri", "dev", "--", "--bin", &companion_bin_name])
        .env(
            "COMPANION_APP_CONFIG_PATH",
            app_config_path.to_str().unwrap_or_default(),
        )
        .env("BUBBLEDESK_COMPANION_SESSION_ID", &session_id)
        .current_dir(".")
        .spawn()
        .map_err(|e| e.to_string())?;

    // Cleanup thread: wait for the child to exit, then remove the sandbox directory
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

    Ok(())
}
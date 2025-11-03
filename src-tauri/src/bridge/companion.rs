use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Child};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use uuid::Uuid;
use toml::Value;
use std::fs::File;

#[tauri::command]
pub fn bd_launch_companion(app_config: serde_json::Value, menu_config: serde_json::Value) -> Result<(), String> {
    // 1. Generate unique session_id
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?;
    let timestamp = since_the_epoch.as_secs();
    let uuid = Uuid::new_v4();
    let session_id = format!("{}_{}", timestamp, uuid);

    // 4. Read package name from Cargo.toml
    let cargo_toml_content = fs::read_to_string("Cargo.toml").map_err(|e| e.to_string())?;
    let cargo_toml: Value = cargo_toml_content.parse::<Value>().map_err(|e| e.to_string())?;
    let package_name = cargo_toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or("Failed to read package name from Cargo.toml")?;

    // 2. Create sandbox directory at /tmp/<package_name>/companions/<session_id>
    let sandbox_path = PathBuf::from(format!("/tmp/{}/companions/{}", package_name, session_id));
    fs::create_dir_all(&sandbox_path).map_err(|e| e.to_string())?;

    // 3. Write JSON files
    let app_config_path = sandbox_path.join("tauri.conf.companion.json");
    let menu_config_path = sandbox_path.join("menu.config.companion.json");

    {
        let mut file = File::create(&app_config_path).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(&app_config).map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    }
    if menu_config.is_null() || (menu_config.is_object() && menu_config.as_object().unwrap().is_empty()) {
        println!("Menu config is null or empty; using native system menu.");
    } else {
        let mut file = File::create(&menu_config_path).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(&menu_config).map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    }

    let companion_bin_name = format!("{}-companion", package_name);

    // 5. Launch process
    let mut child = Command::new("cargo")
        .args(&["tauri", "dev", "--bin", &companion_bin_name])
        .env("TAURI_CONFIG", app_config_path.to_str().unwrap())
        .env("MENU_CONFIG_PATH", menu_config_path.to_str().unwrap())
        .current_dir(".")
        .spawn()
        .map_err(|e| e.to_string())?;

    // 6. Create thread to wait for process termination and cleanup
    let sandbox_path_clone = sandbox_path.clone();
    thread::spawn(move || {
        let _ = child.wait();
        if let Err(e) = fs::remove_dir_all(&sandbox_path_clone) {
            eprintln!("Failed to remove sandbox directory {}: {}", sandbox_path_clone.display(), e);
        } else {
            println!("Sandbox directory {} cleaned up after process exit.", sandbox_path_clone.display());
        }
    });

    // 7. Log sandbox path and session id
    println!("Companion sandbox created at: {} with session ID: {}", sandbox_path.display(), session_id);

    Ok(())
}
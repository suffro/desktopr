// All comments are in English.

use serde::{Deserialize, Serialize};
use tauri::{App, AppHandle, Manager};

use crate::bridge::fs::{bd_fs_read_text, bd_fs_write_text};

// ✅ Correct trait for v2 to access autostart manager via `app.autolaunch()`
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum AutostartMode {
  Shown,
  Minimized,
  Hidden,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AutostartSettings {
  autostart_mode: AutostartMode,
}

/// Persisted under the `data/` scope using your fs.rs helpers.
const SETTINGS_REL_PATH: &str = "settings/autostart.json";

fn load_settings(app: &AppHandle) -> AutostartSettings {
  match bd_fs_read_text(app.clone(), SETTINGS_REL_PATH.to_string(), Some(true)) {
    Ok(text) => serde_json::from_str::<AutostartSettings>(&text)
      .unwrap_or(AutostartSettings { autostart_mode: AutostartMode::Shown }),
    Err(_) => AutostartSettings { autostart_mode: AutostartMode::Shown },
  }
}

fn save_settings(app: &AppHandle, s: &AutostartSettings) -> Result<(), String> {
  let json = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
  // create_dirs = true, append = false
  bd_fs_write_text(
    app.clone(),
    SETTINGS_REL_PATH.to_string(),
    Some(true),
    json,
    Some(true),
    Some(false),
  )
}

/// Apply how the main window should appear when launched by the OS.
fn apply_autostart_window_behavior(app: &AppHandle, mode: AutostartMode) {
  // ✅ v2 uses `get_webview_window`
  if let Some(win) = app.get_webview_window("main") {
    match mode {
      AutostartMode::Shown => {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
      }
      AutostartMode::Minimized => {
        // Show then minimize is the most robust sequence across DEs/OSes.
        let _ = win.show();
        let _ = win.minimize();
      }
      AutostartMode::Hidden => {
        // Keep running (e.g., with tray) without showing the main window.
        let _ = win.hide();
      }
    }
  }
}

/// Register the plugin with a constant CLI flag used to detect OS autostart.
pub fn init_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
  tauri_plugin_autostart::init(
    MacosLauncher::LaunchAgent,
    Some(vec!["--autostart".into()]), // any flags you want at OS launch
  )
}

/// Setup hook: return the signature Tauri expects (Box<dyn Error>).
pub fn setup() -> impl Fn(&mut App) -> Result<(), Box<dyn std::error::Error>> {
  |app| {
    let launched_by_os = std::env::args().any(|a| a == "--autostart");
    if launched_by_os {
      let s = load_settings(&app.handle());
      apply_autostart_window_behavior(&app.handle(), s.autostart_mode);
    }
    Ok(())
  }
}

// ---------- Commands (Rust only; your TS bridge uses `invoke`) ----------

#[tauri::command]
pub fn bd_autostart_enable(app: AppHandle) -> Result<(), String> {
  // ✅ v2: use the autostart manager from the extension trait
  app.autolaunch().enable().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_autostart_disable(app: AppHandle) -> Result<(), String> {
  app.autolaunch().disable().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_autostart_status(app: AppHandle) -> Result<bool, String> {
  app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_get_autostart_mode(app: AppHandle) -> Result<String, String> {
  let s = load_settings(&app);
  Ok(match s.autostart_mode {
    AutostartMode::Shown => "shown",
    AutostartMode::Minimized => "minimized",
    AutostartMode::Hidden => "hidden",
  }.to_string())
}

#[tauri::command]
pub fn bd_set_autostart_mode(app: AppHandle, mode: String) -> Result<(), String> {
  let mut s = load_settings(&app);
  s.autostart_mode = match mode.as_str() {
    "shown" => AutostartMode::Shown,
    "minimized" => AutostartMode::Minimized,
    "hidden" => AutostartMode::Hidden,
    _ => return Err("Invalid autostart mode".into()),
  };
  save_settings(&app, &s)
}
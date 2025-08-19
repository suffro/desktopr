use tauri::WebviewWindow;

#[tauri::command]
pub fn bd_win_minimize(window: WebviewWindow) -> Result<(), String> {
  window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_win_maximize(window: WebviewWindow) -> Result<(), String> {
  if window.is_maximized().unwrap_or(false) {
    window.unmaximize().map_err(|e| e.to_string())
  } else {
    window.maximize().map_err(|e| e.to_string())
  }
}

#[tauri::command]
pub fn bd_win_fullscreen(window: WebviewWindow, enable: bool) -> Result<(), String> {
  window.set_fullscreen(enable).map_err(|e| e.to_string())
}

use tauri::{AppHandle, Manager, WebviewWindow};

fn main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
  app.get_webview_window("main").ok_or_else(|| "main window not found".into())
}

#[tauri::command]
pub fn bd_win_minimize(app: AppHandle) -> Result<(), String> {
  main_window(&app)?.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_win_maximize(app: AppHandle) -> Result<(), String> {
  let w = main_window(&app)?;
  if w.is_maximized().unwrap_or(false) { w.unmaximize().map_err(|e| e.to_string()) }
  else { w.maximize().map_err(|e| e.to_string()) }
}

#[tauri::command]
pub fn bd_win_fullscreen(app: AppHandle, enable: bool) -> Result<(), String> {
  main_window(&app)?.set_fullscreen(enable).map_err(|e| e.to_string())
}

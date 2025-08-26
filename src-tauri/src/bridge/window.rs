use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WebviewWindow};
use url::Url;
use crate::helpers::constants::BRIDGE_JS;
use crate::helpers::window_defaults::read_main_defaults;

#[tauri::command]
pub fn bd_win_minimize(app: AppHandle, label: String) -> Result<(), String> {
  if let Some(window) = app.get_webview_window(&label) {
    window.minimize().map_err(|e| e.to_string())
  } else {
      Err(format!("Window '{}' not found", label))
  }
}

#[tauri::command]
pub fn bd_win_maximize(app: AppHandle, label: String) -> Result<(), String> {
  if let Some(window) = app.get_webview_window(&label) {
    if window.is_maximized().unwrap_or(false) {
      window.unmaximize().map_err(|e| e.to_string())
    } else {
      window.maximize().map_err(|e| e.to_string())
    }
  } else {
      Err(format!("Window '{}' not found", label))
  }
}

#[tauri::command]
pub fn bd_win_fullscreen(app: AppHandle, label: String, enable: bool) -> Result<(), String> {
  if let Some(window) = app.get_webview_window(&label) {
    window.set_fullscreen(enable).map_err(|e| e.to_string())
  } else {
      Err(format!("Window '{}' not found", label))
  }
}

#[tauri::command]
pub fn bd_win_open(
  app: AppHandle,
  label: String,
  url: Option<String>,
  _width: f64,
  _height: f64,
) -> Result<(), String> {
  if app.get_webview_window(&label).is_some() {
    return Ok(());
  }

  let u = url.ok_or("Missing URL")?;
  let parsed = u.parse::<url::Url>().map_err(|e| e.to_string())?;

  let d = read_main_defaults(&app);

  let mut b = WebviewWindowBuilder::new(&app, label, WebviewUrl::External(parsed))
    .title(d.title)
    .inner_size(d.width, d.height)
    .resizable(d.resizable)
    .decorations(d.decorations)
    .fullscreen(d.fullscreen)
    .visible(true)
    .initialization_script(include_str!("../../tsc/bridge.js"));

  if d.always_on_top {
    b = b.always_on_top(true);
  }

  b.build().map_err(|e| e.to_string())?;
  Ok(())
}

#[tauri::command]
pub fn bd_win_close(app: AppHandle, label: String) -> Result<(), String> {
  if let Some(w) = app.get_webview_window(&label) { w.close().map_err(|e| e.to_string())?; }
  Ok(())
}

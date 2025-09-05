use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WebviewWindow};
use url::Url;

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
pub async fn bd_win_open(
  app: AppHandle,
  label: String,
  fullscreen: bool,
  url: String,
) -> Result<(), String> {
  if app.get_webview_window(&label).is_some() {
    return Ok(());
  }
  let s = url.to_string();

  let mut conf = app.config().app.windows.iter().find(|c| c.label == "main").unwrap().clone();
  // This should be a unique label for all windows. For example, we can use a random suffix:
  let mut buf = [0u8; 1];
  assert_eq!(getrandom::fill(&mut buf), Ok(()));
  let suffix = buf[0];
  conf.label = format!("{}-{}", label, suffix);
  conf.fullscreen = fullscreen;
  if !s.is_empty(){
    let webview_url = WebviewUrl::External(
        s.parse::<Url>().map_err(|e| e.to_string())?
    );
    conf.url = webview_url;
  }
  let webview_window = tauri::WebviewWindowBuilder::from_config(&app, &conf)
    .unwrap()
    .build()
    .unwrap();

  Ok(())
}

#[tauri::command]
pub fn bd_win_close(app: AppHandle, label: String) -> Result<(), String> {
  if let Some(w) = app.get_webview_window(&label) { w.close().map_err(|e| e.to_string())?; }
  Ok(())
}

#[tauri::command]
pub fn bd_open_devtools(app: tauri::AppHandle, label: String) -> Result<(), String> {
  if let Some(win) = app.get_webview_window(&label) {
    win.open_devtools();
  }
  Ok(())
}

#[tauri::command]
pub fn bd_close_devtools(app: tauri::AppHandle, label: String) -> Result<(), String> {
  if let Some(win) = app.get_webview_window(&label) {
    win.close_devtools();
  }
  Ok(())
}

#[tauri::command]
pub fn bd_toggle_devtools(app: tauri::AppHandle, label: String) -> Result<(), String> {
  if let Some(win) = app.get_webview_window(&label) {
    if win.is_devtools_open() {
      bd_close_devtools(app, label);
    } else {
      bd_open_devtools(app, label);
    }
  }
  Ok(())
}
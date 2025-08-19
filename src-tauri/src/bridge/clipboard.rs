use tauri::AppHandle;
use tauri_plugin_clipboard::Clipboard;

#[tauri::command]
pub fn bd_clipboard_write(app: AppHandle, text: String) -> Result<(), String> {
  let mut cb = Clipboard::new(app);
  cb.write_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_clipboard_read(app: AppHandle) -> Result<String, String> {
  let mut cb = Clipboard::new(app);
  cb.read_text()
    .map_err(|e| e.to_string())
    .map(|opt| opt.unwrap_or_default())
}

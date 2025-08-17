// All comments in English
use tauri::{AppHandle};

#[tauri::command]
pub async fn bd_notify(title: String, body: String) -> Result<(), String> {
  // TODO: integrate real notifications (e.g., plugin) if needed
  println!("[Notify] {title} — {body}");
  Ok(())
}

#[tauri::command]
pub async fn bd_clipboard_write(text: String, app: AppHandle) -> Result<(), String> {
  // Minimal example via JS-side navigator.clipboard or plugin later; here just log
  println!("[Clipboard/write] {}", text);
  // You can switch to a real clipboard plugin/OS call in the next iteration
  Ok(())
}

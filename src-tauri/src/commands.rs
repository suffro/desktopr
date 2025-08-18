use tauri::{AppHandle};
use tauri_plugin_notification::NotificationExt;

// Check permission state
#[tauri::command]
pub async fn bd_notification_state(app: AppHandle) -> Result<String, String> {
  // "granted" | "denied" | "default" (dipende dalla piattaforma)
  let state = app.notification().permission_state()
    .map_err(|e| e.to_string())?;
  Ok(state.to_string())
}

// Request permission (mostra il prompt se necessario)
#[tauri::command]
pub fn bd_request_permission(app: AppHandle) -> Result<String, String> {
  let result = app.notification()
    .request_permission()
    .map_err(|e| e.to_string())?;
  Ok(result.to_string()) // "granted" | "denied" | "default"
}

// Show a system notification (assume permission OK)
#[tauri::command]
pub async fn bd_notify(app: AppHandle, title: String, body: String) -> Result<bool, String> {
  println!("[Notify] {title} — {body}");

  // 1. check stato
  let state = bd_notification_state(app.clone()).await?;
  if state != "granted" {
      // 2. prova a richiedere
      let req = bd_request_permission(app.clone())?;
      if req != "granted" {
          return Err(format!("Notification permission denied: {}", req));
      }
  }
  println!("[notification] permission {state}");

  // 3. invia
  app.notification()
      .builder()
      .title(title)
      .body(body)
      .show()
      .map_err(|e| e.to_string())?;


  Ok(true)
}

#[tauri::command]
pub async fn bd_clipboard_write(text: String, app: AppHandle) -> Result<(), String> {
  // Minimal example via JS-side navigator.clipboard or plugin later; here just log
  println!("[Clipboard/write] {}", text);
  // You can switch to a real clipboard plugin/OS call in the next iteration
  Ok(())
}

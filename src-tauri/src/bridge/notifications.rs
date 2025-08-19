use tauri::AppHandle;
use tauri_plugin_notification::{NotificationExt, PermissionState};
use serde::Serialize;

#[tauri::command]
pub fn bd_notification_state(app: AppHandle) -> Result<String, String> {
  let s = app.notification().permission_state().map_err(|e| e.to_string())?;
  Ok(s.to_string())
}

#[tauri::command]
pub fn bd_request_permission(app: AppHandle) -> Result<String, String> {
  let r = app.notification().request_permission().map_err(|e| e.to_string())?;
  Ok(r.to_string())
}

#[derive(Serialize)]
pub struct NotifyResult { pub shown: bool, pub state_before: String, pub state_after: String }

#[tauri::command]
pub fn bd_notify(app: AppHandle, title: String, body: String) -> Result<NotifyResult, String> {
  let before = app.notification().permission_state().map_err(|e| e.to_string())?;
  let mut after = before.clone();

  if before != PermissionState::Granted {
    after = app.notification().request_permission().map_err(|e| e.to_string())?;
    if after != PermissionState::Granted {
      return Ok(NotifyResult { shown: false, state_before: before.to_string(), state_after: after.to_string() });
    }
  }

  app.notification().builder().title(title).body(body).show().map_err(|e| e.to_string())?;
  Ok(NotifyResult { shown: true, state_before: before.to_string(), state_after: after.to_string() })
}

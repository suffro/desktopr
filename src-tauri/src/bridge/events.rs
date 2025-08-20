use serde_json::Value;
use tauri::AppHandle;

#[tauri::command]
pub fn bd_event_emit(app: AppHandle, event: String, payload: Option<Value>) -> Result<(), String> {
    app.emit_all(&event, payload.unwrap_or_default())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_event_emit_to(
    app: AppHandle,
    window: String,
    event: String,
    payload: Option<Value>,
) -> Result<(), String> {
    app.emit_to(&window, &event, payload.unwrap_or_default())
        .map_err(|e| e.to_string())
}

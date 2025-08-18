use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct OpenResult { pub paths: Vec<String> }

#[tauri::command]
pub async fn bd_file_open(app: AppHandle, multi: bool) -> Result<OpenResult, String> {
  let dlg = app.dialog().file().title("Select file(s)");
  let res = if multi { dlg.pick_files().await } else { dlg.pick_file().await.map(|p| p.map(|x| vec![x])) };
  let list = res.unwrap_or_default().unwrap_or_default();
  Ok(OpenResult { paths: list.into_iter().map(|p| p.to_string_lossy().to_string()).collect() })
}

#[tauri::command]
pub async fn bd_file_save(app: AppHandle, default_name: Option<String>) -> Result<String, String> {
  let mut dlg = app.dialog().file().title("Save As");
  if let Some(name) = default_name { dlg = dlg.set_file_name(&name); }
  let path = dlg.save_file().await.unwrap_or_default();
  Ok(path.map(|p| p.to_string_lossy().to_string()).unwrap_or_default())
}

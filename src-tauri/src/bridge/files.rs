use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct OpenResult { pub paths: Vec<String> }

#[tauri::command]
pub async fn bd_file_open(app: AppHandle, multi: bool) -> Result<OpenResult, String> {
  let builder = app.dialog().file().set_title("Select file(s)");
  let paths = if multi {
    builder.pick_files().await.unwrap_or_default().unwrap_or_default()
  } else {
    builder.pick_file().await.unwrap_or_default().map(|p| vec![p]).unwrap_or_default()
  };
  Ok(OpenResult { paths: paths.into_iter().map(|p| p.to_string_lossy().to_string()).collect() })
}

#[tauri::command]
pub async fn bd_file_save(app: AppHandle, default_name: Option<String>) -> Result<String, String> {
  let mut builder = app.dialog().file().set_title("Save As");
  if let Some(name) = default_name {
    builder = builder.set_file_name(&name);
  }
  let path = builder.save_file().await.unwrap_or_default();
  Ok(path.map(|p| p.to_string_lossy().to_string()).unwrap_or_default())
}

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct OpenResult { pub paths: Vec<String> }

#[tauri::command]
pub fn bd_file_open(app: AppHandle, multi: bool) -> Result<OpenResult, String> {
  let builder = app.dialog().file().set_title("Select file(s)");

  let paths: Vec<PathBuf> = if multi {
    // Option<Vec<PathBuf>>
    builder.blocking_pick_files().unwrap_or_default()
  } else {
    // Option<PathBuf> -> Vec<PathBuf>
    builder.blocking_pick_file().map(|p| vec![p]).unwrap_or_default()
  }.unwrap_or_default();

  Ok(OpenResult {
    paths: paths.into_iter().map(|p| p.to_string_lossy().to_string()).collect()
  })
}

#[tauri::command]
pub fn bd_file_save(app: AppHandle, default_name: Option<String>) -> Result<String, String> {
  let mut builder = app.dialog().file().set_title("Save As");
  if let Some(name) = default_name {
    builder = builder.set_file_name(&name);
  }
  // Option<PathBuf>
  let saved = builder.blocking_save_file().unwrap_or_default();
  Ok(saved.map(|p| p.to_string_lossy().to_string()).unwrap_or_default())
}

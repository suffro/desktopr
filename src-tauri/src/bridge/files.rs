use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, FilePath};
use serde::Serialize;

#[derive(Serialize)]
pub struct OpenResult { pub paths: Vec<String> }

fn file_path_to_string(p: FilePath) -> String {
  match p {
    FilePath::Path(pb) => pb.to_string_lossy().to_string(),
    FilePath::Url(u)   => u.to_string(),
  }
}

#[tauri::command]
pub async fn bd_file_open(app: AppHandle, multi: bool) -> Result<OpenResult, String> {
  // Cloniamo ciò che serve nel task bloccante
  let handle = app.clone();

  let paths = tauri::async_runtime::spawn_blocking(move || {
    let builder = handle.dialog().file().set_title("Select file(s)");

    if multi {
      // Option<Vec<FilePath>>
      match builder.blocking_pick_files() {
        Some(list) => list.into_iter().map(file_path_to_string).collect::<Vec<_>>(),
        None => Vec::new(),
      }
    } else {
      // Option<FilePath> -> Vec<String>
      match builder.blocking_pick_file() {
        Some(p) => vec![file_path_to_string(p)],
        None => Vec::new(),
      }
    }
  })
  .await
  .map_err(|e| format!("Join error: {e}"))?; // errore nel thread pool

  Ok(OpenResult { paths })
}

#[tauri::command]
pub async fn bd_file_save(app: AppHandle, default_name: Option<String>) -> Result<String, String> {
  let handle = app.clone();

  let saved = tauri::async_runtime::spawn_blocking(move || {
    let mut builder = handle.dialog().file().set_title("Save As");
    if let Some(name) = default_name.as_deref() {
      builder = builder.set_file_name(name);
    }
    // Option<FilePath> -> String (vuota se cancel)
    builder.blocking_save_file().map(file_path_to_string).unwrap_or_default()
  })
  .await
  .map_err(|e| format!("Join error: {e}"))?;

  Ok(saved)
}

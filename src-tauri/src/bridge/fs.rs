use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

/// Ritorna la directory base sicura (es. cache dell'app)
fn base_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// Risolve un percorso relativo alla base_dir, impedendo di uscire fuori.
fn resolve_safe(app: &AppHandle, rel: &str) -> Result<PathBuf, String> {
    let base = base_dir(app);
    let p = base.join(rel.trim_start_matches('/'));
    let canon = p.canonicalize().map_err(|e| e.to_string())?;
    let base_canon = base.canonicalize().map_err(|e| e.to_string())?;
    if !canon.starts_with(&base_canon) {
        return Err("Path fuori dallo scope consentito".into());
    }
    Ok(canon)
}

#[tauri::command]
pub fn fs_list_dir(app: AppHandle, rel: String) -> Result<Vec<FsEntry>, String> {
    let dir = resolve_safe(&app, &rel)?;
    let mut out = Vec::new();
    for e in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let md = e.metadata().map_err(|e| e.to_string())?;
        let is_dir = md.is_dir();
        let size = if md.is_file() { Some(md.len()) } else { None };
        let name = e.file_name().to_string_lossy().into_owned();
        let path_str = e.path().to_string_lossy().into_owned();
        out.push(FsEntry {
            name,
            path: path_str,
            is_dir,
            size,
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn fs_mkdir(app: AppHandle, rel: String) -> Result<(), String> {
    let dir = resolve_safe(&app, &rel)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn fs_rm(app: AppHandle, rel: String, recursive: bool) -> Result<(), String> {
    let p = resolve_safe(&app, &rel)?;
    if recursive {
        fs::remove_dir_all(&p).map_err(|e| e.to_string())
    } else {
        if p.is_dir() {
            fs::remove_dir(&p).map_err(|e| e.to_string())
        } else {
            fs::remove_file(&p).map_err(|e| e.to_string())
        }
    }
}

#[tauri::command]
pub fn fs_stat(app: AppHandle, rel: String) -> Result<FsEntry, String> {
    let p = resolve_safe(&app, &rel)?;
    let md = fs::metadata(&p).map_err(|e| e.to_string())?;
    Ok(FsEntry {
        name: p
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        path: p.to_string_lossy().into_owned(),
        is_dir: md.is_dir(),
        size: if md.is_file() { Some(md.len()) } else { None },
    })
}

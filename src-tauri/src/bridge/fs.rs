use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use tauri::{AppHandle, Manager};
use std::io::{Read, Write};
use base64::{engine::general_purpose, Engine as _}; // Cargo.toml: base64 = "0.22"
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

/// Restituisce la base dir in base allo scope.
/// - permanent=true  -> app_data_dir (persistente)
/// - permanent=false -> app_cache_dir (non persistente)
fn base_dir(app: &AppHandle, permanent: bool) -> PathBuf {
    if permanent {
        app.path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap())
    } else {
        app.path()
            .app_cache_dir()
            .unwrap_or_else(|_| std::env::temp_dir())
    }
}

fn ensure_base_exists(app: &AppHandle, permanent: bool) -> Result<PathBuf, String> {
    let base = base_dir(app, permanent);
    if !base.exists() {
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    }
    Ok(base)
}

/// Join "sicuro" base + rel (no assoluti, no uscita dalla base, non richiede esistenza).
fn safe_join(base: &Path, rel: &str) -> Result<PathBuf, String> {
    if rel.is_empty() || rel == "." {
        return Ok(base.to_path_buf());
    }
    let mut out = PathBuf::from(base);
    // profondità sotto la base (impedisce .. di risalire sopra)
    let mut depth: isize = 0;

    for comp in Path::new(rel).components() {
        match comp {
            Component::Normal(c) => { out.push(c); depth += 1; }
            Component::CurDir => {}
            Component::ParentDir => {
                if depth > 0 { out.pop(); depth -= 1; }
                else { return Err("Path traversal non consentito".into()); }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("Percorsi assoluti non consentiti".into());
            }
        }
    }
    Ok(out)
}

/// Path che DEVE ESISTERE (listDir/stat)
fn resolve_existing(app: &AppHandle, rel: &str, permanent: bool) -> Result<PathBuf, String> {
    let base = ensure_base_exists(app, permanent)?;
    let p = safe_join(&base, rel)?;
    if !p.exists() {
        return Err("No such file or directory".into());
    }
    Ok(p)
}

/// Path che PUÒ NON ESISTERE (mkdir/rm target)
fn resolve_any(app: &AppHandle, rel: &str, permanent: bool) -> Result<PathBuf, String> {
    let base = ensure_base_exists(app, permanent)?;
    safe_join(&base, rel)
}

#[tauri::command]
pub fn bd_fs_list_dir(app: AppHandle, rel: String, permanent: bool) -> Result<Vec<FsEntry>, String> {
    let dir = resolve_existing(&app, &rel, permanent)?;
    let mut out = Vec::new();
    for e in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let md = e.metadata().map_err(|e| e.to_string())?;
        let is_dir = md.is_dir();
        let size = if md.is_file() { Some(md.len()) } else { None };
        let name = e.file_name().to_string_lossy().into_owned();
        let path_str = e.path().to_string_lossy().into_owned();
        out.push(FsEntry { name, path: path_str, is_dir, size });
    }
    Ok(out)
}

#[tauri::command]
pub fn bd_fs_mkdir(app: AppHandle, rel: String, permanent: bool) -> Result<(), String> {
    let dir = resolve_any(&app, &rel, permanent)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_fs_rm(app: AppHandle, rel: String, permanent: bool, recursive: bool) -> Result<(), String> {
    let p = resolve_any(&app, &rel, permanent)?;
    if !p.exists() {
        // idempotente: rimuovere ciò che non esiste è OK
        return Ok(());
    }
    if recursive {
        fs::remove_dir_all(&p).map_err(|e| e.to_string())
    } else if p.is_dir() {
        fs::remove_dir(&p).map_err(|e| e.to_string())
    } else {
        fs::remove_file(&p).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn bd_fs_stat(app: AppHandle, rel: String, permanent: bool) -> Result<FsEntry, String> {
    let p = resolve_existing(&app, &rel, permanent)?;
    let md = fs::metadata(&p).map_err(|e| e.to_string())?;
    Ok(FsEntry {
        name: p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
        path: p.to_string_lossy().into_owned(),
        is_dir: md.is_dir(),
        size: if md.is_file() { Some(md.len()) } else { None },
    })
}

// ---- WRITE TEXT ----
#[tauri::command]
pub fn bd_fs_write_text(
  app: AppHandle,
  rel: String,
  permanent: Option<bool>,
  contents: String,
  create_dirs: Option<bool>,
  append: Option<bool>,
) -> Result<(), String> {
  let permanent = permanent.unwrap_or(false);
  let path = resolve_any(&app, &rel, permanent)?;
  if create_dirs.unwrap_or(true) {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
  }
  let mut file = if append.unwrap_or(false) {
    std::fs::OpenOptions::new().create(true).append(true).open(&path)
  } else {
    std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&path)
  }.map_err(|e| e.to_string())?;

  file.write_all(contents.as_bytes()).map_err(|e| e.to_string())
}

// ---- READ TEXT ----
#[tauri::command]
pub fn bd_fs_read_text(
  app: AppHandle,
  rel: String,
  permanent: Option<bool>,
) -> Result<String, String> {
  let permanent = permanent.unwrap_or(false);
  let path = resolve_existing(&app, &rel, permanent)?;
  std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// ---- WRITE BYTES (base64) ----
#[tauri::command]
pub fn bd_fs_write_bytes(
  app: AppHandle,
  rel: String,
  permanent: Option<bool>,
  data_base64: String,
  create_dirs: Option<bool>,
) -> Result<(), String> {
  let permanent = permanent.unwrap_or(false);
  let path = resolve_any(&app, &rel, permanent)?;
  if create_dirs.unwrap_or(true) {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
  }
  let bytes = general_purpose::STANDARD
    .decode(data_base64.as_bytes())
    .map_err(|e| e.to_string())?;
  std::fs::write(path, bytes).map_err(|e| e.to_string())
}

// ---- READ BYTES (base64) ----
#[tauri::command]
pub fn bd_fs_read_bytes(
  app: AppHandle,
  rel: String,
  permanent: Option<bool>,
) -> Result<String, String> {
  let permanent = permanent.unwrap_or(false);
  let path = resolve_existing(&app, &rel, permanent)?;
  let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
  Ok(general_purpose::STANDARD.encode(buf))
}

// ---- EXISTS ----
#[tauri::command]
pub fn bd_fs_exists(
  app: AppHandle,
  rel: String,
  permanent: Option<bool>,
) -> Result<bool, String> {
  let permanent = permanent.unwrap_or(false);
  let path = resolve_any(&app, &rel, permanent)?; // non richiede esistenza
  Ok(path.exists())
}

// ---- MOVE (file o directory) ----
#[tauri::command]
pub fn bd_fs_move(
  app: AppHandle,
  src: String,
  dest: String,
  permanent: Option<bool>,
  create_dirs: Option<bool>,
  overwrite: Option<bool>,
) -> Result<(), String> {
  let permanent = permanent.unwrap_or(false);
  let src_path = resolve_existing(&app, &src, permanent)?;
  let mut dest_path = resolve_any(&app, &dest, permanent)?;
  let create_dirs = create_dirs.unwrap_or(true);
  let overwrite = overwrite.unwrap_or(false);

  let is_dir_target = dest.ends_with('/') || dest.ends_with('\\');

  if is_dir_target {
    if create_dirs {
      std::fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
    }
    let file_name = src_path.file_name().ok_or("Invalid source name")?;
    dest_path.push(file_name);
  } else {
    if create_dirs {
      if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
      }
    }
  }

  if dest_path.exists() {
    if overwrite {
      if dest_path.is_dir() {
        std::fs::remove_dir_all(&dest_path)
      } else {
        std::fs::remove_file(&dest_path)
      }
      .map_err(|e| e.to_string())?;
    } else {
      return Err("Destination already exists".into());
    }
  }

  std::fs::rename(&src_path, &dest_path).map_err(|e| e.to_string())
}


// ---- COPY (file o directory; ricorsivo opzionale) ----
#[tauri::command]
pub fn bd_fs_copy(
  app: AppHandle,
  src: String,
  dest: String,
  permanent: Option<bool>,
  recursive: Option<bool>,
  create_dirs: Option<bool>,
  overwrite: Option<bool>,
) -> Result<(), String> {
  let permanent = permanent.unwrap_or(false);
  let src_path = resolve_existing(&app, &src, permanent)?;
  let mut dest_path = resolve_any(&app, &dest, permanent)?;
  let recursive = recursive.unwrap_or(false);
  let create_dirs = create_dirs.unwrap_or(true);
  let overwrite = overwrite.unwrap_or(false);

  let is_dir_target = dest.ends_with('/') || dest.ends_with('\\');

  if src_path.is_file() {
    if is_dir_target {
      if create_dirs {
        std::fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
      }
      let file_name = src_path.file_name().ok_or("Invalid source name")?;
      dest_path.push(file_name);
    } else if create_dirs {
      if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
      }
    }

    if dest_path.exists() && !overwrite {
      return Err("Destination already exists".into());
    }
    std::fs::copy(&src_path, &dest_path).map_err(|e| e.to_string())?;
    return Ok(());
  }

  // src è directory
  if !recursive {
    return Err("Source is a directory; set recursive=true to copy".into());
  }

  if is_dir_target {
    if create_dirs {
      std::fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
    }
  } else {
    return Err("Destination must be a directory when copying a folder".into());
  }

  for entry in WalkDir::new(&src_path) {
    let entry = entry.map_err(|e| e.to_string())?;
    let path = entry.path();
    let rel = path.strip_prefix(&src_path).map_err(|e| e.to_string())?;
    let target = dest_path.join(rel);

    if path.is_dir() {
      if create_dirs {
        std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
      }
    } else {
      if target.exists() && !overwrite {
        return Err(format!("Destination already exists: {}", target.to_string_lossy()));
      }
      if let Some(parent) = target.parent() {
        if create_dirs {
          std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
      }
      std::fs::copy(path, &target).map_err(|e| e.to_string())?;
    }
  }

  Ok(())
}



#[tauri::command]
pub fn bd_fs_clear_cache(app: AppHandle) -> Result<(), String> {
    let base = base_dir(&app, false); // false = cache
    if base.exists() {
        fs::remove_dir_all(&base).map_err(|e| e.to_string())?;
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Serialize)]
pub struct FsPaths {
    pub cache: String,
    pub data: String,
}

#[tauri::command]
pub fn bd_fs_paths(app: AppHandle) -> Result<FsPaths, String> {
    let cache = base_dir(&app, false);
    let data = base_dir(&app, true);
    Ok(FsPaths {
        cache: cache.to_string_lossy().into_owned(),
        data: data.to_string_lossy().into_owned(),
    })
}

// Espone la base dir "data" (persistente) usando la stessa logica interna
pub fn bd_fs_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
  ensure_base_exists(app, true)
}

// Espone la base dir "cache" (non persistente)
pub fn bd_fs_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
  ensure_base_exists(app, false)
}

// Join sicuro (stessa logica di safe_join) rispetto a "data"
pub fn bd_fs_safe_join_data(app: &AppHandle, rel: &str) -> Result<PathBuf, String> {
  let base = ensure_base_exists(app, true)?;
  safe_join(&base, rel)
}

// Join sicuro (stessa logica di safe_join) rispetto a "cache"
pub fn bd_fs_safe_join_cache(app: &AppHandle, rel: &str) -> Result<PathBuf, String> {
  let base = ensure_base_exists(app, false)?;
  safe_join(&base, rel)
}
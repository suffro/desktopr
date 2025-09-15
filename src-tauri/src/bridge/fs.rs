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
        return Ok(()); // idempotente
    }

    // Se siamo nello scope data, spostiamo in _trash (ora _trash è a pari livello di data/cache)
    if permanent {
        if p.is_dir() && !recursive {
            let is_empty = std::fs::read_dir(&p).map_err(|e| e.to_string())?.next().is_none();
            if !is_empty {
                return Err("Directory not empty".into());
            }
        }
        return move_rel_in_data_to_trash(&app, &rel);
    }

    // cache: elimina davvero
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

// ---- MOVE ----
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

// ---- COPY ----
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
    let base = base_dir(&app, false);
    if base.exists() {
        fs::remove_dir_all(&base).map_err(|e| e.to_string())?;
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn bd_fs_clear_data(app: AppHandle) -> Result<(), String> {
    let data = ensure_base_exists(&app, true)?;
    let trash = trash_dir(&app)?; // _trash al pari di data/cache

    // Sposta ogni entry in data/ dentro _trash/
    for entry in std::fs::read_dir(&data).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let src = entry.path();
        let dest = trash.join(name);
        let final_dest = if dest.exists() {
            unique_with_suffix(dest, "(cleared)")
        } else {
            dest
        };
        std::fs::rename(&src, &final_dest).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn bd_fs_data_clear_trash(app: AppHandle) -> Result<(), String> {
    let trash = trash_dir(&app)?;
    if trash.exists() {
        std::fs::remove_dir_all(&trash).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&trash).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn bd_fs_data_recover_trash(app: AppHandle, trash_rel_path: String) -> Result<(), String> {
    let data = ensure_base_exists(&app, true)?;
    let trash = trash_dir(&app)?;

    let recover_one = |src: &Path| -> Result<(), String> {
        let rel_from_trash = src.strip_prefix(&trash).map_err(|e| e.to_string())?;
        let mut dest = data.join(rel_from_trash);
        if dest.exists() {
            dest = unique_with_suffix(dest, "(recovered)");
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::rename(src, &dest).map_err(|e| e.to_string())
    };

    if trash_rel_path.trim().is_empty() || trash_rel_path.trim() == "/" {
        let mut to_recover = Vec::new();
        for entry in std::fs::read_dir(&trash).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            to_recover.push(entry.path());
        }
        for src in to_recover {
            if src.exists() {
                recover_one(&src)?;
            }
        }
        return Ok(());
    }

    let target = safe_join(&trash, &trash_rel_path)?;
    if !target.exists() {
        return Err("No such file or directory in trash".into());
    }
    recover_one(&target)
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

/* =========================
   TRASH: helper e comandi
   ========================= */

// Directory _trash a pari livello di data e cache.
// Implementazione: prendo la data dir e uso il suo parent per creare "_trash".
fn trash_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let data = ensure_base_exists(app, true)?;
  let parent = data.parent()
    .ok_or_else(|| "Impossibile calcolare la directory parent per _trash".to_string())?;
  let trash = parent.join("_trash");
  if !trash.exists() {
    std::fs::create_dir_all(&trash).map_err(|e| e.to_string())?;
  }
  Ok(trash)
}

// Nome unico con suffisso, p.es. "(recovered)"
fn unique_with_suffix(mut dest: PathBuf, suffix: &str) -> PathBuf {
  if !dest.exists() {
    return dest;
  }
  let parent = dest.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
  let stem = dest.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
  let ext  = dest.extension().map(|e| e.to_string_lossy().to_string());

  // Se non c'è stem (es. path finisce con /), usa nome completo
  let base_name = if stem.is_empty() {
    dest.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "item".into())
  } else { stem.clone() };

  let mut candidate = if let Some(ext) = &ext {
    parent.join(format!("{base_name} {suffix}.{ext}"))
  } else {
    parent.join(format!("{base_name} {suffix}"))
  };

  let mut i = 2u32;
  while candidate.exists() {
    candidate = if let Some(ext) = &ext {
      parent.join(format!("{base_name} {suffix} {i}.{ext}"))
    } else {
      parent.join(format!("{base_name} {suffix} {i}"))
    };
    i += 1;
  }
  candidate
}

// Sposta in _trash mantenendo la struttura relativa sotto data/.
// Se esiste già in _trash, usa nome con suffisso "(deleted)".
fn move_rel_in_data_to_trash(app: &AppHandle, rel: &str) -> Result<(), String> {
  let data_base = ensure_base_exists(app, true)?;
  let src = safe_join(&data_base, rel)?;
  if !src.exists() {
    return Ok(()); // idempotente
  }
  let trash = trash_dir(app)?;
  let dest = trash.join(rel);
  if let Some(parent) = dest.parent() {
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
  }
  let final_dest = if dest.exists() {
    unique_with_suffix(dest, "(deleted)")
  } else {
    dest
  };
  std::fs::rename(&src, &final_dest).map_err(|e| e.to_string())
}

// ---- LIST DIR nel contesto _trash ----
#[tauri::command]
pub fn bd_fs_trash_list_dir(app: AppHandle, rel: String) -> Result<Vec<FsEntry>, String> {
  let trash = trash_dir(&app)?;
  let dir = {
    let p = safe_join(&trash, &rel)?;
    if !p.exists() {
      return Err("No such file or directory in trash".into());
    }
    p
  };
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

/* =========================
   TRASH: funzioni aggiuntive
   ========================= */

// Stat nel contesto _trash
#[tauri::command]
pub fn bd_fs_trash_stat(app: AppHandle, rel: String) -> Result<FsEntry, String> {
  let trash = trash_dir(&app)?;
  let p = safe_join(&trash, &rel)?;
  if !p.exists() {
    return Err("No such file or directory in trash".into());
  }
  let md = fs::metadata(&p).map_err(|e| e.to_string())?;
  Ok(FsEntry {
      name: p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
      path: p.to_string_lossy().into_owned(),
      is_dir: md.is_dir(),
      size: if md.is_file() { Some(md.len()) } else { None },
  })
}

// Exists nel contesto _trash
#[tauri::command]
pub fn bd_fs_trash_exists(app: AppHandle, rel: String) -> Result<bool, String> {
  let trash = trash_dir(&app)?;
  let p = safe_join(&trash, &rel)?;
  Ok(p.exists())
}

// Read text nel contesto _trash
#[tauri::command]
pub fn bd_fs_trash_read_text(app: AppHandle, rel: String) -> Result<String, String> {
  let trash = trash_dir(&app)?;
  let p = safe_join(&trash, &rel)?;
  if !p.exists() {
    return Err("No such file or directory in trash".into());
  }
  if p.is_dir() {
    return Err("Path is a directory".into());
  }
  std::fs::read_to_string(p).map_err(|e| e.to_string())
}

// Read bytes (base64) nel contesto _trash
#[tauri::command]
pub fn bd_fs_trash_read_bytes(app: AppHandle, rel: String) -> Result<String, String> {
  let trash = trash_dir(&app)?;
  let p = safe_join(&trash, &rel)?;
  if !p.exists() {
    return Err("No such file or directory in trash".into());
  }
  if p.is_dir() {
    return Err("Path is a directory".into());
  }
  let mut f = std::fs::File::open(p).map_err(|e| e.to_string())?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
  Ok(general_purpose::STANDARD.encode(buf))
}

/* =========================
   DIAGNOSTICS: helper e comandi
   ========================= */

// Directory _diagnostics a pari livello di data/cache/_trash.
fn diagnostics_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let data = ensure_base_exists(app, true)?;
  let parent = data.parent()
    .ok_or_else(|| "Impossibile calcolare la directory parent per _diagnostics".to_string())?;
  let diag = parent.join("_diagnostics");
  if !diag.exists() {
    std::fs::create_dir_all(&diag).map_err(|e| e.to_string())?;
  }
  Ok(diag)
}

// Join sicuro rispetto a _diagnostics
pub fn bd_fs_safe_join_diagnostics(app: &AppHandle, rel: &str) -> Result<PathBuf, String> {
  let base = diagnostics_dir(app)?;
  safe_join(&base, rel)
}

// Sposta in _trash preservando la struttura relativa sotto _diagnostics.
// Se esiste già in _trash, usa nome con suffisso "(deleted)".
fn move_rel_in_diagnostics_to_trash(app: &AppHandle, rel: &str) -> Result<(), String> {
  let diag_base = diagnostics_dir(app)?;
  let src = safe_join(&diag_base, rel)?;
  if !src.exists() {
    return Ok(()); // idempotente
  }
  let trash = trash_dir(app)?;
  let dest = trash.join(rel);
  if let Some(parent) = dest.parent() {
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
  }
  let final_dest = if dest.exists() {
    unique_with_suffix(dest, "(deleted)")
  } else {
    dest
  };
  std::fs::rename(&src, &final_dest).map_err(|e| e.to_string())
}

// --- list dir in _diagnostics ---
#[tauri::command]
pub fn bd_fs_diagnostics_list_dir(app: AppHandle, rel: String) -> Result<Vec<FsEntry>, String> {
  let dir = bd_fs_safe_join_diagnostics(&app, &rel)?;
  if !dir.exists() {
    return Err("No such file or directory in _diagnostics".into());
  }
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

// --- stat in _diagnostics ---
#[tauri::command]
pub fn bd_fs_diagnostics_stat(app: AppHandle, rel: String) -> Result<FsEntry, String> {
  let p = bd_fs_safe_join_diagnostics(&app, &rel)?;
  if !p.exists() {
    return Err("No such file or directory in _diagnostics".into());
  }
  let md = fs::metadata(&p).map_err(|e| e.to_string())?;
  Ok(FsEntry {
    name: p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
    path: p.to_string_lossy().into_owned(),
    is_dir: md.is_dir(),
    size: if md.is_file() { Some(md.len()) } else { None },
  })
}

// --- write text in _diagnostics ---
#[tauri::command]
pub fn bd_fs_diagnostics_write_text(
  app: AppHandle,
  rel: String,
  contents: String,
  create_dirs: Option<bool>,
  append: Option<bool>,
) -> Result<(), String> {
  let path = bd_fs_safe_join_diagnostics(&app, &rel)?;
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

// --- read text in _diagnostics ---
#[tauri::command]
pub fn bd_fs_diagnostics_read_text(app: AppHandle, rel: String) -> Result<String, String> {
  let path = bd_fs_safe_join_diagnostics(&app, &rel)?;
  if !path.exists() {
    return Err("No such file or directory in _diagnostics".into());
  }
  std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// --- write bytes base64 in _diagnostics ---
#[tauri::command]
pub fn bd_fs_diagnostics_write_bytes(
  app: AppHandle,
  rel: String,
  data_base64: String,
  create_dirs: Option<bool>,
) -> Result<(), String> {
  let path = bd_fs_safe_join_diagnostics(&app, &rel)?;
  if create_dirs.unwrap_or(true) {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
  }
  let bytes = general_purpose::STANDARD
    .decode(data_base64.as_bytes())
    .map_err(|e| e.to_string())?;
  std::fs::write(path, bytes).map_err(|e| e.to_string())
}

// --- read bytes base64 in _diagnostics ---
#[tauri::command]
pub fn bd_fs_diagnostics_read_bytes(app: AppHandle, rel: String) -> Result<String, String> {
  let path = bd_fs_safe_join_diagnostics(&app, &rel)?;
  if !path.exists() {
    return Err("No such file or directory in _diagnostics".into());
  }
  let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
  Ok(general_purpose::STANDARD.encode(buf))
}

// --- rm relativo in _diagnostics (sposta in _trash, non definitiva) ---
#[tauri::command]
pub fn bd_fs_diagnostics_rm(app: AppHandle, rel: String, recursive: bool) -> Result<(), String> {
  let p = bd_fs_safe_join_diagnostics(&app, &rel)?;
  if !p.exists() {
    return Ok(());
  }
  if p.is_dir() && !recursive {
    let is_empty = std::fs::read_dir(&p).map_err(|e| e.to_string())?.next().is_none();
    if !is_empty {
      return Err("Directory not empty".into());
    }
  }
  move_rel_in_diagnostics_to_trash(&app, &rel)
}

// --- clear totale di _diagnostics (sposta tutto in _trash) ---
#[tauri::command]
pub fn bd_fs_diagnostics_clear(app: AppHandle) -> Result<(), String> {
  let diag = diagnostics_dir(&app)?;
  let trash = trash_dir(&app)?;
  for entry in std::fs::read_dir(&diag).map_err(|e| e.to_string())? {
    let entry = entry.map_err(|e| e.to_string())?;
    let name = entry.file_name();
    let src = entry.path();
    let dest = trash.join(name);
    let final_dest = if dest.exists() {
      unique_with_suffix(dest, "(cleared)")
    } else {
      dest
    };
    std::fs::rename(&src, &final_dest).map_err(|e| e.to_string())?;
  }
  Ok(())
}

// Exists nel contesto _diagnostics
#[tauri::command]
pub fn bd_fs_diagnostics_exists(app: AppHandle, rel: String) -> Result<bool, String> {
  let diag = diagnostics_dir(&app)?;
  let p = safe_join(&diag, &rel)?;
  Ok(p.exists())
}


/* =========================
   SANBOX: helper e comandi
   ========================= */

// Directory _sandbox a pari livello di data e cache.
// Implementazione: prendo la data dir e uso il suo parent per creare "_sandbox".
fn sandbox_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let data = ensure_base_exists(app, true)?;
  let parent = data.parent()
    .ok_or_else(|| "Impossibile calcolare la directory parent per _sandbox".to_string())?;
  let sandbox_dir_base = parent.join("_sandbox");
  if !sandbox_dir_base.exists() {
    std::fs::create_dir_all(&sandbox_dir_base).map_err(|e| e.to_string())?;
  }
  Ok(sandbox_dir_base)
}

// Read text nel contesto _sanbox
#[tauri::command]
pub fn bd_fs_sandbox_read_text(app: AppHandle, rel: String) -> Result<String, String> {
  let sandbox_dir_base = sandbox_dir(&app)?;
  let p = safe_join(&sandbox_dir_base, &rel)?;
  if !p.exists() {
    return Err("No such file or directory in sandbox".into());
  }
  if p.is_dir() {
    return Err("Path is a directory".into());
  }
  std::fs::read_to_string(p).map_err(|e| e.to_string())
}

// Read meta.txt nel contesto _sanbox
#[tauri::command]
pub fn bd_fs_sandbox_read_meta(app: AppHandle, job_id: String) -> Result<String, String> {
  let sandbox_dir_base = sandbox_dir(&app)?;
  let rel = format!("{}/{}", job_id, ("_meta.txt".to_string()));
  let p = safe_join(&sandbox_dir_base, &rel)?;
  if !p.exists() {
    return Err("No such file or directory in sandbox".into());
  }
  if p.is_dir() {
    return Err("Path is a directory".into());
  }
  std::fs::read_to_string(p).map_err(|e| e.to_string())
}

// Read stdin.json nel contesto _sanbox
#[tauri::command]
pub fn bd_fs_sandbox_read_stdin(app: AppHandle, job_id: String) -> Result<String, String> {
  let sandbox_dir_base = sandbox_dir(&app)?;
  let rel = format!("{}/{}", job_id, ("_stdin.json".to_string()));
  let p = safe_join(&sandbox_dir_base, &rel)?;
  if !p.exists() {
    return Err("No such file or directory in sandbox".into());
  }
  if p.is_dir() {
    return Err("Path is a directory".into());
  }
  std::fs::read_to_string(p).map_err(|e| e.to_string())
}


// ---- LIST DIR nel contesto _sandbox ----
#[tauri::command]
pub fn bd_fs_sandbox_list_dir(app: AppHandle, rel: String) -> Result<Vec<FsEntry>, String> {
  let sandbox_dir_base = sandbox_dir(&app)?;
  let dir = {
    let p = safe_join(&sandbox_dir_base, &rel)?;
    if !p.exists() {
      return Err("No such file or directory in sandbox".into());
    }
    p
  };
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
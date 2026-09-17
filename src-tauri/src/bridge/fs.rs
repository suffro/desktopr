use crate::bridge::companion::is_companion_label;
use base64::{engine::general_purpose, Engine as _}; // Cargo.toml: base64 = "0.22"
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use tauri::{AppHandle, Manager, WebviewWindow};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Serialize)]
pub struct FsPaths {
    pub cache: String,
    pub data: String,
}

// ---------------------------------
// Scope / root helpers
// ---------------------------------

// Returns the scope directory name.
// - None => ".main"
// - Some(label) => ".<label>" after validation
fn scope_dir_name(window_label: Option<&str>) -> Result<String, String> {
    match window_label {
        None => Ok(".main".to_string()),
        Some(raw) => {
            let label = raw.trim();

            if label.is_empty() {
                return Err("window_label cannot be empty".into());
            }

            if label == "main" {
                return Err("window_label 'main' is reserved".into());
            }

            if !label
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err("window_label contains invalid characters".into());
            }

            Ok(format!(".{}", label))
        }
    }
}

// Returns the Desktopr scope root inside app_data_dir/app_cache_dir.
// Examples:
// - app_data_dir()/.desktopr/.main
// - app_cache_dir()/.desktopr/.my_window
fn desktopr_scope_root(
    app: &AppHandle,
    permanent: bool,
    window_label: Option<&str>,
) -> Result<PathBuf, String> {
    let base = if permanent {
        app.path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap())
    } else {
        app.path()
            .app_cache_dir()
            .unwrap_or_else(|_| std::env::temp_dir())
    };

    let scope = scope_dir_name(window_label)?;
    let root = base.join(".desktopr").join(scope);

    if !root.exists() {
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    }

    Ok(root)
}

/// Returns the public base dir for fs.data / fs.cache.
/// - permanent=true  -> app_data_dir()/.desktopr/.main/data
/// - permanent=false -> app_cache_dir()/.desktopr/.main/cache
fn base_dir(app: &AppHandle, permanent: bool) -> PathBuf {
    let scope_root = desktopr_scope_root(app, permanent, None).unwrap_or_else(|_| {
        if permanent {
            app.path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap())
                .join(".desktopr")
                .join(".main")
        } else {
            app.path()
                .app_cache_dir()
                .unwrap_or_else(|_| std::env::temp_dir())
                .join(".desktopr")
                .join(".main")
        }
    });

    if permanent {
        scope_root.join("data")
    } else {
        scope_root.join("cache")
    }
}

fn ensure_base_exists(app: &AppHandle, permanent: bool) -> Result<PathBuf, String> {
    let base = base_dir(app, permanent);
    if !base.exists() {
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    }
    Ok(base)
}

/// Returns the public base dir for a specific window scope.
/// - permanent=true  -> app_data_dir()/.desktopr/.<window_label>/data
/// - permanent=false -> app_cache_dir()/.desktopr/.<window_label>/cache
/// - window_label=None -> same as default ".main"
fn base_dir_scoped(
    app: &AppHandle,
    permanent: bool,
    window_label: &Option<String>,
) -> Result<PathBuf, String> {
    let scope_root = desktopr_scope_root(app, permanent, window_label.as_deref())?;
    let base = if permanent {
        scope_root.join("data")
    } else {
        scope_root.join("cache")
    };

    if !base.exists() {
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    }

    Ok(base)
}

/// Safe join base + relative path.
/// - No absolute paths
/// - No escaping above the base
/// - Does not require the final path to exist
fn safe_join(base: &Path, rel: &str) -> Result<PathBuf, String> {
    if rel.is_empty() || rel == "." {
        return Ok(base.to_path_buf());
    }

    let mut out = PathBuf::from(base);
    let mut depth: isize = 0;

    for comp in Path::new(rel).components() {
        match comp {
            Component::Normal(c) => {
                out.push(c);
                depth += 1;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if depth > 0 {
                    out.pop();
                    depth -= 1;
                } else {
                    return Err("Path traversal not allowed".into());
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("Absolute paths are not allowed".into());
            }
        }
    }

    Ok(out)
}

// Validates a WASM module name used as plugin storage scope.
fn validate_plugin_storage_module(module: &str) -> Result<String, String> {
    let clean = module.trim();

    if clean.is_empty() {
        return Err("plugin_storage_module cannot be empty".into());
    }

    if !clean.ends_with(".wasm") {
        return Err("plugin_storage_module must end with .wasm".into());
    }

    if clean == "." || clean == ".." {
        return Err("invalid plugin_storage_module".into());
    }

    if Path::new(clean).is_absolute()
        || clean.contains('/')
        || clean.contains('\\')
        || clean.contains(std::path::is_separator)
    {
        return Err("invalid plugin_storage_module".into());
    }

    Ok(clean.to_string())
}

// Returns the persistent storage root for a plugin module.
// This lets the existing fs commands target _external_modules_storage/<module>.
fn plugin_storage_base_dir(
    app: &AppHandle,
    plugin_storage_module: &str,
) -> Result<PathBuf, String> {
    let module = validate_plugin_storage_module(plugin_storage_module)?;
    let root = desktopr_scope_root(app, true, None)?;
    let base = root.join("_external_modules_storage").join(module);

    if !base.exists() {
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    }

    Ok(base)
}

// Returns the effective root for fs commands.
// If plugin_storage_module is set, it overrides data/cache resolution.
fn scoped_or_plugin_base_dir(
    app: &AppHandle,
    permanent: bool,
    window_label: &Option<String>,
    plugin_storage_module: &Option<String>,
) -> Result<PathBuf, String> {
    if let Some(module) = plugin_storage_module.as_deref() {
        return plugin_storage_base_dir(app, module);
    }

    base_dir_scoped(app, permanent, window_label)
}

/// Existing path resolver for window-scoped data/cache or plugin storage.
fn resolve_existing_scoped(
    app: &AppHandle,
    rel: &str,
    permanent: bool,
    window_label: &Option<String>,
    plugin_storage_module: &Option<String>,
) -> Result<PathBuf, String> {
    let base = scoped_or_plugin_base_dir(app, permanent, window_label, plugin_storage_module)?;
    let p = safe_join(&base, rel)?;

    if !p.exists() {
        return Err("No such file or directory".into());
    }

    Ok(p)
}

/// Non-existing path resolver for window-scoped data/cache or plugin storage.
fn resolve_any_scoped(
    app: &AppHandle,
    rel: &str,
    permanent: bool,
    window_label: &Option<String>,
    plugin_storage_module: &Option<String>,
) -> Result<PathBuf, String> {
    let base = scoped_or_plugin_base_dir(app, permanent, window_label, plugin_storage_module)?;
    safe_join(&base, rel)
}

// ---------------------------------
// Internal Desktopr dirs
// ---------------------------------

fn trash_dir_scoped(app: &AppHandle, window_label: Option<&str>) -> Result<PathBuf, String> {
    let trash = desktopr_scope_root(app, true, window_label)?.join("_trash");
    if !trash.exists() {
        std::fs::create_dir_all(&trash).map_err(|e| e.to_string())?;
    }
    Ok(trash)
}

fn trash_dir(app: &AppHandle) -> Result<PathBuf, String> {
    trash_dir_scoped(app, None)
}

fn diagnostics_dir_scoped(app: &AppHandle, window_label: Option<&str>) -> Result<PathBuf, String> {
    let diag = desktopr_scope_root(app, true, window_label)?.join("_diagnostics");
    if !diag.exists() {
        std::fs::create_dir_all(&diag).map_err(|e| e.to_string())?;
    }
    Ok(diag)
}

fn diagnostics_dir(app: &AppHandle) -> Result<PathBuf, String> {
    diagnostics_dir_scoped(app, None)
}

// ---------------------------------
// Caller identity
// ---------------------------------

// Returns the filesystem scope of the calling window. The scope is derived from
// the window Tauri injects into the command, never from frontend input:
// companion windows are confined to their own scope and every other window uses
// the main scope. A `window_label` sent by the frontend must match that scope.
fn caller_scope(
    window: &WebviewWindow,
    requested: Option<String>,
) -> Result<Option<String>, String> {
    scope_for_label(window.label(), requested)
}

fn scope_for_label(label: &str, requested: Option<String>) -> Result<Option<String>, String> {
    let own = is_companion_label(label).then(|| label.to_string());

    match requested {
        Some(requested) if Some(requested.trim()) != own.as_deref() => {
            Err("window_label does not match the calling window".into())
        }
        _ => Ok(own),
    }
}

// Plugin storage belongs to the main app; companion windows cannot reach it.
fn caller_plugin_storage(
    window: &WebviewWindow,
    module: Option<String>,
) -> Result<Option<String>, String> {
    plugin_storage_for_label(window.label(), module)
}

fn plugin_storage_for_label(label: &str, module: Option<String>) -> Result<Option<String>, String> {
    if module.is_some() && is_companion_label(label) {
        return Err("plugin storage is not available to companion windows".into());
    }
    Ok(module)
}

// ---------------------------------
// Public fs.data / fs.cache commands
// ---------------------------------

#[tauri::command]
pub fn dtr_fs_list_dir(
    app: AppHandle,
    rel: String,
    permanent: bool,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, rel is resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<Vec<FsEntry>, String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let dir =
        resolve_existing_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;
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
pub fn dtr_fs_mkdir(
    app: AppHandle,
    rel: String,
    permanent: bool,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, rel is resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<(), String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let dir = resolve_any_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn dtr_fs_rm(
    app: AppHandle,
    rel: String,
    permanent: bool,
    recursive: bool,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. Plugin storage deletes permanently
    // and does not use the data trash system.
    plugin_storage_module: Option<String>,
) -> Result<(), String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let p = resolve_any_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;

    if !p.exists() {
        return Ok(());
    }

    // Plugin storage always deletes for real, even though it lives under app data.
    if plugin_storage_module.is_some() {
        if recursive {
            return if p.is_dir() {
                fs::remove_dir_all(&p).map_err(|e| e.to_string())
            } else {
                fs::remove_file(&p).map_err(|e| e.to_string())
            };
        }

        return if p.is_dir() {
            fs::remove_dir(&p).map_err(|e| e.to_string())
        } else {
            fs::remove_file(&p).map_err(|e| e.to_string())
        };
    }

    // In data scope, move to scoped trash instead of deleting.
    if permanent {
        if p.is_dir() && !recursive {
            let is_empty = std::fs::read_dir(&p)
                .map_err(|e| e.to_string())?
                .next()
                .is_none();
            if !is_empty {
                return Err("Directory not empty".into());
            }
        }
        return move_rel_in_data_to_trash(&app, &rel, window_label.as_deref());
    }

    // In cache scope, delete for real.
    if recursive {
        fs::remove_dir_all(&p).map_err(|e| e.to_string())
    } else if p.is_dir() {
        fs::remove_dir(&p).map_err(|e| e.to_string())
    } else {
        fs::remove_file(&p).map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn dtr_fs_stat(
    app: AppHandle,
    rel: String,
    permanent: bool,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, rel is resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<FsEntry, String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let p = resolve_existing_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;
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

// ---- WRITE TEXT ----
#[tauri::command]
#[allow(clippy::too_many_arguments)] // IPC arguments
pub fn dtr_fs_write_text(
    app: AppHandle,
    rel: String,
    permanent: Option<bool>,
    contents: String,
    create_dirs: Option<bool>,
    append: Option<bool>,
    window: WebviewWindow,
    window_label: Option<String>,
    plugin_storage_module: Option<String>,
) -> Result<(), String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    fs_write_text_in_scope(
        app,
        rel,
        permanent,
        contents,
        create_dirs,
        append,
        window_label,
        plugin_storage_module,
    )
}

// Writes text in an explicit scope; for runtime code that has no calling window.
#[allow(clippy::too_many_arguments)]
pub fn fs_write_text_in_scope(
    app: AppHandle,
    rel: String,
    permanent: Option<bool>,
    contents: String,
    create_dirs: Option<bool>,
    append: Option<bool>,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, rel is resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<(), String> {
    let permanent = permanent.unwrap_or(false);
    let path = resolve_any_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;
    if create_dirs.unwrap_or(true) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let mut file = if append.unwrap_or(false) {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
    } else {
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
    }
    .map_err(|e| e.to_string())?;

    file.write_all(contents.as_bytes())
        .map_err(|e| e.to_string())
}

// ---- READ TEXT ----
#[tauri::command]
pub fn dtr_fs_read_text(
    app: AppHandle,
    rel: String,
    permanent: Option<bool>,
    window: WebviewWindow,
    window_label: Option<String>,
    plugin_storage_module: Option<String>,
) -> Result<String, String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    fs_read_text_in_scope(app, rel, permanent, window_label, plugin_storage_module)
}

// Reads text in an explicit scope; for runtime code that has no calling window.
pub fn fs_read_text_in_scope(
    app: AppHandle,
    rel: String,
    permanent: Option<bool>,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, rel is resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<String, String> {
    let permanent = permanent.unwrap_or(false);
    let path =
        resolve_existing_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// ---- WRITE BYTES (base64) ----
#[tauri::command]
#[allow(clippy::too_many_arguments)] // IPC arguments
pub fn dtr_fs_write_bytes(
    app: AppHandle,
    rel: String,
    permanent: Option<bool>,
    data_base64: String,
    create_dirs: Option<bool>,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, rel is resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<(), String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let permanent = permanent.unwrap_or(false);
    let path = resolve_any_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;
    if create_dirs.unwrap_or(true) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let bytes = general_purpose::STANDARD
        .decode(data_base64.as_bytes())
        .map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

// ---- READ BYTES (base64) ----
#[tauri::command]
pub fn dtr_fs_read_bytes(
    app: AppHandle,
    rel: String,
    permanent: Option<bool>,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, rel is resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<String, String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let permanent = permanent.unwrap_or(false);
    let path =
        resolve_existing_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;
    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(buf))
}

// ---- EXISTS ----
#[tauri::command]
pub fn dtr_fs_exists(
    app: AppHandle,
    rel: String,
    permanent: Option<bool>,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, rel is resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<bool, String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let permanent = permanent.unwrap_or(false);
    let path = resolve_any_scoped(&app, &rel, permanent, &window_label, &plugin_storage_module)?;
    Ok(path.exists())
}

// ---- MOVE ----
#[tauri::command]
#[allow(clippy::too_many_arguments)] // IPC arguments
pub fn dtr_fs_move(
    app: AppHandle,
    src: String,
    dest: String,
    permanent: Option<bool>,
    create_dirs: Option<bool>,
    overwrite: Option<bool>,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, src/dest are resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<(), String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let permanent = permanent.unwrap_or(false);
    let src_path =
        resolve_existing_scoped(&app, &src, permanent, &window_label, &plugin_storage_module)?;
    let mut dest_path = resolve_any_scoped(
        &app,
        &dest,
        permanent,
        &window_label,
        &plugin_storage_module,
    )?;
    let create_dirs = create_dirs.unwrap_or(true);
    let overwrite = overwrite.unwrap_or(false);

    let is_dir_target = dest.ends_with('/') || dest.ends_with('\\');

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
#[allow(clippy::too_many_arguments)] // IPC arguments
pub fn dtr_fs_copy(
    app: AppHandle,
    src: String,
    dest: String,
    permanent: Option<bool>,
    recursive: Option<bool>,
    create_dirs: Option<bool>,
    overwrite: Option<bool>,
    window: WebviewWindow,
    window_label: Option<String>,
    // Optional plugin storage scope. When set, src/dest are resolved under
    // _external_modules_storage/<module> instead of data/cache.
    plugin_storage_module: Option<String>,
) -> Result<(), String> {
    let window_label = caller_scope(&window, window_label)?;
    let plugin_storage_module = caller_plugin_storage(&window, plugin_storage_module)?;
    let permanent = permanent.unwrap_or(false);
    let src_path =
        resolve_existing_scoped(&app, &src, permanent, &window_label, &plugin_storage_module)?;
    let mut dest_path = resolve_any_scoped(
        &app,
        &dest,
        permanent,
        &window_label,
        &plugin_storage_module,
    )?;
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
                return Err(format!(
                    "Destination already exists: {}",
                    target.to_string_lossy()
                ));
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
pub fn dtr_fs_clear_cache(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    let scope = caller_scope(&window, None)?;
    let base = base_dir_scoped(&app, false, &scope)?;
    if base.exists() {
        fs::remove_dir_all(&base).map_err(|e| e.to_string())?;
        fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn dtr_fs_clear_data(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    let scope = caller_scope(&window, None)?;
    let data = base_dir_scoped(&app, true, &scope)?;
    let trash = trash_dir_scoped(&app, scope.as_deref())?;

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
pub fn dtr_fs_data_clear_trash(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    let scope = caller_scope(&window, None)?;
    let trash = trash_dir_scoped(&app, scope.as_deref())?;
    if trash.exists() {
        std::fs::remove_dir_all(&trash).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&trash).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn dtr_fs_data_recover_trash(
    app: AppHandle,
    window: WebviewWindow,
    trash_rel_path: String,
) -> Result<(), String> {
    let scope = caller_scope(&window, None)?;
    let data = base_dir_scoped(&app, true, &scope)?;
    let trash = trash_dir_scoped(&app, scope.as_deref())?;

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

#[tauri::command]
pub fn dtr_fs_paths(app: AppHandle, window: WebviewWindow) -> Result<FsPaths, String> {
    let scope = caller_scope(&window, None)?;
    let cache = base_dir_scoped(&app, false, &scope)?;
    let data = base_dir_scoped(&app, true, &scope)?;
    Ok(FsPaths {
        cache: cache.to_string_lossy().into_owned(),
        data: data.to_string_lossy().into_owned(),
    })
}

/* =========================
TRASH: helpers and commands
========================= */

fn unique_with_suffix(dest: PathBuf, suffix: &str) -> PathBuf {
    if !dest.exists() {
        return dest;
    }

    let parent = dest
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let stem = dest
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = dest.extension().map(|e| e.to_string_lossy().to_string());

    let base_name = if stem.is_empty() {
        dest.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "item".into())
    } else {
        stem.clone()
    };

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

// Moves a relative path from scoped data into the matching scoped trash.
// If the destination already exists, a "(deleted)" suffix is added.
fn move_rel_in_data_to_trash(
    app: &AppHandle,
    rel: &str,
    window_label: Option<&str>,
) -> Result<(), String> {
    let data_base = if let Some(label) = window_label {
        base_dir_scoped(app, true, &Some(label.to_string()))?
    } else {
        ensure_base_exists(app, true)?
    };

    let src = safe_join(&data_base, rel)?;
    if !src.exists() {
        return Ok(());
    }

    let trash = trash_dir_scoped(app, window_label)?;
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

// ---- LIST DIR in _trash ----
#[tauri::command]
pub fn dtr_fs_trash_list_dir(
    app: AppHandle,
    window: WebviewWindow,
    rel: String,
) -> Result<Vec<FsEntry>, String> {
    let scope = caller_scope(&window, None)?;
    let trash = trash_dir_scoped(&app, scope.as_deref())?;
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
        out.push(FsEntry {
            name,
            path: path_str,
            is_dir,
            size,
        });
    }
    Ok(out)
}

// Stat in _trash
#[tauri::command]
pub fn dtr_fs_trash_stat(
    app: AppHandle,
    window: WebviewWindow,
    rel: String,
) -> Result<FsEntry, String> {
    let scope = caller_scope(&window, None)?;
    let trash = trash_dir_scoped(&app, scope.as_deref())?;
    let p = safe_join(&trash, &rel)?;
    if !p.exists() {
        return Err("No such file or directory in trash".into());
    }
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

// Exists in _trash
#[tauri::command]
pub fn dtr_fs_trash_exists(
    app: AppHandle,
    window: WebviewWindow,
    rel: String,
) -> Result<bool, String> {
    let scope = caller_scope(&window, None)?;
    let trash = trash_dir_scoped(&app, scope.as_deref())?;
    let p = safe_join(&trash, &rel)?;
    Ok(p.exists())
}

// Read text in _trash
#[tauri::command]
pub fn dtr_fs_trash_read_text(
    app: AppHandle,
    window: WebviewWindow,
    rel: String,
) -> Result<String, String> {
    let scope = caller_scope(&window, None)?;
    let trash = trash_dir_scoped(&app, scope.as_deref())?;
    let p = safe_join(&trash, &rel)?;
    if !p.exists() {
        return Err("No such file or directory in trash".into());
    }
    if p.is_dir() {
        return Err("Path is a directory".into());
    }
    std::fs::read_to_string(p).map_err(|e| e.to_string())
}

// Read bytes (base64) in _trash
#[tauri::command]
pub fn dtr_fs_trash_read_bytes(
    app: AppHandle,
    window: WebviewWindow,
    rel: String,
) -> Result<String, String> {
    let scope = caller_scope(&window, None)?;
    let trash = trash_dir_scoped(&app, scope.as_deref())?;
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
DIAGNOSTICS: helpers and commands
========================= */

// Safe join relative to _diagnostics
pub fn dtr_fs_safe_join_diagnostics(app: &AppHandle, rel: &str) -> Result<PathBuf, String> {
    let base = diagnostics_dir(app)?;
    safe_join(&base, rel)
}

// Moves a relative path from _diagnostics into _trash.
// If the destination already exists, a "(deleted)" suffix is added.
fn move_rel_in_diagnostics_to_trash(app: &AppHandle, rel: &str) -> Result<(), String> {
    let diag_base = diagnostics_dir(app)?;
    let src = safe_join(&diag_base, rel)?;
    if !src.exists() {
        return Ok(());
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
pub fn dtr_fs_diagnostics_list_dir(app: AppHandle, rel: String) -> Result<Vec<FsEntry>, String> {
    let dir = dtr_fs_safe_join_diagnostics(&app, &rel)?;
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
        out.push(FsEntry {
            name,
            path: path_str,
            is_dir,
            size,
        });
    }
    Ok(out)
}

// --- stat in _diagnostics ---
#[tauri::command]
pub fn dtr_fs_diagnostics_stat(app: AppHandle, rel: String) -> Result<FsEntry, String> {
    let p = dtr_fs_safe_join_diagnostics(&app, &rel)?;
    if !p.exists() {
        return Err("No such file or directory in _diagnostics".into());
    }
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

// --- write text in _diagnostics ---
#[tauri::command]
pub fn dtr_fs_diagnostics_write_text(
    app: AppHandle,
    rel: String,
    contents: String,
    create_dirs: Option<bool>,
    append: Option<bool>,
) -> Result<(), String> {
    let path = dtr_fs_safe_join_diagnostics(&app, &rel)?;
    if create_dirs.unwrap_or(true) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let mut file = if append.unwrap_or(false) {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
    } else {
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
    }
    .map_err(|e| e.to_string())?;
    file.write_all(contents.as_bytes())
        .map_err(|e| e.to_string())
}

// --- read text in _diagnostics ---
#[tauri::command]
pub fn dtr_fs_diagnostics_read_text(app: AppHandle, rel: String) -> Result<String, String> {
    let path = dtr_fs_safe_join_diagnostics(&app, &rel)?;
    if !path.exists() {
        return Err("No such file or directory in _diagnostics".into());
    }
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// --- read bytes base64 in _diagnostics ---
#[tauri::command]
pub fn dtr_fs_diagnostics_read_bytes(app: AppHandle, rel: String) -> Result<String, String> {
    let path = dtr_fs_safe_join_diagnostics(&app, &rel)?;
    if !path.exists() {
        return Err("No such file or directory in _diagnostics".into());
    }
    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(buf))
}

// --- rm relative in _diagnostics (moves to _trash, not permanent) ---
#[tauri::command]
pub fn dtr_fs_diagnostics_rm(app: AppHandle, rel: String, recursive: bool) -> Result<(), String> {
    let p = dtr_fs_safe_join_diagnostics(&app, &rel)?;
    if !p.exists() {
        return Ok(());
    }
    if p.is_dir() && !recursive {
        let is_empty = std::fs::read_dir(&p)
            .map_err(|e| e.to_string())?
            .next()
            .is_none();
        if !is_empty {
            return Err("Directory not empty".into());
        }
    }
    move_rel_in_diagnostics_to_trash(&app, &rel)
}

// --- clear all _diagnostics (moves everything to _trash) ---
#[tauri::command]
pub fn dtr_fs_diagnostics_clear(app: AppHandle) -> Result<(), String> {
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

// Exists in _diagnostics
#[tauri::command]
pub fn dtr_fs_diagnostics_exists(app: AppHandle, rel: String) -> Result<bool, String> {
    let diag = diagnostics_dir(&app)?;
    let p = safe_join(&diag, &rel)?;
    Ok(p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPANION: &str = "dtr-cache-only-window-0b6e2f3a";

    #[test]
    fn main_windows_use_the_main_scope() {
        assert_eq!(scope_for_label("main", None), Ok(None));
        assert_eq!(scope_for_label("settings", None), Ok(None));
    }

    #[test]
    fn main_windows_cannot_select_another_scope() {
        assert!(scope_for_label("main", Some(COMPANION.into())).is_err());
        assert!(scope_for_label("settings", Some("settings".into())).is_err());
    }

    #[test]
    fn companion_windows_are_confined_to_their_own_scope() {
        assert_eq!(scope_for_label(COMPANION, None), Ok(Some(COMPANION.into())));
        assert_eq!(
            scope_for_label(COMPANION, Some(COMPANION.into())),
            Ok(Some(COMPANION.into()))
        );
        assert!(scope_for_label(COMPANION, Some("dtr-cache-only-window-other".into())).is_err());
        assert!(scope_for_label(COMPANION, Some("main".into())).is_err());
    }

    #[test]
    fn plugin_storage_is_denied_to_companion_windows() {
        assert_eq!(
            plugin_storage_for_label("main", Some("math.wasm".into())),
            Ok(Some("math.wasm".into()))
        );
        assert!(plugin_storage_for_label(COMPANION, Some("math.wasm".into())).is_err());
        assert_eq!(plugin_storage_for_label(COMPANION, None), Ok(None));
    }

    #[test]
    fn safe_join_rejects_traversal_and_absolute_paths() {
        let base = Path::new("/base");
        assert_eq!(safe_join(base, "a/b/../c"), Ok(PathBuf::from("/base/a/c")));
        assert!(safe_join(base, "../outside").is_err());
        assert!(safe_join(base, "a/../../outside").is_err());
        assert!(safe_join(base, "/etc/passwd").is_err());
    }

    #[test]
    fn scope_names_reject_reserved_and_path_like_labels() {
        assert_eq!(scope_dir_name(None), Ok(".main".into()));
        assert_eq!(scope_dir_name(Some(COMPANION)), Ok(format!(".{COMPANION}")));
        for label in ["", "  ", "main", "..", "../main", "a/b", "a\\b", "a.b"] {
            assert!(scope_dir_name(Some(label)).is_err(), "accepted {label:?}");
        }
    }

    #[test]
    fn plugin_storage_modules_cannot_escape_the_storage_root() {
        assert_eq!(
            validate_plugin_storage_module(" math.wasm "),
            Ok("math.wasm".into())
        );
        for module in [
            "",
            "math",
            "../math.wasm",
            "sub/math.wasm",
            "sub\\math.wasm",
            "/abs.wasm",
        ] {
            assert!(
                validate_plugin_storage_module(module).is_err(),
                "accepted {module:?}"
            );
        }
    }
}

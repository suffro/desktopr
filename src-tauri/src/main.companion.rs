#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod bubbledesk;
mod helpers;

use bridge::*;
use bubbledesk::bridge;
use tauri::{Manager, WindowBuilder, WindowUrl, AppHandle};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use base64;
use dirs;

// --- Helper function to show error window ---
fn show_error_window(app: &AppHandle, message: &str) {
    // Check if error window already exists
    if app.get_window("companion_error").is_some() {
        return;
    }
    let html_content = format!(
        r#"
        <html>
            <head><title>Errore Bubbledesk Companion</title></head>
            <body style="font-family:sans-serif; padding:20px; background:#f8d7da; color:#721c24;">
                <h2>Errore Bubbledesk Companion</h2>
                <pre style="white-space: pre-wrap;">{}</pre>
            </body>
        </html>
        "#,
        message
    );
    let window = WindowBuilder::new(app, "companion_error", WindowUrl::App("about:blank".into()))
        .title("Errore Bubbledesk Companion")
        .inner_size(420.0, 260.0)
        .build();
    if let Ok(win) = window {
        let _ = win.eval(&format!(
            "document.documentElement.innerHTML = `{}`",
            html_content.replace('`', "\\`")
        ));
    }
}

// --- Configurazione runtime ricevuta dal builder tramite file temporaneo ---
#[derive(Debug, Deserialize)]
struct CompanionConfig {
    title: Option<String>,
    url: String,
    color: Option<String>,
    logo: Option<String>,
    menu_config_base64: Option<String>,
    fullscreen: Option<bool>,
}

// --- Applica la configurazione base64 o da file ---
#[tauri::command]
fn bd_companion_apply_config(app: AppHandle, token: String) -> Result<(), String> {
    // Percorso della configurazione temporanea (scritta dal builder)
    let path = dirs::data_dir()
        .ok_or_else(|| {
            let err = "Failed to resolve app data dir".to_string();
            show_error_window(&app, &err);
            err
        })?
        .join("Bubbledesk/tmp")
        .join(format!("{}.json", token));

    if !path.exists() {
        let err = format!("Configuration file not found: {:?}", path);
        show_error_window(&app, &err);
        return Err(err);
    }

    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            let err = format!("Failed to read config file: {}", e);
            show_error_window(&app, &err);
            return Err(err);
        }
    };
    let cfg: CompanionConfig = match serde_json::from_str(&contents) {
        Ok(c) => c,
        Err(e) => {
            let err = format!("Invalid config JSON: {}", e);
            show_error_window(&app, &err);
            return Err(err);
        }
    };

    // Costruzione finestra
    let mut builder = WindowBuilder::new(
        &app,
        "companion",
        WindowUrl::External(cfg.url.parse().map_err(|e| {
            let err = e.to_string();
            show_error_window(&app, &err);
            err
        })?),
    )
    .title(cfg.title.unwrap_or_else(|| "Preview".into()))
    .fullscreen(cfg.fullscreen.unwrap_or(false));

    // --- Applicazione menu dinamico ---
    if let Some(menu_b64) = cfg.menu_config_base64 {
        let decoded = match base64::decode(menu_b64) {
            Ok(d) => d,
            Err(e) => {
                let err = e.to_string();
                eprintln!("[companion] Failed to decode menu base64: {}", err);
                show_error_window(&app, &err);
                return Err(err);
            }
        };
        let json = match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(e) => {
                let err = e.to_string();
                eprintln!("[companion] Failed to parse menu JSON: {}", err);
                show_error_window(&app, &err);
                return Err(err);
            }
        };
        if let Err(e) = crate::bridge::menu::bd_apply_menu_json(app.clone(), json, Some(false)) {
            eprintln!("[companion] Failed to apply menu: {e}");
            show_error_window(&app, &format!("Failed to apply menu: {e}"));
        }
    }

    // --- (opzionale) Colori o logo ---
    if let Some(color) = cfg.color {
        println!("[companion] Applying window color: {}", color);
        // Potresti usarlo per tema o accent color del frontend
    }

    if let Some(logo) = cfg.logo {
        println!("[companion] Logo path: {}", logo);
        // Potresti caricarlo nel webview come overlay
    }

    if let Err(e) = builder.build() {
        let err = e.to_string();
        show_error_window(&app, &err);
        return Err(err);
    }

    // Cleanup file temporaneo (opzionale)
    let _ = fs::remove_file(&path);

    Ok(())
}

// --- Entry point principale Companion ---
fn main() {
    let mut builder = tauri::Builder::default();

    // Stessi plugin e bridge del builder principale
    builder = builder
        .plugin(bridge())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(bridge::autostart::init_plugin())
        .plugin(tauri_plugin_deep_link::init());

    builder = builder.setup(|app| {
        // Registrazione runtime (solo dev su Win/Linux)
        #[cfg(any(target_os = "linux", all(debug_assertions, target_os = "windows")))]
        {
            app.deep_link().register_all()?;
        }

        // Deep link iniziale (quando Companion viene avviato)
        if let Some(urls) = app.deep_link().get_current()? {
            if let Some(u) = urls.first() {
                if let Some(token) = u.strip_prefix("bubbledesk-companion://launch/") {
                    println!("[companion] Received token: {}", token);
                    let app_handle = app.handle().clone();
                    match bd_companion_apply_config(app_handle.clone(), token.to_string()) {
                        Ok(_) => {}
                        Err(e) => {
                            show_error_window(&app_handle, &e);
                        }
                    }
                }
            }
        }

        // Deep link runtime (quando Companion è già aperto)
        let handle = app.handle().clone();
        app.deep_link().on_open_url(move |e| {
            if let Some(u) = e.urls().first() {
                if let Some(token) = u.strip_prefix("bubbledesk-companion://launch/") {
                    println!("[companion] Received token (runtime): {}", token);
                    let handle = handle.clone();
                    tauri::async_runtime::spawn(async move {
                        match bd_companion_apply_config(handle.clone(), token.to_string()) {
                            Ok(_) => {}
                            Err(e) => {
                                show_error_window(&handle, &e);
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    });

    builder
        .invoke_handler(tauri::generate_handler![
            // tutte le API bridge normali + quella del companion
            bd_notification_state, bd_request_permission, bd_notify,
            bd_clipboard_write, bd_clipboard_read,
            bd_file_open, bd_file_save, bd_file_open_with_bytes,
            bd_app_info, bd_app_exit,
            bd_win_minimize, bd_win_maximize, bd_win_fullscreen, bd_win_open, bd_win_close,
            bd_toggle_devtools, bd_open_devtools, bd_close_devtools,
            bd_event_emit, bd_event_emit_to, bd_event_emit_to_current_window,
            bd_fs_list_dir, bd_fs_mkdir, bd_fs_rm, bd_fs_stat, bd_fs_write_text, bd_fs_read_text,
            bd_fs_write_bytes, bd_fs_read_bytes, bd_fs_exists, bd_fs_move, bd_fs_copy,
            bd_menu_set_enabled, bd_menu_set_checked, bd_apply_menu_json,
            bd_menu_set_label, bd_menu_toggle_checked, bd_menu_get_state,
            bd_menu_reload_from_file, bd_menu_list_items, bd_menu_reset,
            bd_menu_disable_all, bd_menu_enable_section, bd_menu_disable_section,
            bd_companion_apply_config, // 👈 nuovo comando companion
        ])
        .run(tauri::generate_context!("tauri.conf.companion.json"))
        .expect("error while running Bubbledesk Companion");
}
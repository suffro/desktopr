#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod bubbledesk;
mod helpers;

use bridge::*;
use bubbledesk::bridge;
use tauri::{WindowEvent, Emitter, DragDropEvent, PhysicalSize, Manager};
use crate::bridge::dragdrop;

// Global Shortcut plugin
use tauri_plugin_global_shortcut as gsc;
use crate::gsc::Builder;
use crate::gsc::ShortcutState;

// Deep-link + single-instance
use tauri_plugin_deep_link::DeepLinkExt;

use tauri_plugin_prevent_default::{
  Builder as PD, Flags, KeyboardShortcut,
  ModifierKey::{CtrlKey, ShiftKey, AltKey, MetaKey}
};


fn main() {
  // let prevent = PD::new()
  // .with_flags(Flags::CONTEXT_MENU | Flags::DEV_TOOLS) // disabilita menu e scorciatoie DevTools
  // .shortcut(KeyboardShortcut::new("F12"))
  // .shortcut(KeyboardShortcut::with_modifiers("I", &[CtrlKey, ShiftKey])) // Ctrl+Shift+I
  // .shortcut(KeyboardShortcut::with_modifiers("I", &[MetaKey, AltKey]))  // ⌘⌥I
  // .build();

  let mut builder = tauri::Builder::default();

  // --- 0) Single-instance PRIMO (importante con deep-link) ---
  builder = builder.plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
    println!("single-instance argv: {argv:?}");
  }));

  // --- 1) Plugin del tuo bridge + altri già presenti ---
  builder = builder
    .plugin(bridge())
    // .plugin(prevent)
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init());

  // --- 2) Plugin Deep Link ---
  builder = builder.plugin(tauri_plugin_deep_link::init());

  // --- 3) Setup: tray, shortcut e deeplink (boot + runtime) ---
  builder = builder.setup(|app| {
    // Menu nativo
    crate::bridge::menu::init_menu(app)?;
    
    let version = app.package_info().version.to_string();
    start_heartbeat(app.handle().clone());
    // panic hook
    install_panic_hook(app.handle().clone(), version);
    // retention all’avvio
    let _ = db_logs_run_retention(app.handle().clone());

    // Global Shortcut
    app.handle().plugin(
      gsc::Builder::new().build(),
    )?;

    // Registrazione runtime (solo dev su Win/Linux)
    #[cfg(any(target_os = "linux", all(debug_assertions, target_os = "windows")))]
    {
      app.deep_link().register_all()?;
    }

    // Deep link: URL di avvio
    let start_urls = app.deep_link().get_current()?;
    if let Some(urls) = start_urls {
      if let Some(u) = urls.first() {
        crate::bridge::deeplink::emit_parsed_deeplink(&app.handle(), u.as_str());
      }
    }

    // Deep link: URL runtime (quando l’app è già aperta)
    let handle = app.handle().clone();
    app.deep_link().on_open_url(move |e| {
      if let Some(u) = e.urls().first() {
        crate::bridge::deeplink::emit_parsed_deeplink(&handle, u.as_str());
      }
    });

    Ok(())
  });


  // --- 4) Eventi finestra + invoke handler ---
  builder
    .on_window_event(|window, event| {
      match event {
        WindowEvent::Focused(true)  => { let _ = window.emit("window:focus",  ()); }
        WindowEvent::Focused(false) => { let _ = window.emit("window:blur",   ()); }
        WindowEvent::CloseRequested { api, .. } => {
          // blocca chiusura immediata
          api.prevent_close();
          // emette evento
          let _ = window.emit("window:close-requested", ());
          // marca clean shutdown centralmente
          mark_clean_shutdown_now(&window.app_handle());
          // chiudi davvero ora
          let _ = window.close();
        }
        WindowEvent::Resized(size) => {
          let _ = window.emit("window:resized", Some(serde_json::json!({
            "width": size.width,
            "height": size.height
          })));
        }
        WindowEvent::DragDrop(e) => {
          match e {
            DragDropEvent::Enter { paths, position } => {
              dragdrop::emit_enter(window, &paths, position.x, position.y);
            }
            DragDropEvent::Over { position } => {
              dragdrop::emit_over(window, position.x, position.y);
            }
            DragDropEvent::Drop { paths, position } => {
              dragdrop::emit_drop(window, &paths, position.x, position.y);
            }
            DragDropEvent::Leave => {
              dragdrop::emit_cancel(window);
            }
            _ => {}
          }
        }
        _ => {}
      }
    })
    .invoke_handler(tauri::generate_handler![
      // notifications
      bd_notification_state, bd_request_permission, bd_notify,
      // clipboard
      bd_clipboard_write, bd_clipboard_read,
      // files
      bd_file_open, bd_file_save,
      // app info
      bd_app_info,
      // window
      bd_win_minimize, bd_win_maximize, bd_win_fullscreen, bd_win_open, bd_win_close,
      bd_toggle_devtools, bd_open_devtools, bd_close_devtools,
      // events
      bd_event_emit, bd_event_emit_to,
      // fs
      db_fs_list_dir, db_fs_mkdir, db_fs_rm, db_fs_stat, db_fs_write_text, db_fs_read_text,
      db_fs_write_bytes, db_fs_read_bytes, db_fs_exists, db_fs_move, db_fs_copy,
      db_fs_clear_cache, db_fs_paths,
      // menu
      bd_menu_set_enabled, bd_menu_set_checked,
      // diagnostics
      db_logs_get_privacy, db_logs_set_privacy, db_logs_run_retention, db_logs_list_files, db_logs_read_file,
      db_logs_record_js_error, db_logs_new_record, db_logs_export_zip
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}




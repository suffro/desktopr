#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod Bubbledesk;

use bridge::*;
use bridge::tray::init_tray;
use Bubbledesk::bridge;
use tauri::{WindowEvent, Emitter, DragDropEvent, PhysicalSize, Manager};
use crate::bridge::dragdrop;

// Global Shortcut plugin
use tauri_plugin_global_shortcut as gsc;
use crate::gsc::Builder;
use crate::gsc::ShortcutState;

// Deep-link + single-instance
use tauri_plugin_deep_link::DeepLinkExt;

fn main() {
  let mut builder = tauri::Builder::default();

  // --- 0) Single-instance PRIMO (importante con deep-link) ---
  builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
    // Quando un deep-link tenta di aprire una seconda istanza,
    // il plugin deep-link si occuperà di inoltrare l'URL all'istanza esistente.
    // Qui puoi eventualmente loggare argv (debug).
    println!("single-instance argv: {argv:?}");
  }));

  // --- 1) Plugin del tuo bridge + altri già presenti ---
  builder = builder
    .plugin(bridge())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init());

  // --- 2) Plugin Deep Link ---
  builder = builder.plugin(tauri_plugin_deep_link::init());

  // --- 3) Setup: tray, shortcut e deeplink (boot + runtime) ---
  builder = builder.setup(|app| {
    // Global Shortcut (già tuo)
    app.handle().plugin(
      gsc::Builder::new().build(),
    )?;

    // Tray (già tuo)
    init_tray(app)?;

    // Deep link: URL di avvio (se l'app è stata aperta con deeplink)
    if let Ok(Some(urls)) = app.deep_link().get_current() {
      if let Some(u) = urls.first() {
        bridge_deeplink::emit_parsed(app, u);
      }
    }

    // Deep link: URL runtime (quando l'app è già aperta)
    app.deep_link().on_open_url(|e| {
      if let Some(u) = e.urls().first() {
        bridge_deeplink::emit_parsed(e.app_handle(), u);
      }
    });

    Ok(())
  });

  // --- 4) Eventi finestra (come già avevi) + invoke handler ---
  builder
    .on_window_event(|window, event| {
      match event {
        WindowEvent::Focused(true)  => { let _ = window.emit("window:focus",  ()); }
        WindowEvent::Focused(false) => { let _ = window.emit("window:blur",   ()); }
        WindowEvent::CloseRequested { .. } => { let _ = window.emit("window:close-requested", ()); }
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
      bd_win_minimize, bd_win_maximize, bd_win_fullscreen,
      // events
      bd_event_emit, bd_event_emit_to,
      // fs
      fs_list_dir, fs_mkdir, fs_rm, fs_stat, fs_write_text, fs_read_text, fs_write_bytes, fs_read_bytes, fs_exists, fs_move, fs_copy, fs_clear_cache, fs_paths
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

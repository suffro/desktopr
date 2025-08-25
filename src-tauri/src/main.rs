#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod bubbledesk;

use bridge::*;
use bridge::tray::init_tray;
use bubbledesk::bridge;
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
  builder = builder.plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
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
      fs_list_dir, fs_mkdir, fs_rm, fs_stat, fs_write_text, fs_read_text,
      fs_write_bytes, fs_read_bytes, fs_exists, fs_move, fs_copy,
      fs_clear_cache, fs_paths
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

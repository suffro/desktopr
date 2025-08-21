#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod plugin_bubbledesk;

use bridge::*;
use bridge::tray::init_tray;
use plugin_bubbledesk::bubbledesk_plugin;
use tauri::{WindowEvent, Emitter, DragDropEvent, PhysicalSize}; // <-- IMPORTA Emitter
use crate::bridge::dragdrop;
use tauri_plugin_global_shortcut as gsc;
use crate::gsc::Builder;
use crate::gsc::ShortcutState;

fn main() {
  tauri::Builder::default()
    .plugin(bubbledesk_plugin())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
    .setup(|app| {
      // 1) Global Shortcut plugin (stile v2: Builder)
      app.handle().plugin(
        gsc::Builder::new()
          .build(),
      )?;

      // 2) Tray
      init_tray(app)?;

      Ok(())
    })
    .on_window_event(|window, event| {
      match event {
        WindowEvent::Focused(true)  => { let _ = window.emit("window:focus",  ()); }
        WindowEvent::Focused(false) => { let _ = window.emit("window:blur",   ()); }
        WindowEvent::CloseRequested { .. } => { let _ = window.emit("window:close-requested", ()); }
        WindowEvent::Resized(size) => {
          // qui hai accesso a PhysicalSize
          let _ = window.emit("window:resized", Some(serde_json::json!({
            "width": size.width,
            "height": size.height
          })));
        }
        // --- Drag & Drop ---
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
      fs_list_dir, fs_mkdir, fs_rm, fs_stat
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

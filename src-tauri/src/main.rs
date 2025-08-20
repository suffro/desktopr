#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod plugin_bubbledesk;

use bridge::*;
use plugin_bubbledesk::bubbledesk_plugin;

fn main() {
  tauri::Builder::default()
    .plugin(bubbledesk_plugin())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_dialog::init())
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
      bd_win_minimize, bd_win_maximize, bd_win_fullscreen
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

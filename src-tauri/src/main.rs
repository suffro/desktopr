mod plugin_bubbledesk;
mod commands;

use plugin_bubbledesk::init;

fn main() {
  tauri::Builder::default()
  .plugin(plugin_bubbledesk::init())
  .plugin(tauri_plugin_notification::init())
  .invoke_handler(tauri::generate_handler![
    commands::bd_notify,
    commands::bd_clipboard_write,
  ])
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
}

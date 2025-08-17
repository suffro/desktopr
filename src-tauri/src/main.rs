mod plugin_bubbledesk;
mod commands;

use plugin_bubbledesk::bubbledesk_plugin;

fn main() {
  tauri::Builder::default()
    .plugin(bubbledesk_plugin())
    .invoke_handler(tauri::generate_handler![
      commands::bd_notify,
      commands::bd_clipboard_write,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

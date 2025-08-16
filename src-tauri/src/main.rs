#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Emitter};

#[cfg(feature = "notification")]
fn init_notification(app: &tauri::App) {
  app.handle().plugin(tauri_plugin_notification::init());
}
#[cfg(not(feature = "notification"))]
fn init_notification(_: &tauri::App) {}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      init_notification(app);

      if let Some(w) = app.get_webview_window("main") {
        let bridge = include_str!("../../bridge/sdk-bridge.js");
        w.eval(bridge)?;
      }

      app.emit("bubbledesk:ready", "ok").ok();
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

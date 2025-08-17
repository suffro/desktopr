// All comments in English as per your preference
use tauri::plugin::{Builder as PluginBuilder, TauriPlugin};
use tauri::{Runtime};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  PluginBuilder::new("bubbledesk")
    // This runs at document start for every navigation/reload
    .js_init_script(include_str!("bubbledesk_init.js"))
    .build()
}

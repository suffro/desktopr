use tauri::plugin::{Builder as PluginBuilder, TauriPlugin};
use tauri::Runtime;

pub fn bubbledesk_plugin<R: Runtime>() -> TauriPlugin<R> {
  PluginBuilder::new("bubbledesk")
    .js_init_script(include_str!("bubbledesk_init.js"))
    .build()
}

use tauri::plugin::{Builder as PluginBuilder, TauriPlugin};
use tauri::Runtime;
use crate::helpers::constants::BRIDGE_JS;

pub fn bridge<R: Runtime>() -> TauriPlugin<R> {
  PluginBuilder::new("bubbledesk")
    .js_init_script(include_str!("../tsc/bridge.js"))
    .build()
}

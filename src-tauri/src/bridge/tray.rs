// src/bridge/tray.rs
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, // <-- IMPORTA Emitter
    Manager,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// in cima al file (o in uno scope statico)
static LAST_CLICK_MS: AtomicU64 = AtomicU64::new(0);

fn debounce(ms: u64) -> bool {
  let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
  let last = LAST_CLICK_MS.load(Ordering::Relaxed);
  if now.saturating_sub(last) < ms {
    return true; // skip
  }
  LAST_CLICK_MS.store(now, Ordering::Relaxed);
  false
}


pub fn init_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "tray.show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "tray.hide", "Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray.quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    // src/bridge/tray.rs
TrayIconBuilder::new()
.icon(app.default_window_icon().unwrap().clone())
.menu(&menu)
.show_menu_on_left_click(false) // ⟵ importante: lasciamo passare l'evento
.on_menu_event(|app, ev| {
  let id = ev.id.0.as_str();
  let _ = app.emit("tray:menu", Some(serde_json::json!({ "id": id })));
  match id {
    "tray.show" => {
      if let Some(win) = app.get_webview_window("main") {
        if let Ok(true) = win.is_visible() {
            let _ = win.show();
            let _ = win.set_focus();
            let _ = win.unminimize();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
      }
    }
    "tray.hide" => {
      if let Some(win) = app.get_webview_window("main") {
        let _ = win.minimize();
        let _ = win.hide();
      }
    }
    "tray.quit" => app.exit(0),
    _ => {}
  }
})
.on_tray_icon_event(|tray, ev| {
  use tauri::{Emitter};
  use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
  
  let (id, position, rect) = match &ev {
    TrayIconEvent::Click       { id, position, rect, .. }
    | TrayIconEvent::DoubleClick { id, position, rect, .. }
    | TrayIconEvent::Enter     { id, position, rect, .. }
    | TrayIconEvent::Move      { id, position, rect, .. }
    | TrayIconEvent::Leave     { id, position, rect, .. } => (id.clone(), *position, *rect),
    _ => return, // altre varianti future: esci o gestiscile qui
  };
  let id_str = id.0.as_str();
  let formatted_id = format!("tray.icon.{}",id_str);
  match ev {
    // ora questo arriva anche su macOS
    TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Down, .. } => {
        if debounce(120) { return; }
        let _ = tray.app_handle().emit("tray:icon", Some(serde_json::json!({
        "id": formatted_id, "type": "click", "button": "left", "state": "down", "position": position, "rect": rect
      })));

      // se vuoi anche mostrare la finestra al click:
      if let Some(win) = tray.app_handle().get_webview_window("main") {
        if let Ok(true) = win.is_visible() {
            if let Ok(true) = win.is_focused(){
                let _ = win.minimize();
            } else {
                let _ = win.set_focus();
                let _ = win.unminimize();
            }
        } else {
            let _ = win.show();
        }
      }
    }
    TrayIconEvent::Click { button: MouseButton::Right, button_state: MouseButtonState::Down, .. } => {
        if debounce(120) { return; }
        let _ = tray.app_handle().emit("tray:icon", Some(serde_json::json!({
          "id": formatted_id, "type": "click", "button": "right", "state": "down", "position": position, "rect": rect
      })));
    }
    TrayIconEvent::Click { button: MouseButton::Middle, button_state: MouseButtonState::Down, .. } => {
        if debounce(120) { return; }
        let _ = tray.app_handle().emit("tray:icon", Some(serde_json::json!({
          "id": formatted_id, "type": "click", "button": "middle", "state": "down", "position": position, "rect": rect
        })));
      }
    TrayIconEvent::Enter { .. } => {
      if debounce(120) { return; }
      let _ = tray.app_handle().emit("tray:icon", Some(serde_json::json!({
        "id": formatted_id, "type": "enter", "position": position, "rect": rect
      })));
    }
    TrayIconEvent::Leave { .. } => {
      if debounce(120) { return; }
      let _ = tray.app_handle().emit("tray:icon", Some(serde_json::json!({
        "id": formatted_id, "type": "leave", "position": position, "rect": rect
      })));
    }
    _ => {}
  }
})
.build(app)?;


    Ok(())
}

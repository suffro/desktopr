// src/bridge/tray.rs
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, // <-- IMPORTA Emitter
    Manager,
};

pub fn init_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
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
    "show" => {
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
    "hide" => {
      if let Some(win) = app.get_webview_window("main") {
        let _ = win.minimize();
        let _ = win.hide();
      }
    }
    "quit" => app.exit(0),
    _ => {}
  }
})
.on_tray_icon_event(|tray, ev| {
  use tauri::{Emitter};
  use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};

  match ev {
    // ora questo arriva anche su macOS
    TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Down, .. } => {
      let _ = tray.app_handle().emit("tray:click", Some(serde_json::json!({
        "type": "click", "button": "left", "state": "down"
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
      let _ = tray.app_handle().emit("tray:click", Some(serde_json::json!({
        "type": "click", "button": "right", "state": "down"
      })));
    }
    _ => {}
  }
})
.build(app)?;


    Ok(())
}

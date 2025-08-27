// src/bridge/menu.rs
use tauri::{
  App, AppHandle, Manager,
  menu::{Menu, MenuItem, Submenu, PredefinedMenuItem, MenuItemKind},
};

pub fn init_menu(app: &App) -> tauri::Result<()> {
  // --- File ---
  let file_menu = Menu::new(app)?
    .add_item(MenuItem::new(app, "New Window", true, Some("CmdOrCtrl+N"))?)?
    .add_item(MenuItem::new(app, "Close Window", true, Some("CmdOrCtrl+W"))?)?
    .add_item(MenuItem::new(app, "Quit", true, Some("CmdOrCtrl+Q"))?)?;

  // --- Edit (predefiniti di sistema) ---
  let edit_menu = Menu::new(app)?
    .add_item(PredefinedMenuItem::undo(app, None)?)?
    .add_item(PredefinedMenuItem::redo(app, None)?)?
    .add_native_item(PredefinedMenuItem::separator(app)?)?
    .add_item(PredefinedMenuItem::cut(app, None)?)?
    .add_item(PredefinedMenuItem::copy(app, None)?)?
    .add_item(PredefinedMenuItem::paste(app, None)?)?
    .add_item(PredefinedMenuItem::select_all(app, None)?)?;

  // --- View ---
  let view_menu = Menu::new(app)?
    .add_item(MenuItem::new(app, "Reload", true, Some("CmdOrCtrl+R"))?)?
    .add_item(MenuItem::new(app, "Toggle Fullscreen", true, Some("F11"))?)?
    .add_native_item(PredefinedMenuItem::separator(app)?)?
    .add_item(MenuItem::new(app, "Toggle DevTools", true, Some("CmdOrCtrl+Alt+I"))?)?;

  // --- Window ---
  let window_menu = Menu::new(app)?
    .add_item(PredefinedMenuItem::minimize(app, None)?)?
    .add_item(PredefinedMenuItem::zoom(app, None)?)?;

  // --- Help ---
  let help_menu = Menu::new(app)?
    .add_item(MenuItem::new(app, "Learn More", true, None::<&str>)?)?;

  // --- Root menu ---
  let mut root = Menu::new(app)?;

  // App submenu (macOS): About/Services/Hide...
  #[cfg(target_os = "macos")]
  {
    let app_sub = Menu::new(app)?
      .add_item(PredefinedMenuItem::about(app, None, None)?)?
      .add_native_item(PredefinedMenuItem::separator(app)?)?
      .add_item(PredefinedMenuItem::services(app, None)?)?
      .add_native_item(PredefinedMenuItem::separator(app)?)?
      .add_item(PredefinedMenuItem::hide(app, None)?)?
      .add_item(PredefinedMenuItem::hide_others(app, None)?)?
      .add_item(PredefinedMenuItem::show_all(app, None)?)?
      .add_native_item(PredefinedMenuItem::separator(app)?)?
      .add_item(PredefinedMenuItem::quit(app, None)?)?;

    root = root.add_submenu(Submenu::new(app, "Bubbledesk", true, app_sub)?)?;
  }

  let root = root
    .add_submenu(Submenu::new(app, "File", true, file_menu)?)?
    .add_submenu(Submenu::new(app, "Edit", true, edit_menu)?)?
    .add_submenu(Submenu::new(app, "View", true, view_menu)?)?
    .add_submenu(Submenu::new(app, "Window", true, window_menu)?)?
    .add_submenu(Submenu::new(app, "Help", true, help_menu)?)?;

  app.set_menu(root)?;

  // Forward eventi menu → event bus ("menu:click" con id String)
  app.on_menu_event(|app, ev| {
    let id: String = ev.id().0.clone();
    let _ = app.emit("menu:click", id);
  });

  Ok(())
}

// Utility: abilita/disabilita entry per ID
fn set_enabled(app: &AppHandle, id: &str, enabled: bool) -> tauri::Result<()> {
  if let Some(item) = app.menu().as_ref().and_then(|m| m.get(id)) {
    match item {
      MenuItemKind::MenuItem(mi) => mi.set_enabled(enabled)?,
      MenuItemKind::Predefined(mi) => mi.set_enabled(enabled)?,
      MenuItemKind::Submenu(_) => { /* ignore or handle differently */ }
    }
  }
  Ok(())
}

// Utility: set checked (solo se l’item è checkable)
fn set_checked(app: &AppHandle, id: &str, checked: bool) -> tauri::Result<()> {
  if let Some(item) = app.menu().as_ref().and_then(|m| m.get(id)) {
    match item {
      MenuItemKind::MenuItem(mi) => mi.set_checked(checked)?,
      MenuItemKind::Predefined(mi) => mi.set_checked(checked)?,
      MenuItemKind::Submenu(_) => { /* ignore */ }
    }
  }
  Ok(())
}


  // Comandi di utilità per controllare il menu dal renderer
#[tauri::command]
pub fn bd_menu_set_enabled(app: tauri::AppHandle, id: String, enabled: bool) -> Result<(), String> {
  set_enabled(&app, &id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_menu_set_checked(app: tauri::AppHandle, id: String, checked: bool) -> Result<(), String> {
  set_checked(&app, &id, checked).map_err(|e| e.to_string())
}
  
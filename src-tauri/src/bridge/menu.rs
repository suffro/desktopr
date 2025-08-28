use tauri::{
  App, AppHandle, Manager, Emitter,
  menu::{Menu, Submenu, MenuItem, CheckMenuItem, PredefinedMenuItem, MenuItemKind},
};

pub fn init_menu(app: &App) -> tauri::Result<()> {
  // File
  let file = Submenu::with_items(app, "File", true, &[
    &MenuItem::with_id(app, "file.new-window", "New Window", true, Some("CmdOrCtrl+N"))?,
    &PredefinedMenuItem::close_window(app, Some("Close Window"))?,
    &PredefinedMenuItem::quit(app, Some("Quit"))?,
  ])?;

  // Edit (predefiniti)
  let edit = Submenu::with_items(app, "Edit", true, &[
    &PredefinedMenuItem::undo(app, None)?,
    &PredefinedMenuItem::redo(app, None)?,
    &PredefinedMenuItem::separator(app)?,
    &PredefinedMenuItem::cut(app, None)?,
    &PredefinedMenuItem::copy(app, None)?,
    &PredefinedMenuItem::paste(app, None)?,
    &PredefinedMenuItem::select_all(app, None)?,
  ])?;

  // View
  let devtools = CheckMenuItem::with_id(
    app,
    "view.devtools",
    "Toggle DevTools",
    true,   // enabled
    false,  // checked (manca nel tuo errore: è il 5° argomento bool)
    Some("CmdOrCtrl+Alt+I"),
  )?;
  let view = Submenu::with_items(app, "View", true, &[
    &MenuItem::with_id(app, "view.reload", "Reload", true, Some("CmdOrCtrl+R"))?,
    &MenuItem::with_id(app, "view.reload-hard", "Reload (Clear Cache)", true, Some("CmdOrCtrl+Shift+R"))?,
    &PredefinedMenuItem::separator(app)?,
    &PredefinedMenuItem::fullscreen(app, Some("Toggle Full Screen"))?, // niente "zoom" in v2.7
    &devtools,
  ])?;

  // Window
  let window = Submenu::with_items(app, "Window", true, &[
    &PredefinedMenuItem::minimize(app, None)?,
    &PredefinedMenuItem::maximize(app, None)?,
    &PredefinedMenuItem::close_window(app, None)?,
  ])?;

  // Root menu (aggiungi l’App submenu solo su macOS)
  #[cfg(target_os = "macos")]
  {
    let app_sub = Submenu::with_items(app, "Bubbledesk", true, &[
      &PredefinedMenuItem::about(app, Some("About Bubbledesk"), None)?,
      &PredefinedMenuItem::services(app, None)?,
      &PredefinedMenuItem::separator(app)?,
      &PredefinedMenuItem::hide(app, None)?,
      &PredefinedMenuItem::hide_others(app, None)?,
      &PredefinedMenuItem::show_all(app, None)?,
      &PredefinedMenuItem::separator(app)?,
      &PredefinedMenuItem::quit(app, None)?,
    ])?;

    let root = Menu::with_items(app, &[&app_sub, &file, &edit, &view, &window])?;
    app.set_menu(root)?;
  }

  #[cfg(not(target_os = "macos"))]
  {
    let root = Menu::with_items(app, &[&file, &edit, &view, &window])?;
    app.set_menu(root)?;
  }

  // Forward click → "menu:click" (ID come stringa)
  app.on_menu_event(|app, ev| {
    let id = ev.id().as_ref().to_string(); // <-- niente to_string su &MenuId
    let _ = app.emit("menu:click", id);
  });

  Ok(())
}

// Abilita/disabilita (solo MenuItem/Check/Icon; i Predefined non espongono toggle)
pub fn set_enabled(app: &AppHandle, id: &str, enabled: bool) -> tauri::Result<()> {
  if let Some(menu) = app.menu() {
    if let Some(kind) = menu.get(id) {
      match kind {
        MenuItemKind::MenuItem(mi) => mi.set_enabled(enabled)?,
        MenuItemKind::Check(mi)    => mi.set_enabled(enabled)?,
        MenuItemKind::Icon(mi)     => mi.set_enabled(enabled)?,
        _ => { /* Predefined/Submenu: nessuna opzione standard di enable/disable */ }
      }
    }
  }
  Ok(())
}

// Spunta solo per CheckMenuItem
pub fn set_checked(app: &AppHandle, id: &str, checked: bool) -> tauri::Result<()> {
  if let Some(menu) = app.menu() {
    if let Some(MenuItemKind::Check(mi)) = menu.get(id) {
      mi.set_checked(checked)?;
    }
  }
  Ok(())
}

// (facoltativo) comandi esponibili al renderer
#[tauri::command]
pub fn bd_menu_set_enabled(app: AppHandle, id: String, enabled: bool) -> Result<(), String> {
  set_enabled(&app, &id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_menu_set_checked(app: AppHandle, id: String, checked: bool) -> Result<(), String> {
  set_checked(&app, &id, checked).map_err(|e| e.to_string())
}

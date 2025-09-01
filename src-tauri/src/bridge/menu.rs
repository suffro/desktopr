use serde::Serialize;
use tauri::{
  App, AppHandle, Emitter, Manager, Runtime,
  menu::{Menu, MenuItemKind, IsMenuItem, Submenu},
  path::BaseDirectory
};
use std::fs;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::helpers::menu_builder::*;


#[derive(Clone)]
struct MenuMeta {
  section: String,
  parent_label: Option<String>,      // label del submenu genitore (se presente)
  parent_id: Option<String>,         // id del submenu genitore (se presente)
}

struct MenuIndex {
  by_id: HashMap<String, MenuMeta>,
}

struct CheckState {
  map: Mutex<HashMap<String, bool>>,
}

// -------- init --------

pub fn init_menu<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
  let cfg = match parse_menu_config(app) {
    Some(c) => c,
    None => return Ok(()),
  };
  if !cfg.enabled || !is_current_platform_in(&cfg.platforms) {
    return Ok(());
  }

  // 🔎 indice id -> { section, parent }
  let mut index_map: HashMap<String, MenuMeta> = HashMap::new();
  // 🔘 stato check id -> bool
  let mut check_map: HashMap<String, bool> = HashMap::new();

  let mut subs: Vec<Submenu<R>> = Vec::new();

  #[cfg(target_os = "macos")]
  if let Some(sec) = cfg.macos_root.as_ref() {
    // indicizza + raccogli check
    index_section_items(&mut index_map, "Bubbledesk", &sec.items, None, None);
    collect_check_items(&mut check_map, &sec.items);
    subs.push(build_submenu_from_section(app, "Bubbledesk", sec)?);
  }

  if let Some(sec) = cfg.file.as_ref() {
    index_section_items(&mut index_map, "File", &sec.items, None, None);
    collect_check_items(&mut check_map, &sec.items);
    subs.push(build_submenu_from_section(app, "File", sec)?);
  }
  if let Some(sec) = cfg.edit.as_ref() {
    index_section_items(&mut index_map, "Edit", &sec.items, None, None);
    collect_check_items(&mut check_map, &sec.items);
    subs.push(build_submenu_from_section(app, "Edit", sec)?);
  }
  if let Some(sec) = cfg.view.as_ref() {
    index_section_items(&mut index_map, "View", &sec.items, None, None);
    collect_check_items(&mut check_map, &sec.items);
    subs.push(build_submenu_from_section(app, "View", sec)?);
  }
  if let Some(sec) = cfg.window.as_ref() {
    index_section_items(&mut index_map, "Window", &sec.items, None, None);
    collect_check_items(&mut check_map, &sec.items);
    subs.push(build_submenu_from_section(app, "Window", sec)?);
  }

  let refs: Vec<&dyn IsMenuItem<R>> = subs.iter().map(|s| s as &dyn IsMenuItem<R>).collect();
  let root = Menu::with_items(app, &refs)?;
  app.set_menu(root)?;

  // 🧠 salva gli state
  app.manage(MenuIndex { by_id: index_map });
  app.manage(CheckState { map: Mutex::new(check_map) });

  attach_menu_events(app.handle().clone());
  Ok(())
}

// raccoglie tutti i check dell'albero e li mette in map (id -> checked)
fn collect_check_items(map: &mut HashMap<String, bool>, items: &[MenuItemUnion]) {
  for it in items {
    match it {
      MenuItemUnion::Custom(MenuConfigCustomItem { id, interaction, checked, .. }) => {
        if matches!(interaction, crate::helpers::menu_builder::MenuInteraction::Check) {
          map.insert(id.clone(), checked.unwrap_or(false));
        }
      }
      MenuItemUnion::Submenu(MenuConfigSubmenuItem { items, .. }) => {
        collect_check_items(map, items);
      }
      _ => {}
    }
  }
}

fn add_if_some<R: Runtime>(
  app: &App<R>,
  acc: &mut Vec<Submenu<R>>,
  title: &str,
  sec: Option<&MenuSectionConfig>,
) -> tauri::Result<()> {
  if let Some(section) = sec {
    acc.push(build_submenu_from_section(app, title, section)?);
  }
  Ok(())
}

fn is_current_platform_in(list: &[MenuPlatform]) -> bool {
  #[cfg(target_os = "macos")]   { list.contains(&MenuPlatform::Macos) }
  #[cfg(target_os = "windows")] { list.contains(&MenuPlatform::Windows) }
  #[cfg(target_os = "linux")]   { list.contains(&MenuPlatform::Linux) }
}

// -------- config parsing (risorse) --------

fn parse_menu_config<R: Runtime>(app: &App<R>) -> Option<MenuConfig> {
  #[cfg(debug_assertions)]
  {
    if let Ok(root) = app.path().resolve("", BaseDirectory::Resource) {
      eprintln!("[RES] root: {:?}", root);
      if let Ok(entries) = fs::read_dir(&root) {
        eprintln!("[RES] entries at root:");
        for e in entries.flatten() {
          eprintln!("  - {:?}", e.path());
        }
      }
    }
  }

  // Copre sia la dichiarazione glob ("resources/**") sia il singolo file
  let candidates = [
    "menu/menu.config.json",           // se bundle: "resources/**"
    "menu.config.json",                // se bundle: "resources/menu/menu.config.json"
    "resources/menu/menu.config.json", // fallback
    "resources/menu.config.json",
  ];

  for candidate in candidates {
    if let Ok(path) = app.path().resolve(candidate, BaseDirectory::Resource) {
      if let Ok(s) = fs::read_to_string(&path) {
        #[cfg(debug_assertions)]
        eprintln!("[RES] loaded: {:?}", path);
        if let Ok(cfg) = serde_json::from_str::<MenuConfig>(&s) {
          return Some(cfg);
        }
      }
    }
  }

  #[cfg(debug_assertions)]
  eprintln!("⚠️  No menu config found in resources (tried multiple candidates)");
  None
}

// indicizza tutti gli item (custom) con sezione (lowercase) e parent (label+id)
fn index_section_items(
  map: &mut HashMap<String, MenuMeta>,
  section_title: &str,
  items: &[MenuItemUnion],
  parent_label: Option<&str>,
  parent_id: Option<&str>,
) {
  let section_lc = section_title.to_ascii_lowercase();

  for it in items {
    match it {
      MenuItemUnion::Custom(MenuConfigCustomItem { id, .. }) => {
        map.insert(
          id.clone(),
          MenuMeta {
            section: section_lc.clone(),
            parent_label: parent_label.map(|s| s.to_string()),
            parent_id: parent_id.map(|s| s.to_string()),
          },
        );
      }
      MenuItemUnion::Submenu(MenuConfigSubmenuItem { id, label, items }) => {
        // ricorsione: passa label + id del submenu come parent
        index_section_items(map, section_title, items, Some(label.as_str()), Some(id.as_str()));
      }
      _ => {}
    }
  }
}

// -------- event bridge --------

#[derive(Serialize, Clone)]
struct MenuEventPayload<'a> {
  id: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  checked: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  section: Option<String>,        // sempre lowercase
  #[serde(skip_serializing_if = "Option::is_none")]
  parent_label: Option<String>,   // se annidato
  #[serde(skip_serializing_if = "Option::is_none")]
  parent_id: Option<String>,      // se annidato
}

fn attach_menu_events<R: Runtime>(app: AppHandle<R>) {
  app.on_menu_event(move |app, ev| {
    let id = ev.id().as_ref();

    // 🔘 deterministico: se è un item "check" noto nel nostro stato, toggle e usa quel valore
    let checked = if let Some(state) = app.try_state::<CheckState>() {
      let mut map = state.map.lock().unwrap();
      if let Some(val) = map.get_mut(id) {
        *val = !*val;                 // toggle
        let new_val = *val;
        // allinea l'UI (idempotente anche se l'OS ha già aggiornato)
        let _ = super::set_checked(&app, id, new_val);
        Some(new_val)
      } else {
        None
      }
    } else {
      None
    };

    // 🔎 lookup in indice (se presente: section + parent)
    let (section, parent_label, parent_id) = if let Some(idx) = app.try_state::<MenuIndex>() {
      if let Some(meta) = idx.by_id.get(id) {
        (
          Some(meta.section.clone()),
          meta.parent_label.clone(),
          meta.parent_id.clone(),
        )
      } else {
        (None, None, None)
      }
    } else {
      (None, None, None)
    };
    
    let _ = app.emit(
      "menu:event",
      MenuEventPayload { id, checked, section, parent_label, parent_id }
    );
  });
}

// -------- runtime mutators --------

pub fn set_enabled<R: Runtime>(app: &AppHandle<R>, id: &str, enabled: bool) -> tauri::Result<()> {
  if let Some(menu) = app.menu() {
    if let Some(kind) = menu.get(id) {
      match kind {
        MenuItemKind::MenuItem(mi) => mi.set_enabled(enabled)?,
        MenuItemKind::Check(mi)    => mi.set_enabled(enabled)?,
        MenuItemKind::Icon(mi)     => mi.set_enabled(enabled)?,
        _ => {}
      }
    }
  }
  Ok(())
}

pub fn set_checked<R: Runtime>(app: &AppHandle<R>, id: &str, checked: bool) -> tauri::Result<()> {
  if let Some(menu) = app.menu() {
    if let Some(MenuItemKind::Check(mi)) = menu.get(id) {
      mi.set_checked(checked)?;
    }
  }
  Ok(())
}

#[tauri::command]
pub fn bd_menu_set_enabled<R: Runtime>(app: AppHandle<R>, id: String, enabled: bool) -> Result<(), String> {
  set_enabled(&app, &id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_menu_set_checked<R: Runtime>(app: AppHandle<R>, id: String, checked: bool) -> Result<(), String> {
  set_checked(&app, &id, checked).map_err(|e| e.to_string())
}

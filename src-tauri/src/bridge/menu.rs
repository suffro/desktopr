use serde::Serialize;
use tauri::{
  App, AppHandle, Wry, Emitter, Manager,
  menu::{Menu, MenuItemKind, IsMenuItem, Submenu},
  path::BaseDirectory
};
use std::fs;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::helpers::menu_builder::*;
use crate::bridge::tray::init_tray_from_section;
use crate::helpers::states::*;


#[derive(Clone)]
pub struct MenuMeta {
  pub section: String,
  pub parent_label: Option<String>,      // label del submenu genitore (se presente)
  pub parent_id: Option<String>,         // id del submenu genitore (se presente)
}

pub struct MenuIndex {
  pub by_id: HashMap<String, MenuMeta>,
}

struct CheckState {
  map: Mutex<HashMap<String, bool>>,
}

// -------- init --------

pub fn init_menu(app: &App<Wry>) -> tauri::Result<()> {
  eprintln!("[MENU] >>> init_menu CALLED");
  let cfg = match parse_menu_config(app) {
    Some(c) => c,
    None => return Ok(()),
  };
  eprintln!("[MENU] enabled={} platforms={:?}", cfg.enabled, cfg.platforms);
  eprintln!("[MENU] has macos_root? {} | file? {} | edit? {} | view? {} | window? {} | tray? {}",
    cfg.macos_root.is_some(), cfg.file.is_some(), cfg.edit.is_some(), cfg.view.is_some(), cfg.window.is_some(), cfg.tray.is_some()
  );

  if !cfg.enabled || !is_current_platform_in(&cfg.platforms) {
    return Ok(());
  }

  // 🔎 indice id -> { section, parent }
  let mut index_map: HashMap<String, MenuMeta> = HashMap::new();
  // 🔘 stato check id -> bool
  let mut check_map: HashMap<String, bool> = HashMap::new();

  let mut subs: Vec<Submenu<Wry>> = Vec::new();

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
  if let Some(sec) = cfg.tray.as_ref() {
    // 1) costruisci i 3 item predefiniti (custom click) + separator
    let mut tray_items: Vec<MenuItemUnion> = vec![
      MenuItemUnion::Custom(MenuConfigCustomItem {
        id: "tray.show".to_string(),
        label: "Show".to_string(),
        enabled: true,
        interaction: MenuInteraction::Click,
        checked: None,
        accelerator: None,
      }),
      MenuItemUnion::Custom(MenuConfigCustomItem {
        id: "tray.hide".to_string(),
        label: "Hide".to_string(),
        enabled: true,
        interaction: MenuInteraction::Click,
        checked: None,
        accelerator: None,
      }),
      MenuItemUnion::Custom(MenuConfigCustomItem {
        id: "tray.close".to_string(),
        label: "Close".to_string(),
        enabled: true,
        interaction: MenuInteraction::Click,
        checked: None,
        accelerator: None,
      }),
      MenuItemUnion::Custom(MenuConfigCustomItem {
        id: "tray.quit".to_string(),
        label: "Quit".to_string(),
        enabled: true,
        interaction: MenuInteraction::Click,
        checked: None,
        accelerator: None,
      }),
      MenuItemUnion::Separator,
    ];

    // 2) aggiungi gli item dal JSON (in coda, così i 3 restano in cima)
    tray_items.extend(sec.items.clone());

    // 3) indicizza + raccogli check sull'intera lista (ora include i predefiniti)
    index_section_items(&mut index_map, "Tray", &tray_items, None, None);
    collect_check_items(&mut check_map, &tray_items);

    // 4) crea una sezione "completa" da passare al builder del tray
    //    (se il tuo enum MenuSection non ha Tray, usa `section: sec.section.clone()` e assicurati che MenuSection: Clone)
    let sec_augmented = MenuSectionConfig {
      section: MenuSection::Tray,
      items: tray_items,
    };

    // 5) delega al costruttore del tray
    init_tray_from_section(&app.handle(), &sec_augmented)?;
  }

  let refs: Vec<&dyn IsMenuItem<Wry>> = subs.iter().map(|s| s as &dyn IsMenuItem<Wry>).collect();
  let root = Menu::with_items(app, &refs)?;
  eprintln!("[MENU] submenus count = {}", subs.len());
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

fn add_if_some(
  app: &App<Wry>,
  acc: &mut Vec<Submenu<Wry>>,
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

fn parse_menu_config(app: &App<Wry>) -> Option<MenuConfig> {
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
        if let Ok(cfg) = serde_json::from_str::<MenuConfig>(&s) {
          #[cfg(debug_assertions)]
          eprintln!("[RES] loaded: {:?}", path);
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
  #[serde(skip_serializing_if = "Option::is_none")]
  last_focused_window_label: Option<String>,
}

fn attach_menu_events(app: AppHandle<Wry>) {
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
    let window_label = get_latest_window_label(app);
    let _ = app.emit(
      "menu:event",
      MenuEventPayload { id, checked, section, parent_label, parent_id, last_focused_window_label: Some(window_label) }
    );
  });
}

// -------- runtime mutators --------

pub fn set_enabled(app: &AppHandle<Wry>, id: &str, enabled: bool) -> tauri::Result<()> {
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

pub fn set_checked(app: &AppHandle<Wry>, id: &str, checked: bool) -> tauri::Result<()> {
  if let Some(menu) = app.menu() {
    if let Some(MenuItemKind::Check(mi)) = menu.get(id) {
      mi.set_checked(checked)?;
    }
  }
  Ok(())
}

#[tauri::command]
pub fn bd_menu_set_enabled(app: AppHandle<Wry>, id: String, enabled: bool) -> Result<(), String> {
  set_enabled(&app, &id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bd_menu_set_checked(app: AppHandle<Wry>, id: String, checked: bool) -> Result<(), String> {
  set_checked(&app, &id, checked).map_err(|e| e.to_string())
}

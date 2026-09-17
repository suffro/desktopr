use serde::Deserialize;

// ========= Tipi coerenti con i "CORRECT MENU CONFIGURATION TYPES" =========

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuConfig {
    pub enabled: bool,
    pub platforms: Vec<MenuPlatform>,
    // Chiavi opzionali: file/view/edit/window/macosRoot -> MenuSectionConfig
    #[serde(default)]
    pub file: Option<MenuSectionConfig>,
    #[serde(default)]
    pub view: Option<MenuSectionConfig>,
    #[serde(default)]
    pub edit: Option<MenuSectionConfig>,
    #[serde(default)]
    pub window: Option<MenuSectionConfig>,
    #[serde(default)]
    pub macos_root: Option<MenuSectionConfig>, // camelCase -> macosRoot
    #[serde(default)]
    pub tray: Option<MenuSectionConfig>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MenuPlatform {
    Macos,
    Windows,
    Linux,
}

// Valore della chiave: { section: "...", items: [...] }
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuSectionConfig {
    // Part of the JSON contract; the Rust side selects sections by key.
    #[allow(dead_code)]
    pub section: MenuSection,
    pub items: Vec<MenuItemUnion>,
}

// "file" | "view" | "edit" | "window" | "macosRoot"
#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MenuSection {
    File,
    View,
    Edit,
    Window,
    MacosRoot,
    Tray,
}

// Unione TAGGATA sul campo "type" degli item
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MenuItemUnion {
    Custom(MenuConfigCustomItem),
    Predefined(MenuConfigPredefinedItem),
    Submenu(MenuConfigSubmenuItem),
    Separator, // item con { "type": "separator" }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuConfigCustomItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub interaction: MenuInteraction,
    #[serde(default)]
    pub checked: Option<bool>,
    #[serde(default)]
    pub accelerator: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuConfigPredefinedItem {
    pub item: MenuPredefinedMenuItemSlug,
    #[serde(default)]
    pub custom_label: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MenuInteraction {
    Click,
    Check,
}

// NB: niente "bring_all_to_front" in Tauri v2
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MenuPredefinedMenuItemSlug {
    About,
    CloseWindow,
    Copy,
    Cut,
    Fullscreen,
    Hide,
    HideOthers,
    Maximize,
    Minimize,
    Paste,
    Quit,
    Redo,
    SelectAll,
    Separator,
    Services,
    ShowAll,
    Undo,
}

// Per i submenu ricorsivi: il tag "type":"submenu" è gestito dall’enum
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MenuConfigSubmenuItem {
    pub id: String,
    pub label: String,
    pub items: Vec<MenuItemUnion>,
}

---
title: Menu Module
description: Documentation for the Menu module, managing native menus and interactive menu items.
prev:
    text: Badge
    link: guide/bridge/api/badge
next:
    text: Events
    link: guide/bridge/api/events
---


# Menu Module

The Menu Interface module allows you to manage native menus, such as the menu bar or context menus, and their interactive menu items. This module provides methods to enable, disable, or check menu items dynamically.


## Methods

| Method                  | Description                                                                                 |
|-------------------------|---------------------------------------------------------------------------------------------|
| `init.fromConfig(config)` | Initiate a custom menu passing a `MenuConfig` object type. |
| `init.fromJsonFile(filePath)` | Initiate a custom menu passing a JSON `string` that confroms to the `MenuConfig` object type. |
| `setEnabled(id, enabled)` | Enables or disables the menu item identified by `id`. Pass `true` to enable, `false` to disable. |
| `setChecked(id, checked)` | Checks or unchecks the menu item identified by `id`. Pass `true` to check, `false` to uncheck. |


## Types

### `MenuConfig`
```ts
/** Full menu configuration (runtime-applied or build-time equivalent) */
{
  /** Master switch for the whole menu */
  enabled: boolean;

  /** Platforms where this menu should be active */
  platforms: MenuPlatform[]; // "macos" | "windows" | "linux"

  /** macOS application menu (only shown on macOS) */
  macosRoot?: MenuSectionConfig;

  /** Standard sections */
  file?: MenuSectionConfig;
  edit?: MenuSectionConfig;
  view?: MenuSectionConfig;
  window?: MenuSectionConfig;

  /** Tray section (interpreted by your tray builder) */
  tray?: MenuSectionConfig;
}
```

### `MenuSectionConfig`
```ts
/** A section corresponds to a top-level menu (e.g., File, Edit, View, etc.) */
{
  /** Section identifier (e.g., "file", "edit", "view", "window", "macosRoot", "tray") */
  section: MenuSectionId;
  /** Items inside this section */
  items: MenuItem[];
}
```

### `MenuItem`
One of: `MenuCustomItem`, `MenuPredefinedItem`, `MenuSeparatorItem`, `MenuSubmenuItem`

#### `MenuCustomItem`
```ts
/** Custom clickable or checkable item */
{
  /** Discriminator */
  type: "custom";

  /** Unique id you’ll receive on click/check events */
  id: string;

  /** Visible label */
  label: string;

  /** Enable/disable item (default: true) */
  enabled?: boolean;

  /** one among "click" or "check" (default: "click") */
  interaction?: MenuInteraction;

  /** Initial check state (only for interaction: "check") */
  checked?: boolean;

  /** Optional accelerator, e.g. "CmdOrCtrl+R" */
  accelerator?: string;
}
```

#### `MenuPredefinedItem`
```ts
/** Predefined native item provided by Tauri */
{
  type: "predefined";

  /** One of the known predefined items */
  item: PredefinedItem;

  /** Optional custom label to override default (where applicable) */
  customLabel?: string;
}
```

#### `MenuSubmenuItem`
```ts
/** Submenu (recursive) */
{
  type: "submenu";

  /** Optional id for the submenu */
  id?: string;

  /** Submenu label */
  label: string;

  /** Nested items (recursive) */
  items: MenuItem[];
}
```

#### `MenuSeparatorItem`
```ts
/** Simple separator line */
{
  type: "separator";
}
```

## Examples

```js
await Desktopr.menu.init.fromConfig({
  enabled: true,
  platforms: ["macos", "windows", "linux"],

  view: {
    section: "view",
    items: [
      {
        type: "custom",
        id: "view.reload",
        label: "Reload",
        enabled: true,
        interaction: "click",
        accelerator: "CmdOrCtrl+R"
      }
    ]
  },

  edit: {
    section: "edit",
    items: [/* ... */]
  },

  file: {
    section: "edit",
    items: [/* ... */]
  },

  window: {
    section: "edit",
    items: [/* ... */]
  },

  macosRoot: {
    section: "macosRoot",
    items: [/* ... */]
  },

  tray: {
    section: "tray",
    items: [/* ... */]
  }
});
```

```js
// Enable a menu item by id
await Desktopr.menu.setEnabled('item1_id', true);

// Check a menu item by id
await Desktopr.menu.setChecked('item2_id', true);
```

## Menu Events

Too see how to emit or intercept menu events check the [events module](events).
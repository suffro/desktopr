/** Supported desktop platforms for native menus. */
export type MenuPlatform = "macos" | "windows" | "linux";

/** Interaction model for custom menu items. */
export type MenuInteraction = "click" | "check";

/** Predefined native items supported by the Rust menu builder. */
export type PredefinedItem =
  | "about"
  | "close_window"
  | "copy"
  | "cut"
  | "fullscreen"
  | "hide"
  | "hide_others"
  | "maximize"
  | "minimize"
  | "paste"
  | "quit"
  | "redo"
  | "select_all"
  | "separator"
  | "services"
  | "show_all"
  | "undo";

/** Allowed top-level native menu sections. */
export type MenuSectionId =
  | "macosRoot"
  | "file"
  | "edit"
  | "view"
  | "window"
  | "tray";

export interface MenuCustomItem {
  type: "custom";
  id: string;
  label: string;
  enabled: boolean;
  interaction: MenuInteraction;
  checked?: boolean;
  accelerator?: string;
}

export interface MenuPredefinedItem {
  type: "predefined";
  item: PredefinedItem;
  customLabel?: string;
}

export interface MenuSeparatorItem {
  type: "separator";
}

export interface MenuSubmenuItem {
  type: "submenu";
  id: string;
  label: string;
  items: MenuItem[];
}

export type MenuItem =
  | MenuCustomItem
  | MenuPredefinedItem
  | MenuSeparatorItem
  | MenuSubmenuItem;

export interface MenuSectionConfig {
  section: MenuSectionId;
  items: MenuItem[];
}

export interface MenuConfig {
  enabled: boolean;
  platforms: MenuPlatform[];
  macosRoot?: MenuSectionConfig;
  file?: MenuSectionConfig;
  edit?: MenuSectionConfig;
  view?: MenuSectionConfig;
  window?: MenuSectionConfig;
  tray?: MenuSectionConfig;
}

const menuItemDefinitions = {
  MenuCustomItem: {
    additionalProperties: false,
    properties: {
      accelerator: { type: "string" },
      checked: { type: "boolean" },
      enabled: { type: "boolean" },
      id: { type: "string" },
      interaction: { enum: ["click", "check"], type: "string" },
      label: { type: "string" },
      type: { const: "custom", type: "string" },
    },
    required: ["type", "id", "label", "enabled", "interaction"],
    type: "object",
  },
  MenuPredefinedItem: {
    additionalProperties: false,
    properties: {
      customLabel: { type: "string" },
      item: { $ref: "#/definitions/PredefinedItem" },
      type: { const: "predefined", type: "string" },
    },
    required: ["type", "item"],
    type: "object",
  },
  MenuSeparatorItem: {
    additionalProperties: false,
    properties: { type: { const: "separator", type: "string" } },
    required: ["type"],
    type: "object",
  },
  MenuSubmenuItem: {
    additionalProperties: false,
    properties: {
      id: { type: "string" },
      items: {
        items: { $ref: "#/definitions/MenuItem" },
        type: "array",
      },
      label: { type: "string" },
      type: { const: "submenu", type: "string" },
    },
    required: ["type", "id", "label", "items"],
    type: "object",
  },
  PredefinedItem: {
    enum: [
      "about",
      "close_window",
      "copy",
      "cut",
      "fullscreen",
      "hide",
      "hide_others",
      "maximize",
      "minimize",
      "paste",
      "quit",
      "redo",
      "select_all",
      "separator",
      "services",
      "show_all",
      "undo",
    ],
    type: "string",
  },
} as const;

/** JSON Schema for the menu configuration accepted by the Rust runtime. */
export const DESKTOPR_MENU_CONFIG_JSON_SCHEMA = {
  $id: "urn:desktopr:schema:menu-config",
  $schema: "http://json-schema.org/draft-07/schema#",
  $ref: "#/definitions/MenuConfig",
  definitions: {
    ...menuItemDefinitions,
    MenuConfig: {
      additionalProperties: false,
      properties: {
        edit: { $ref: "#/definitions/MenuSectionConfig" },
        enabled: { type: "boolean" },
        file: { $ref: "#/definitions/MenuSectionConfig" },
        macosRoot: { $ref: "#/definitions/MenuSectionConfig" },
        platforms: {
          items: { enum: ["macos", "windows", "linux"], type: "string" },
          type: "array",
        },
        tray: { $ref: "#/definitions/MenuSectionConfig" },
        view: { $ref: "#/definitions/MenuSectionConfig" },
        window: { $ref: "#/definitions/MenuSectionConfig" },
      },
      required: ["enabled", "platforms"],
      type: "object",
    },
    MenuItem: {
      oneOf: [
        { $ref: "#/definitions/MenuCustomItem" },
        { $ref: "#/definitions/MenuPredefinedItem" },
        { $ref: "#/definitions/MenuSeparatorItem" },
        { $ref: "#/definitions/MenuSubmenuItem" },
      ],
    },
    MenuSectionConfig: {
      additionalProperties: false,
      properties: {
        items: {
          items: { $ref: "#/definitions/MenuItem" },
          type: "array",
        },
        section: {
          enum: ["file", "view", "edit", "window", "macosRoot", "tray"],
          type: "string",
        },
      },
      required: ["section", "items"],
      type: "object",
    },
  },
} as const;

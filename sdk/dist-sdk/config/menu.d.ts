/** Supported desktop platforms for native menus. */
export type MenuPlatform = "macos" | "windows" | "linux";
/** Interaction model for custom menu items. */
export type MenuInteraction = "click" | "check";
/** Predefined native items supported by the Rust menu builder. */
export type PredefinedItem = "about" | "close_window" | "copy" | "cut" | "fullscreen" | "hide" | "hide_others" | "maximize" | "minimize" | "paste" | "quit" | "redo" | "select_all" | "separator" | "services" | "show_all" | "undo";
/** Allowed top-level native menu sections. */
export type MenuSectionId = "macosRoot" | "file" | "edit" | "view" | "window" | "tray";
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
export type MenuItem = MenuCustomItem | MenuPredefinedItem | MenuSeparatorItem | MenuSubmenuItem;
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
/** JSON Schema for the menu configuration accepted by the Rust runtime. */
export declare const DESKTOPR_MENU_CONFIG_JSON_SCHEMA: {
    readonly $id: "urn:desktopr:schema:menu-config";
    readonly $schema: "http://json-schema.org/draft-07/schema#";
    readonly $ref: "#/definitions/MenuConfig";
    readonly definitions: {
        readonly MenuConfig: {
            readonly additionalProperties: false;
            readonly properties: {
                readonly edit: {
                    readonly $ref: "#/definitions/MenuSectionConfig";
                };
                readonly enabled: {
                    readonly type: "boolean";
                };
                readonly file: {
                    readonly $ref: "#/definitions/MenuSectionConfig";
                };
                readonly macosRoot: {
                    readonly $ref: "#/definitions/MenuSectionConfig";
                };
                readonly platforms: {
                    readonly items: {
                        readonly enum: readonly ["macos", "windows", "linux"];
                        readonly type: "string";
                    };
                    readonly type: "array";
                };
                readonly tray: {
                    readonly $ref: "#/definitions/MenuSectionConfig";
                };
                readonly view: {
                    readonly $ref: "#/definitions/MenuSectionConfig";
                };
                readonly window: {
                    readonly $ref: "#/definitions/MenuSectionConfig";
                };
            };
            readonly required: readonly ["enabled", "platforms"];
            readonly type: "object";
        };
        readonly MenuItem: {
            readonly oneOf: readonly [{
                readonly $ref: "#/definitions/MenuCustomItem";
            }, {
                readonly $ref: "#/definitions/MenuPredefinedItem";
            }, {
                readonly $ref: "#/definitions/MenuSeparatorItem";
            }, {
                readonly $ref: "#/definitions/MenuSubmenuItem";
            }];
        };
        readonly MenuSectionConfig: {
            readonly additionalProperties: false;
            readonly properties: {
                readonly items: {
                    readonly items: {
                        readonly $ref: "#/definitions/MenuItem";
                    };
                    readonly type: "array";
                };
                readonly section: {
                    readonly enum: readonly ["file", "view", "edit", "window", "macosRoot", "tray"];
                    readonly type: "string";
                };
            };
            readonly required: readonly ["section", "items"];
            readonly type: "object";
        };
        readonly MenuCustomItem: {
            readonly additionalProperties: false;
            readonly properties: {
                readonly accelerator: {
                    readonly type: "string";
                };
                readonly checked: {
                    readonly type: "boolean";
                };
                readonly enabled: {
                    readonly type: "boolean";
                };
                readonly id: {
                    readonly type: "string";
                };
                readonly interaction: {
                    readonly enum: readonly ["click", "check"];
                    readonly type: "string";
                };
                readonly label: {
                    readonly type: "string";
                };
                readonly type: {
                    readonly const: "custom";
                    readonly type: "string";
                };
            };
            readonly required: readonly ["type", "id", "label", "enabled", "interaction"];
            readonly type: "object";
        };
        readonly MenuPredefinedItem: {
            readonly additionalProperties: false;
            readonly properties: {
                readonly customLabel: {
                    readonly type: "string";
                };
                readonly item: {
                    readonly $ref: "#/definitions/PredefinedItem";
                };
                readonly type: {
                    readonly const: "predefined";
                    readonly type: "string";
                };
            };
            readonly required: readonly ["type", "item"];
            readonly type: "object";
        };
        readonly MenuSeparatorItem: {
            readonly additionalProperties: false;
            readonly properties: {
                readonly type: {
                    readonly const: "separator";
                    readonly type: "string";
                };
            };
            readonly required: readonly ["type"];
            readonly type: "object";
        };
        readonly MenuSubmenuItem: {
            readonly additionalProperties: false;
            readonly properties: {
                readonly id: {
                    readonly type: "string";
                };
                readonly items: {
                    readonly items: {
                        readonly $ref: "#/definitions/MenuItem";
                    };
                    readonly type: "array";
                };
                readonly label: {
                    readonly type: "string";
                };
                readonly type: {
                    readonly const: "submenu";
                    readonly type: "string";
                };
            };
            readonly required: readonly ["type", "id", "label", "items"];
            readonly type: "object";
        };
        readonly PredefinedItem: {
            readonly enum: readonly ["about", "close_window", "copy", "cut", "fullscreen", "hide", "hide_others", "maximize", "minimize", "paste", "quit", "redo", "select_all", "separator", "services", "show_all", "undo"];
            readonly type: "string";
        };
    };
};

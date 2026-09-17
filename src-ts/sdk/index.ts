// src-ts/sdk/index.ts

// Main SDK entry: typed proxy over window.Desktopr
export { Desktopr, isDesktoprAvailable } from "./_proxy";

// Re-export the API type so app devs can type their code against it if they want
export {
  type DesktoprAPI,
  type MenuConfig,
  type MenuCustomItem,
  type MenuInteraction,
  type MenuItem,
  type MenuPlatform,
  type MenuPredefinedItem,
  type MenuSectionConfig,
  type MenuSectionId,
  type MenuSeparatorItem,
  type MenuSubmenuItem,
  type PredefinedItem,
} from "../types";
export { DESKTOPR_MENU_CONFIG_JSON_SCHEMA } from "../config/menu";

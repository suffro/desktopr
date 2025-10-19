import { BubbledeskAPI, MenuConfig } from "@types"
import { cryptoTools, validate } from "suffro-lib";


export const applyMenuConfig = async (core: { invoke: BubbledeskAPI["invoke"] }, menuConfig: MenuConfig): Promise<void> => {
    validateMenuConfig(menuConfig);
    const jsonString: string = JSON.stringify(menuConfig);
    const jsonBase64: string = cryptoTools.base64.encode(jsonString?.trim())
    await core.invoke("bd_apply_menu_json", { json: jsonBase64, isBase64: true });
}


/**
 * Public entry point — validates a menu configuration object before sending it
 * to Rust. Throws an Error if invalid. Depth control is internal only.
 */
function validateMenuConfig(config: any): void {
  _validateMenuConfig(config, 0);
}

// ---- Internal recursive implementation ----

function _validateMenuConfig(config: any, depth: number): void {
  if (depth > 32) throw new Error("Menu config too deeply nested (possible recursion).");
  
  if (typeof config !== "object" || config === null)
    throw new Error("Menu config must be an object.");

  if(validate.emptyObject(config)) throw new Error(`Menu config is an empty object:\n\n${config}`);

  // --- Root fields ---
  if (typeof config.enabled !== "boolean")
    throw new Error("Missing or invalid 'enabled' (boolean required).");

  if (!Array.isArray(config.platforms))
    throw new Error("Missing or invalid 'platforms' (array required).");

  for (const p of config.platforms) {
    if (!["macos", "windows", "linux"].includes(p))
      throw new Error(`Invalid platform '${p}'.`);
  }

  // --- Sections ---
  const validSections = ["macosRoot", "file", "edit", "view", "window", "tray"];
  for (const key of validSections) {
    if (config[key]) {
      validateSection(config[key], key, depth + 1);
    }
  }
}

/** Validates a single menu section */
function validateSection(section: any, name: string, depth: number): void {
  if (typeof section !== "object" || section === null)
    throw new Error(`Section '${name}' must be an object.`);

  if (!Array.isArray(section.items))
    throw new Error(`Section '${name}' missing or invalid 'items' array.`);

  validateItems(section.items, `${name}.items`, depth + 1);
}

/** Recursively validates items and nested submenus */
function validateItems(items: any[], path: string, depth: number): void {
  if (!Array.isArray(items)) throw new Error(`${path} must be an array.`);
  if (depth > 32) throw new Error(`Exceeded maximum menu depth at ${path}`);

  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    const loc = `${path}[${i}]`;
    if (typeof item !== "object" || item === null)
      throw new Error(`${loc} must be an object.`);

    const type = item.type;
    if (typeof type !== "string")
      throw new Error(`${loc} missing 'type' field.`);

    switch (type) {
      case "custom": {
        if (typeof item.id !== "string") throw new Error(`${loc} missing 'id' (string).`);
        if (typeof item.label !== "string") throw new Error(`${loc} missing 'label' (string).`);
        if (item.enabled !== undefined && typeof item.enabled !== "boolean")
          throw new Error(`${loc} invalid 'enabled' type (boolean expected).`);

        if (
          item.interaction !== undefined &&
          !["click", "check"].includes(item.interaction)
        ) {
          throw new Error(`${loc} invalid 'interaction' value.`);
        }

        if (
          item.interaction === "check" &&
          item.checked !== undefined &&
          typeof item.checked !== "boolean"
        ) {
          throw new Error(`${loc} invalid 'checked' for checkable item (boolean expected).`);
        }

        if (
          item.accelerator !== undefined &&
          typeof item.accelerator !== "string"
        ) {
          throw new Error(`${loc} invalid 'accelerator' type (string expected).`);
        }
        break;
      }

      case "submenu": {
        if (typeof item.label !== "string")
          throw new Error(`${loc} submenu missing 'label' (string).`);
        if (!Array.isArray(item.items))
          throw new Error(`${loc} submenu missing 'items' array.`);
        validateItems(item.items, `${loc}.items`, depth + 1);
        break;
      }

      case "predefined": {
        if (typeof item.item !== "string")
          throw new Error(`${loc} predefined item missing 'item' field.`);
        if (
          item.customLabel !== undefined &&
          typeof item.customLabel !== "string"
        ) {
          throw new Error(`${loc} invalid 'customLabel' type (string expected).`);
        }
        break;
      }

      case "separator":
        break;

      default:
        throw new Error(`${loc} has unknown type '${type}'.`);
    }
  }
}
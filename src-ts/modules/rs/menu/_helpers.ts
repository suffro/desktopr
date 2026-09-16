import Ajv, { type ErrorObject } from "ajv";
import { DESKTOPR_MENU_CONFIG_JSON_SCHEMA } from "../../../config/menu";
import type { DesktoprAPI, MenuConfig } from "../../../types";

const ajv = new Ajv({ allErrors: true, strict: false });
const validateMenuConfigSchema = ajv.compile(
  DESKTOPR_MENU_CONFIG_JSON_SCHEMA,
);

function isMacOS(): boolean {
  return typeof navigator !== "undefined" && /Mac/i.test(navigator.platform);
}

function formatValidationError(error: ErrorObject): string {
  const path = error.instancePath || "(root)";
  return `${path} ${error.message ?? "is invalid"}`;
}

function validateMenuConfig(config: unknown): asserts config is MenuConfig {
  if (validateMenuConfigSchema(config)) {
    return;
  }

  const details = (validateMenuConfigSchema.errors ?? [])
    .map(formatValidationError)
    .join("; ");
  throw new Error(`Invalid Desktopr menu configuration: ${details}`);
}

export const initMenuConfig = async (
  core: { invoke: DesktoprAPI["invoke"] },
  menuConfig: MenuConfig,
  windowLabel?: string,
): Promise<void> => {
  validateMenuConfig(menuConfig);

  if (windowLabel) {
    if (isMacOS()) {
      console.warn(
        "[Desktopr] Native window-specific menus are not supported on macOS.",
      );
      return;
    }

    await core.invoke("dtr_init_menu_for_window_from_json", {
      windowLabel,
      cfgJson: menuConfig,
    });
    return;
  }

  await core.invoke("dtr_init_menu_from_json", {
    cfgJson: menuConfig,
  });
};

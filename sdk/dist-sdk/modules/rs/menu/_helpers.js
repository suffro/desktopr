"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.initMenuConfig = void 0;
const ajv_1 = __importDefault(require("ajv"));
const menu_1 = require("../../../config/menu");
const ajv = new ajv_1.default({ allErrors: true, strict: false });
const validateMenuConfigSchema = ajv.compile(menu_1.DESKTOPR_MENU_CONFIG_JSON_SCHEMA);
function isMacOS() {
    return typeof navigator !== "undefined" && /Mac/i.test(navigator.platform);
}
function formatValidationError(error) {
    const path = error.instancePath || "(root)";
    return `${path} ${error.message ?? "is invalid"}`;
}
function validateMenuConfig(config) {
    if (validateMenuConfigSchema(config)) {
        return;
    }
    const details = (validateMenuConfigSchema.errors ?? [])
        .map(formatValidationError)
        .join("; ");
    throw new Error(`Invalid Desktopr menu configuration: ${details}`);
}
const initMenuConfig = async (core, menuConfig, windowLabel) => {
    validateMenuConfig(menuConfig);
    if (windowLabel) {
        if (isMacOS()) {
            console.warn("[Desktopr] Native window-specific menus are not supported on macOS.");
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
exports.initMenuConfig = initMenuConfig;

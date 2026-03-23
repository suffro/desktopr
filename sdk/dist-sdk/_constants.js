"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.COMAPNION_WINDOW_LABEL_PREFIX = exports.APP_VERSION = exports.APP_URL = void 0;
const bridge_constants_json_1 = __importDefault(require("./bridge.constants.json"));
exports.APP_URL = bridge_constants_json_1.default.appUrl;
exports.APP_VERSION = bridge_constants_json_1.default.appVersion;
exports.COMAPNION_WINDOW_LABEL_PREFIX = bridge_constants_json_1.default.companionWindowLabelPrefix;

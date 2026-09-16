"use strict";
// src-ts/sdk/index.ts
Object.defineProperty(exports, "__esModule", { value: true });
exports.DESKTOPR_MENU_CONFIG_JSON_SCHEMA = exports.Desktopr = void 0;
// Main SDK entry: typed proxy over window.Desktopr
var _proxy_1 = require("./_proxy");
Object.defineProperty(exports, "Desktopr", { enumerable: true, get: function () { return _proxy_1.Desktopr; } });
var menu_1 = require("../config/menu");
Object.defineProperty(exports, "DESKTOPR_MENU_CONFIG_JSON_SCHEMA", { enumerable: true, get: function () { return menu_1.DESKTOPR_MENU_CONFIG_JSON_SCHEMA; } });

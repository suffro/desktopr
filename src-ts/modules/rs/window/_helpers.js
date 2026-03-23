"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.waitTauri = exports.tauriReadyCheck = void 0;
const suffro_lib_1 = require("suffro-lib");
const tauriReadyCheck = () => (typeof window !== "undefined" && (window.__TAURI__) && (window.Desktopr));
exports.tauriReadyCheck = tauriReadyCheck;
const waitTauri = async () => {
    const interval = 500;
    let counter = 0;
    while (counter <= 30000 && !(0, exports.tauriReadyCheck)()) {
        await (0, suffro_lib_1.wait)(interval);
        counter = counter + interval;
    }
};
exports.waitTauri = waitTauri;

"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.newWindow = exports.waitTauri = exports.tauriReadyCheck = void 0;
const _constants_1 = require("../../../_constants");
const suffro_lib_1 = require("suffro-lib");
const tauriReadyCheck = () => typeof window !== "undefined" &&
    window.__TAURI__ &&
    window.Bubbledesk;
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
const newWindow = async (core, options) => {
    if (options?.label &&
        options.label.trim().toLowerCase().startsWith((_constants_1.COMAPNION_WINDOW_LABEL_PREFIX).trim().toLowerCase()))
        throw new Error(`[Reserved window label] ${_constants_1.COMAPNION_WINDOW_LABEL_PREFIX}* is an app reserved label`);
    const randomWindowLabel = `w_${Math.random()
        .toString(36)
        .substring(2, 2 + 8)}`;
    core.invoke("bd_win_open", {
        label: options?.label ?? randomWindowLabel,
        fullscreen: options?.fullscreen || false,
        url: options?.url ?? "",
    });
};
exports.newWindow = newWindow;

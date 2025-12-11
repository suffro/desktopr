"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildWindow = buildWindow;
const _helpers_1 = require("../companion/_helpers");
function buildWindow(core) {
    const randomWindowLabel = `w_${Math.random().toString(36).substring(2, 2 + 8)}`;
    return {
        devTools: {
            toggle: (label) => core.invoke("bd_toggle_devtools", { label: label ?? "main" }),
            open: (label) => core.invoke("bd_open_devtools", { label: label ?? "main" }),
            close: (label) => core.invoke("bd_close_devtools", { label: label ?? "main" }),
        },
        minimize: (label) => core.invoke("bd_win_minimize", { label: label ?? "main" }),
        maximizeToggle: (label) => core.invoke("bd_win_maximize", { label: label ?? "main" }),
        fullscreen: (enable, label) => core.invoke("bd_win_fullscreen", { enable, label: label ?? "main" }),
        new: async (options) => {
            if (options?.cacheOnly)
                (0, _helpers_1.launchCompanion)(core, {
                    title: options?.label,
                    url: options?.url,
                    openFullscreen: options?.fullscreen
                });
            else
                core.invoke("bd_win_open", {
                    label: ((options?.label) ?? randomWindowLabel),
                    fullscreen: ((options?.fullscreen) || false),
                    url: ((options?.url) ?? "")
                });
        },
        close: (label) => core.invoke("bd_win_close", { label }),
        getInfo: (label) => core.invoke("bd_win_get_info", { label }),
        state: {}
    };
}

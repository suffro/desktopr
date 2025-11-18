"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildAutostart = buildAutostart;
function buildAutostart(core) {
    return {
        enable: () => core.invoke("bd_autostart_enable"),
        disable: () => core.invoke("bd_autostart_disable"),
        isEnabled: () => core.invoke("bd_autostart_status"),
        mode: {
            get: () => core.invoke("bd_get_autostart_mode"),
            set: (mode) => core.invoke("bd_set_autostart_mode", { mode }),
        }
    };
}

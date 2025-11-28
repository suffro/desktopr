"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Bubbledesk = void 0;
exports.isBubbledeskAvailable = isBubbledeskAvailable;
// src-ts/sdk/bubbledesk.ts
const _helpers_1 = require("../_helpers");
// Internal helper: get a safe window reference
function getWindow() {
    if (typeof window === "undefined") {
        // Avoid using the bridge in SSR or in non-browser environments
        throw new Error("[Bubbledesk] window is not defined. Are you running in SSR?");
    }
    return window;
}
// Internal helper: access the real global bridge
function getGlobalBridge() {
    const w = getWindow();
    const bridge = w.Bubbledesk;
    if (!bridge) {
        // Bubbledesk bridge is not yet injected by the wrapper
        console.error("[Bubbledesk] window.Bubbledesk is not available. Is the desktop wrapper loaded?");
        return;
    }
    return bridge;
}
// Public helper to check if the native Bubbledesk bridge is available.
// This must never throw, even in SSR or when running outside the wrapper.
function isBubbledeskAvailable() {
    try {
        if (typeof window === "undefined")
            return false;
        if (!(0, _helpers_1.isTauri)())
            return false;
        const w = window;
        return !!w.Bubbledesk;
    }
    catch (error) {
        console.error(error);
        return false;
    }
}
// Public SDK object.
// At runtime, this is just a Proxy that forwards everything to window.Bubbledesk,
// but from the developer point of view it is strongly typed as BubbledeskAPI.
exports.Bubbledesk = new Proxy({}, {
    get(_target, prop, _receiver) {
        const bridge = getGlobalBridge() ?? undefined;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const value = bridge[prop];
        // If the property is a function, bind it to the original object
        if (typeof value === "function") {
            return value.bind(bridge);
        }
        return value;
    },
    set(_target, prop, value) {
        const bridge = getGlobalBridge() ?? undefined;
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        bridge[prop] = value;
        return true;
    }
});

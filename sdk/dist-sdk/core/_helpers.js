"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.windowTauriProxy = exports.ensureCore = void 0;
exports.extractCore = extractCore;
function extractCore(source) {
    if (!source)
        return null;
    if (typeof source === "object" && "invoke" in source) {
        const core = source;
        if (typeof core.invoke === "function")
            return core;
    }
    if (typeof source === "object" && "core" in source) {
        const maybe = source.core;
        if (maybe && typeof maybe.invoke === "function")
            return maybe;
    }
    return null;
}
const ensureCore = () => new Promise((resolve, reject) => {
    const deadline = Date.now() + 10_000;
    (function tick() {
        const core = extractCore(window?.__TAURI__);
        if (core)
            return resolve(core);
        if (Date.now() > deadline)
            return reject(new Error("Tauri core.invoke not available"));
        requestAnimationFrame(tick);
    })();
});
exports.ensureCore = ensureCore;
// Universal Proxy for window.__TAURI__
exports.windowTauriProxy = new Proxy({}, {
    get(_target, prop) {
        const tauri = window.__TAURI__;
        if (!tauri) {
            console.warn("window.__TAURI__ is not available");
            return undefined;
        }
        const value = tauri[prop];
        // If it's a function, bind correct 'this'
        if (typeof value === "function") {
            return value.bind(tauri);
        }
        return value;
    }
});

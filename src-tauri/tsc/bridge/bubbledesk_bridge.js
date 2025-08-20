"use strict";
/* Bubbledesk bridge bootstrap (TypeScript, Tauri v2)
 * - Waits for __TAURI__.core.invoke (or __TAURI__.invoke)
 * - Exposes window.Bubbledesk with typed helpers
 * - Safe to import multiple times (idempotent)
 * NOTE: keep comments in English as per project convention
 */
/** Safely extract the Tauri core from window.__TAURI__ */
function extractCore(source) {
    if (!source)
        return null;
    // case 1: source is already TauriCore
    if (typeof source === "object" && "invoke" in source) {
        const core = source;
        if (typeof core.invoke === "function")
            return core;
    }
    // case 2: source is { core: TauriCore }
    if (typeof source === "object" && "core" in source) {
        const maybe = source.core;
        if (maybe && typeof maybe.invoke === "function") {
            return maybe;
        }
    }
    return null;
}
/** Promise that resolves when Tauri core is available (with a timeout) */
const ensureCore = () => new Promise((resolve, reject) => {
    const deadline = Date.now() + 10000;
    (function tick() {
        const core = extractCore(window?.__TAURI__);
        if (core)
            return resolve(core);
        if (Date.now() > deadline) {
            return reject(new Error("Tauri core.invoke not available"));
        }
        requestAnimationFrame(tick);
    })();
});
/** Idempotent bootstrap */
(() => {
    if (typeof window === "undefined" || window.Bubbledesk)
        return;
    const api = {
        get isAvailable() {
            return true;
        },
        version: "0.2.1",
        get ready() {
            return ensureCore().then(() => true);
        },
        async invoke(cmd, payload) {
            const core = await ensureCore();
            return core.invoke(cmd, payload);
        },
        notifications: {
            state: () => api.invoke("bd_notification_state"),
            request: () => api.invoke("bd_request_permission"),
            show: (title, body) => api.invoke("bd_notify", { title, body }),
        },
        clipboard: {
            readText: () => api.invoke("bd_clipboard_read"),
            writeText: (text) => api.invoke("bd_clipboard_write", { text }),
        },
        files: {
            open: (option) => api.invoke("bd_file_open", {
                multi: option?.multi ?? false,
            }),
            // NOTE: Rust currently returns String ("" on cancel). If you switch to Option<String>,
            // change this signature to Promise<string | null>.
            save: (default_name) => api.invoke("bd_file_save", {
                default_name: default_name ?? null,
            }),
        },
        app: {
            info: () => api.invoke("bd_app_info"),
        },
        window: {
            minimize: () => api.invoke("bd_win_minimize"),
            maximizeToggle: () => api.invoke("bd_win_maximize"),
            fullscreen: (enable) => api.invoke("bd_win_fullscreen", { enable }),
        },
        events: {
            emit: (event, payload) => api.invoke("bd_event_emit", { event, payload }),
            emitTo: (window_label, event, payload) => api.invoke("bd_event_emit_to", { window_label, event, payload }),
            listen: async (event, handler) => {
                await api.ready;
                const evt = window.__TAURI__?.event;
                if (!evt?.listen)
                    throw new Error("Tauri event API not available");
                const unlisten = await evt.listen(event, (e) => handler(e?.payload));
                return () => unlisten();
            },
        },
    };
    Object.defineProperty(window, "Bubbledesk", {
        value: api,
        enumerable: false,
        configurable: false,
        writable: false,
    });
})();

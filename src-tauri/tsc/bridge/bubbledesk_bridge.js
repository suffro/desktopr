"use strict";
/* Bubbledesk bridge bootstrap (TypeScript, Tauri v2)
 * - Waits for __TAURI__.core.invoke (or __TAURI__.invoke)
 * - Exposes window.Bubbledesk with typed helpers
 * - Safe to import multiple times (idempotent)
 * NOTE: keep comments in English as per project convention
 */
const _gs = () => {
    const g = window.__TAURI__?.globalShortcut;
    if (!g)
        throw new Error("Global Shortcut plugin not available");
    return g;
};
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
const listenForEvent = async (event, handler) => {
    const tauri = window.__TAURI__;
    const eventApi = tauri?.event;
    if (!eventApi?.listen)
        throw new Error("Tauri event API not available");
    const unlisten = await eventApi.listen(event, (e) => handler(e?.payload));
    return () => unlisten();
};
const getCurrentWebviewWindow = () => {
    const tauri = window.__TAURI__;
    return tauri?.window?.getCurrent?.();
};
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
            on: async (event, handler) => await listenForEvent(event, handler),
            once: (event) => new Promise(async (resolve) => {
                const off = await listenForEvent(event, (p) => {
                    off();
                    resolve(p);
                });
            }),
            onMany: async (events, handler) => {
                const offs = await Promise.all(events.map((name) => listenForEvent(name, (p) => handler(name, p))));
                return () => offs.forEach((off) => off());
            },
            onShortcut: async (handler) => await listenForEvent("shortcut:event", handler),
            onDragDrop: async (handler, options) => {
                const evs = ["dragdrop:enter", "dragdrop:drop", "dragdrop:cancel"];
                if (options?.includeHover)
                    evs.push("dragdrop:hover");
                const offs = await Promise.all(evs.map((name) => listenForEvent(name, (p) => handler(name, p))));
                return () => offs.forEach((off) => off());
            },
        },
        globalShortcut: {
            /** eg: "CommandOrControl+Alt+T" */
            register: async (accelerator, cb, options) => {
                const gs = _gs();
                await gs.register(accelerator, async (e) => {
                    const _playload = { accelerator, ...e };
                    if (options?.emitEvent)
                        await api.events.emit("shortcut:event", _playload);
                    cb(_playload);
                });
            },
            unregister: async (accelerator) => {
                const gs = _gs();
                const reg = await gs.isRegistered(accelerator);
                if (reg)
                    await gs.unregister(accelerator);
            },
            unregisterAll: async () => {
                const gs = _gs();
                await gs.unregisterAll();
            },
            isRegistered: async (accelerator) => {
                const gs = _gs();
                return gs.isRegistered(accelerator);
            },
        },
        fs: {
            // ----------- CACHED DATA -----------
            cache: {
                listDir: (rel = "") => api.invoke("fs_list_dir", { rel, permanent: false }),
                mkdir: (rel) => api.invoke("fs_mkdir", { rel, permanent: false }),
                rm: (rel, recursive = false) => api.invoke("fs_rm", { rel, recursive, permanent: false }),
                stat: (rel = "") => api.invoke("fs_stat", { rel, permanent: false }),
                // NEW:
                writeText: (rel, contents, opts) => api.invoke("fs_write_text", {
                    rel,
                    permanent: false,
                    contents,
                    createDirs: opts?.createDirs,
                    append: opts?.append,
                }),
                readText: (rel) => api.invoke("fs_read_text", { rel, permanent: false }),
                writeBytes: (rel, base64, opts) => api.invoke("fs_write_bytes", {
                    rel,
                    permanent: false,
                    dataBase64: base64,
                    createDirs: opts?.createDirs,
                }),
                readBytes: (rel) => api.invoke("fs_read_bytes", { rel, permanent: false }),
                exists: (rel) => api.invoke("fs_exists", { rel, permanent: false }),
                move: (src, dest, opts) => api.invoke("fs_move", {
                    src,
                    dest,
                    permanent: false,
                    createDirs: opts?.createDirs,
                    overwrite: opts?.overwrite,
                }),
                copy: (src, dest, opts) => api.invoke("fs_copy", {
                    src,
                    dest,
                    permanent: false,
                    recursive: opts?.recursive,
                    createDirs: opts?.createDirs,
                    overwrite: opts?.overwrite,
                }),
                clear: () => api.invoke("fs_clear_cache", {}),
                path: async () => (await api.invoke("fs_paths"))?.cache,
                base: ".cache",
            },
            // ----------- PERMANENT DATA -----------
            data: {
                listDir: (rel = "") => api.invoke("fs_list_dir", { rel, permanent: true }),
                mkdir: (rel) => api.invoke("fs_mkdir", { rel, permanent: true }),
                rm: (rel, recursive = false) => api.invoke("fs_rm", { rel, recursive, permanent: true }),
                stat: (rel = "") => api.invoke("fs_stat", { rel, permanent: true }),
                // NEW:
                writeText: (rel, contents, opts) => api.invoke("fs_write_text", {
                    rel,
                    permanent: true,
                    contents,
                    createDirs: opts?.createDirs,
                    append: opts?.append,
                }),
                readText: (rel) => api.invoke("fs_read_text", { rel, permanent: true }),
                writeBytes: (rel, base64, opts) => api.invoke("fs_write_bytes", {
                    rel,
                    permanent: true,
                    dataBase64: base64,
                    createDirs: opts?.createDirs,
                }),
                readBytes: (rel) => api.invoke("fs_read_bytes", { rel, permanent: true }),
                exists: (rel) => api.invoke("fs_exists", { rel, permanent: true }),
                move: (src, dest, opts) => api.invoke("fs_move", {
                    src,
                    dest,
                    permanent: true,
                    createDirs: opts?.createDirs,
                    overwrite: opts?.overwrite,
                }),
                copy: (src, dest, opts) => api.invoke("fs_copy", {
                    src,
                    dest,
                    permanent: true,
                    recursive: opts?.recursive,
                    createDirs: opts?.createDirs,
                    overwrite: opts?.overwrite,
                }),
                path: async () => (await api.invoke("fs_paths"))?.data,
                base: ".data",
            },
            paths: () => api.invoke("fs_paths"),
            base: { cache: ".cache", data: ".data" },
        },
    };
    Object.defineProperty(window, "Bubbledesk", {
        value: api,
        enumerable: false,
        configurable: false,
        writable: false,
    });
})();

"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const _main_1 = require("./_main");
const _constants_1 = require("./_constants");
const _helpers_1 = require("./_helpers");
const _main_2 = require("./modules/rs/worker/_main");
const _companion_context_1 = require("./_companion_context");
(() => {
    if (typeof window === "undefined")
        return;
    if (window.Desktopr)
        return;
    console.log("[Desktopr bridge] init script evaluated");
    const core = (0, _main_1.buildCore)();
    const api = {
        get isAvailable() { return true; },
        version: _constants_1.APP_VERSION,
        get ready() {
            return core.ready;
        },
        invoke: core.invoke,
        isDesktop: true,
        notifications: (0, _main_1.buildNotifications)(core),
        clipboard: (0, _main_1.buildClipboard)(core),
        files: (0, _main_1.buildFiles)(core),
        app: (0, _main_1.buildAppInfo)(core),
        window: (0, _main_1.buildWindow)(core),
        events: (0, _main_1.buildEvents)(core),
        globalShortcut: (0, _main_1.buildShortcuts)(core),
        fs: (0, _main_1.buildFs)(core),
        menu: (0, _main_1.buildMenu)(core),
        diagnostics: (0, _main_1.buildDiagnostics)(core),
        network: (0, _main_1.buildNetwork)(core),
        autostart: (0, _main_1.buildAutostart)(core),
        badge: (0, _main_1.buildBadge)(core),
        worker: (0, _main_2.buildWorker)(core),
        contextMenu: (0, _main_1.buildContextMenu)(core),
        tauri: _helpers_1.windowTauriProxy,
        onReady: _main_1.dtrReadyEventListener,
        // companion:        buildCompanion(core),
        globalVariables: (0, _main_1.buildGlobVar)(core),
        openBrowser: (url) => window.__TAURI__?.shell?.open(url)
    };
    Object.defineProperty(window, "Desktopr", {
        value: api,
        enumerable: false,
        configurable: false,
        writable: false,
    });
    console.log("[Desktopr bridge] window.Desktopr assigned", window.Desktopr);
    (0, _companion_context_1.getCacheOnlyWindowContext)();
})();
(async () => {
    await (0, _helpers_1.waitTauri)();
    if ((0, _helpers_1.tauriReadyCheck)()) {
        (0, _main_1.dtrInitiators)();
    }
    else {
        console.error("[Desktopr bridge] Tauri did not become ready in time");
    }
})();

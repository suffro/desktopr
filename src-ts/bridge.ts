import type { DesktoprAPI, WindowTauri } from "./_types";
import {
    buildCore,
    buildFs,
    buildNotifications,
    buildClipboard,
    buildFiles,
    buildWindow,
    buildEvents,
    buildShortcuts,
    buildAppInfo,
    buildMenu,
    dtrInitiators,
    buildDiagnostics,
    buildNetwork,
    buildAutostart,
    buildBadge,
    buildContextMenu,
    // buildCompanion,
    buildGlobVar,
    dtrReadyEventListener,
} from "./_main";
import { APP_VERSION } from "./_constants";
import { windowTauriProxy, tauriReadyCheck, waitTauri } from "./_helpers";
import { getCacheOnlyWindowContext } from "./_companion_context";
import { buildPlugins } from "./modules/rs/plugins/_main";

(() => {
  if (typeof window === "undefined") return;
  if ((window as any).Desktopr) return;

  console.log("[Desktopr bridge] init script evaluated");

  const core = buildCore();

  const api: DesktoprAPI = {
    get isAvailable() { return true; },
    version: APP_VERSION,
    get ready() {
      return core.ready;
    },
    invoke: core.invoke,
    isDesktop: true,
    notifications:    buildNotifications(core),
    clipboard:        buildClipboard(core),
    files:            buildFiles(core),
    app:              buildAppInfo(core),
    window:           buildWindow(core),
    events:           buildEvents(core),
    globalShortcut:   buildShortcuts(core),
    fs:               buildFs(core),
    menu:             buildMenu(core),
    diagnostics:      buildDiagnostics(core),
    network:          buildNetwork(core),
    autostart:        buildAutostart(core),
    badge:            buildBadge(core),
    plugins:          buildPlugins(core),
    contextMenu:      buildContextMenu(core),
    tauri:            windowTauriProxy as WindowTauri,
    onReady:          dtrReadyEventListener,
    // companion:        buildCompanion(core),
    globalVariables:  buildGlobVar(core),
    openBrowser: (url: string): Promise<void> => window.__TAURI__?.shell?.open(url)
  };

  Object.defineProperty(window, "Desktopr", {
    value: api,
    enumerable: false,
    configurable: false,
    writable: false,
  });

  console.log("[Desktopr bridge] window.Desktopr assigned", (window as any).Desktopr);

  getCacheOnlyWindowContext();
})();

(async () => {
  await waitTauri();

  if (tauriReadyCheck()) {
    dtrInitiators();
  } else {
    console.error("[Desktopr bridge] Tauri did not become ready in time");
  }
})();
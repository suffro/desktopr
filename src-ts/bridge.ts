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
} from "./_main";
import { APP_VERSION } from "./_constants";
import { windowTauriProxy, isTauri, tauriReadyCheck, waitTauri } from "./_helpers";
import { buildWorker } from "modules/rs/worker/_main";
import { getCacheOnlyWindowContext } from "_companion_context";

(() => {
  if (!isTauri() || (window as any).Desktopr) return;

  const core = buildCore();

  const api: DesktoprAPI = {
    get isAvailable() { return true; },
    version: APP_VERSION,
    get ready() {
      return core.ready; 
    },
    invoke: core.invoke,
    isDesktop:        isTauri(),
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
    worker:           buildWorker(core),
    contextMenu:      buildContextMenu(core),
    tauri:            windowTauriProxy as WindowTauri,
    // companion:        buildCompanion(core),
    globalVariables:  buildGlobVar(core),
    openBrowser: (url: string): Promise<void> => window.__TAURI__?.shell.open(url)
  };

  Object.defineProperty(window, "Desktopr", {
    value: api, enumerable: false, configurable: false, writable: false,
  });

  getCacheOnlyWindowContext();

})();


(async () => {
  await waitTauri();
  if(tauriReadyCheck()) dtrInitiators();
})();

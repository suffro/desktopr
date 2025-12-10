import type { BubbledeskAPI } from "./_types";
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
    bdInitiators,
    buildDiagnostics,
    buildNetwork,
    buildAutostart,
    buildBadge,
    buildContextMenu,
    // buildCompanion,
    buildGlobVar,
} from "./_main";
import { APP_VERSION } from "./_constants";
import { isTauri, tauriReadyCheck, waitTauri } from "./_helpers";
import { buildWorker } from "modules/rs/worker/_main";
import { getCompanionContext } from "_companion_context";

(() => {
  if (!isTauri() || (window as any).Bubbledesk) return;

  const core = buildCore();

  const api: BubbledeskAPI = {
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
    // companion:        buildCompanion(core),
    globalVariables:  buildGlobVar(core)
  };

  Object.defineProperty(window, "Bubbledesk", {
    value: api, enumerable: false, configurable: false, writable: false,
  });

  getCompanionContext();

})();


(async () => {
  await waitTauri();
  if(tauriReadyCheck()) bdInitiators();
})();

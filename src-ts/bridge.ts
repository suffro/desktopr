import type { BubbledeskAPI } from "@types";
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
} from "@main";
import { APP_VERSION } from "@constants";
import { tauriReadyCheck, waitTauri } from "@helpers";
import { buildWorker } from "modules/rs/worker/_main";

(() => {
  if (typeof window === "undefined" || (window as any).Bubbledesk) return;

  const core = buildCore();

  const api: BubbledeskAPI = {
    get isAvailable() { return true; },
    version: APP_VERSION,
    get ready() {
      return core.ready; 
    },
    invoke: core.invoke,

    notifications: buildNotifications(core),
    clipboard:     buildClipboard(core),
    files:         buildFiles(core),
    app:           buildAppInfo(core),
    window:        buildWindow(core),
    events:        buildEvents(core),
    globalShortcut:buildShortcuts(core),
    fs:            buildFs(core),
    menu:          buildMenu(core),
    diagnostics:   buildDiagnostics(core),
    network:       buildNetwork(core),
    autostart:     buildAutostart(core),
    badge:         buildBadge(core),
    worker:        buildWorker(core),
    contextMenu:   buildContextMenu(core),
  };

  Object.defineProperty(window, "Bubbledesk", {
    value: api, enumerable: false, configurable: false, writable: false,
  });

})();


(async () => {
  await waitTauri();
  if(tauriReadyCheck()) bdInitiators();
})();

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
    buildAppInfo
} from "@main";
import { VERSION } from "@constants";

(() => {
  // importantissimo: niente export in questo file
  if (typeof window === "undefined" || (window as any).Bubbledesk) return;

  const core = buildCore();

  const api: BubbledeskAPI = {
    get isAvailable() { return true; },
    version: VERSION,
    get ready() { return core.ready; },
    invoke: core.invoke,

    notifications: buildNotifications(core),
    clipboard:     buildClipboard(core),
    files:         buildFiles(core),
    app:           buildAppInfo(core),
    window:        buildWindow(core),
    events:        buildEvents(core),
    globalShortcut:buildShortcuts(core),
    fs:            buildFs(core),
  };

  Object.defineProperty(window, "Bubbledesk", {
    value: api, enumerable: false, configurable: false, writable: false,
  });
})();

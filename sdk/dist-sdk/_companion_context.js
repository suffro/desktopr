"use strict";
// Comments are in English
Object.defineProperty(exports, "__esModule", { value: true });
exports.getCacheOnlyWindowContext = getCacheOnlyWindowContext;
exports.isCacheOnlyWindow = isCacheOnlyWindow;
exports.getCacheOnlyWindowLabel = getCacheOnlyWindowLabel;
exports.getCacheOnlySessionId = getCacheOnlySessionId;
const _constants_1 = require("./_constants");
let state = {
    isCacheOnly: false,
};
function parseCompanionWindowLabel(label) {
    const prefix = _constants_1.COMAPNION_WINDOW_LABEL_PREFIX.trim();
    if (!label.startsWith(prefix)) {
        return { isCacheOnly: false };
    }
    const sessionId = label.slice(prefix.length);
    if (!sessionId) {
        return { isCacheOnly: false, windowLabel: label };
    }
    return { isCacheOnly: true, sessionId, windowLabel: label };
}
let initialized = false;
async function getCacheOnlyWindowContext() {
    if (initialized)
        return;
    initialized = true;
    if (typeof window === 'undefined' || !window?.Desktopr) {
        console.error("Error executing setupCompanionContext():\nwindow.Desktopr undefined or not yet initialized.");
        return;
    }
    const windowInfo = await window.Desktopr.window.getInfo();
    const compState = parseCompanionWindowLabel(windowInfo.label);
    if (window?.Desktopr?.window?.state)
        window.Desktopr.window.state = compState;
    // if(window?.Desktopr?.companion?.state) window.Desktopr.companion.state = compState;
}
// Simple getters
function isCacheOnlyWindow() {
    return (state?.isCacheOnly) ?? false;
}
function getCacheOnlyWindowLabel() {
    return state?.windowLabel;
}
function getCacheOnlySessionId() {
    return state?.sessionId;
}

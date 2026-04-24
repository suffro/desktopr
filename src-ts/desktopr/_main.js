"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.dtrReadyEventListener = exports.dtrInitiators = exports.DesktoprInstance = void 0;
// import { listenForEvent } from "../_helpers";
const _constants_1 = require("@constants");
exports.DesktoprInstance = {
    ready: () => {
        if (!window?.Desktopr)
            return false;
        return true;
    },
    get: () => {
        if (!exports.DesktoprInstance.ready())
            throw ("'window.Desktopr' not found");
        return window?.Desktopr;
    }
};
const dtrInitiators = async () => {
    try {
        if (!exports.DesktoprInstance.ready())
            throw ("'window.Desktopr' not found");
        const Desktopr = exports.DesktoprInstance.get();
        Desktopr.events.onMenuEvent((id) => {
            if (id === "view.devtools") {
                console.log("DevTools toggled");
                Desktopr.window.devTools.toggle("main");
            }
        });
        console.log("## READY ##");
        const eventReady = new CustomEvent(_constants_1.READY_EVENT_NAME);
        window?.dispatchEvent(eventReady);
    }
    catch (error) {
        console.error(error);
    }
};
exports.dtrInitiators = dtrInitiators;
const dtrReadyEventListener = (callback) => window.addEventListener(_constants_1.READY_EVENT_NAME, () => callback());
exports.dtrReadyEventListener = dtrReadyEventListener;

"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.removeContextMenuListener = exports.initContextMenuListener = exports.CM_TYPES = void 0;
exports.isCmType = isCmType;
exports.parseCmType = parseCmType;
exports.normalizeEntries = normalizeEntries;
const _helpers_1 = require("@helpers");
const suffro_lib_1 = require("suffro-lib");
// Keep CM_TYPES as a readonly tuple and assert it matches CmType[]
exports.CM_TYPES = [
    "item",
    "check",
    "separator",
    "submenu",
    "predefined",
];
// Narrow to CmType by inclusion
function isCmType(value) {
    return (typeof value === "string" && exports.CM_TYPES.includes(value));
}
/**
 * Normalize any unknown value into a CmType if possible:
 * - must be a non-empty string
 * - lowercased
 * - whitespace removed
 * - must be one of CM_TYPES
 */
function parseCmType(value) {
    if (!suffro_lib_1.validate.nonEmptyString(value))
        return null;
    const normalized = (0, _helpers_1.normalizeString)(value);
    return isCmType(normalized) ? normalized : null;
}
/**
 * Return only entries with a valid, normalized type.
 * Does not mutate input entries.
 */
function normalizeEntries(entries) {
    const out = [];
    for (const entry of entries) {
        const t = parseCmType(entry?.type);
        if (!t)
            continue;
        out.push({ ...entry, type: t });
    }
    return out;
}
const onCmClick = (ev, callback, preventDefault = true) => {
    try {
        // impedire il menu contestuale nativo se vuoi mostrare il tuo
        if (preventDefault)
            ev.preventDefault();
        // elemento effettivo bersaglio (può essere text node -> cast a HTMLElement)
        const target = ev.target;
        // se vuoi risalire fino a un elemento significativo (con attributo data-bd-context)
        const ancestorActionable = target?.closest("[data-bd-contextmenu]") ?? null;
        const descendantActionable = target?.querySelector("[data-bd-contextmenu]") ?? null;
        const info = {
            targetTag: target?.tagName ?? null,
            targetId: target?.id ?? null,
            targetClasses: target?.className ?? null,
            ancestorActionable,
            descendantActionable,
            // coordinate
            pageX: ev.pageX, // rispetto al documento (scorrimento incluso)
            pageY: ev.pageY,
            clientX: ev.clientX, // rispetto alla viewport
            clientY: ev.clientY,
            screenX: ev.screenX, // coordinate dello schermo
            screenY: ev.screenY,
            altKey: ev.altKey,
            ctrlKey: ev.ctrlKey,
            shiftKey: ev.shiftKey,
            metaKey: ev.metaKey,
        };
        if (!window?.Bubbledesk)
            throw new Error("[contextmenu listener] Bubbledesk not found");
        window.Bubbledesk.events.emit("cm:click", info);
        const callbackPlayload = {
            event: ev,
            ...info
        };
        if (callback)
            callback(callbackPlayload);
    }
    catch (error) {
        console.error(error);
    }
};
const initContextMenuListener = (callback, preventDefault = true) => {
    if (!window?.Bubbledesk)
        throw new Error("[contextmenu listener] Bubbledesk not found");
    const listening = window.Bubbledesk.contextMenu.listening;
    if (listening)
        return console.warn("[contextmenu listener] already initialized");
    // listener globale: intercetta i click col destro su qualunque elemento della pagina
    const listener = (ev) => onCmClick(ev, callback, preventDefault);
    window.Bubbledesk.contextMenu.listener = listener;
    document.addEventListener("contextmenu", listener);
    window.Bubbledesk.contextMenu.listening = true;
};
exports.initContextMenuListener = initContextMenuListener;
const removeContextMenuListener = () => {
    if (!window?.Bubbledesk)
        throw new Error("[contextmenu listener] Bubbledesk not found");
    const listening = window.Bubbledesk.contextMenu.listening;
    if (!listening)
        return;
    const listener = window.Bubbledesk.contextMenu.listener ?? (() => { });
    document.removeEventListener("contextmenu", listener);
    window.Bubbledesk.contextMenu.listening = false;
    window.Bubbledesk.contextMenu.listener = undefined;
};
exports.removeContextMenuListener = removeContextMenuListener;

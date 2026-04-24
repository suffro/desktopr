"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildContextMenu = buildContextMenu;
const _helpers_1 = require("../../../_helpers");
function buildContextMenu(core) {
    return {
        show: (entries, options) => core.invoke("dtr_context_menu_popup", { items: (0, _helpers_1.normalizeEntries)(entries), options }),
        handler: {
            init: _helpers_1.initContextMenuListener,
            remove: _helpers_1.removeContextMenuListener
        }
    };
}

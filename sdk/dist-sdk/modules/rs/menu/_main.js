"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildMenu = buildMenu;
const _helpers_1 = require("../../../_helpers");
function buildMenu(core) {
    return {
        setEnabled: (id, enabled) => core.invoke("bd_menu_set_enabled", { id, enabled }),
        setChecked: (id, checked) => core.invoke("bd_menu_set_checked", { id, checked }),
        init: {
            fromConfig: (config, windowLabel) => (0, _helpers_1.initMenuConfig)(core, config, windowLabel),
            fromJsonFile: (filePath) => core.invoke("bd_init_menu_from_file", { filePath })
        }
    };
}

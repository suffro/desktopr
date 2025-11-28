"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildBadge = buildBadge;
const _helpers_1 = require("../../../_helpers");
function buildBadge(core) {
    return {
        set: async (count) => {
            await (0, _helpers_1.platformSpecifcFilter)(["macos"]);
            core.invoke("bd_badge_set", { count });
        },
        clear: async () => {
            await (0, _helpers_1.platformSpecifcFilter)(["macos"]);
            core.invoke("bd_badge_clear");
        },
    };
}

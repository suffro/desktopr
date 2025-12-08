"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildCompanion = buildCompanion;
const _helpers_1 = require("../../../_helpers");
function buildCompanion(core) {
    return {
        launch: (appConfig) => (0, _helpers_1.launchCompanion)(core, appConfig),
    };
}

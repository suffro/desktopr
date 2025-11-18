"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildCompanion = buildCompanion;
function buildCompanion(core) {
    return {
        launch: (appConfig) => core.invoke("bd_launch_companion", { appConfig }),
    };
}

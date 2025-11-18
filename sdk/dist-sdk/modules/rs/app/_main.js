"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildAppInfo = buildAppInfo;
function buildAppInfo(core) {
    return {
        info: () => core.invoke("bd_app_info"),
        exit: (code) => core.invoke("bd_app_exit", { code: code ?? 0 })
    };
}

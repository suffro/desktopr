"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildGlobVar = buildGlobVar;
function buildGlobVar(core) {
    return {
        get: async (key) => core.invoke("bd_global_vars_get", { key }),
        set: async (key, value) => core.invoke("bd_global_vars_set", { key, value }),
        remove: async (key) => core.invoke("bd_global_vars_remove", { key }),
        list: async () => core.invoke("bd_global_vars_list")
    };
}

"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildWorker = buildWorker;
const _helpers_1 = require("../../../_helpers");
function buildWorker(core) {
    return {
        call: (method, payload, timeoutMs) => core.invoke("bd_worker_call", { modulePath: (0, _helpers_1.normalizeModuleName)(method), payload, timeoutMs }),
        status: () => core.invoke("bd_worker_status"),
        restart: () => core.invoke("bd_worker_restart"),
        // delivery: (id: string, result: string): Promise<boolean> => core.invoke("bd_worker_delivery", {id, result}),
        modules: {
            list: () => core.invoke("bd_worker_list_modules"),
            remove: (name) => core.invoke("bd_worker_remove_module", { name: (0, _helpers_1.normalizeModuleName)(name) }),
            addFromBytes: (name, contents) => core.invoke("bd_worker_add_module", { name: (0, _helpers_1.normalizeModuleName)(name), contents }),
            add: (name, maxBytes) => core.invoke("bd_worker_pick_and_add_module", { maxBytes, defaultName: (0, _helpers_1.normalizeModuleName)(name) }),
        },
        clearSandbox: () => core.invoke("bd_worker_clear_all"),
        paths: () => core.invoke("bd_worker_paths"),
    };
}

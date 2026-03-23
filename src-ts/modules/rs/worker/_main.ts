import { BubbledeskAPI } from "../../../_types";
import { U64, U8 } from "suffro-lib/utils";
import { WorkerCallPayload, WorkerInterface } from "./_types";
import { normalizeModuleName } from "../../../_helpers";

export function buildWorker(core: { invoke: BubbledeskAPI["invoke"] }) {
    return {
      call: (method: string, payload: WorkerCallPayload, timeoutMs?: U64): Promise<string> => core.invoke("bd_worker_call", {modulePath: normalizeModuleName(method), payload, timeoutMs}),
      status: (): Promise<boolean> => core.invoke("bd_worker_status"),
      restart: (): Promise<boolean> => core.invoke("bd_worker_restart"),
      // delivery: (id: string, result: string): Promise<boolean> => core.invoke("bd_worker_delivery", {id, result}),
      modules: {
        list: (): Promise<string[]> => core.invoke("bd_worker_list_modules"),
        remove: (name: string): Promise<boolean> => core.invoke("bd_worker_remove_module", {name: normalizeModuleName(name)}),
        addFromBytes: (name: string, contents: U8[]): Promise<boolean> => core.invoke("bd_worker_add_module", {name: normalizeModuleName(name), contents}),
        add: (name: string, maxBytes?: U64): Promise<boolean> => core.invoke("bd_worker_pick_and_add_module", {maxBytes, defaultName: normalizeModuleName(name)}),
      },
      clearSandbox: (): Promise<boolean> => core.invoke("bd_worker_clear_all"),
      paths: (): Promise<{ externalModules: string; sandbox: string }> => core.invoke("bd_worker_paths"),
    };
  }
  
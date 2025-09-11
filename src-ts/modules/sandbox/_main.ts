import { AppInterface, BubbledeskAPI } from "@types";
import { U64, U8 } from "suffro-lib";
import { SandboxCallInput, SandboxInterface } from "./_types";
import { normalizeModuleInput, normalizeModuleName } from "@helpers";

export function buildSandbox(core: { invoke: BubbledeskAPI["invoke"] }): SandboxInterface {
    return {
      call: (input: SandboxCallInput): Promise<any> => core.invoke("bd_sandbox_call", {input: normalizeModuleInput(input)}),
      modules: {
        list: (): Promise<string[]> => core.invoke("bd_sandbox_list_modules"),
        remove: (name: string): Promise<boolean> => core.invoke("bd_sandbox_remove_module", {name: normalizeModuleName(name)}),
        addFromBytes: (name: string, contents: U8[]): Promise<boolean> => core.invoke("bd_sandbox_save_module", {name: normalizeModuleName(name), contents}),
        add: (name: string, maxBytes?: U64): Promise<boolean> => core.invoke("bd_sandbox_pick_and_save_module", {maxBytes, defaultName: normalizeModuleName(name)}),
      },
      clearAll: (): Promise<boolean> => core.invoke("bd_sandbox_clear_all"),
      paths: (): Promise<{ externalModules: string; sandbox: string }> => core.invoke("bd_sandbox_paths"),
      activeJobs: (): Promise<string[]> => core.invoke("bd_sandbox_list_active"),
      ttl: {
        sweep: (): Promise<number> => core.invoke("bd_sandbox_sweep"),
        set: (minutes: U64): Promise<number> => core.invoke("bd_sandbox_set_ttl_minutes", {minutes}),
        get: (): Promise<number> => core.invoke("bd_sandbox_get_ttl_minutes"),
      },
      concurrency: {
        setLimit: (limit: number): Promise<number> => core.invoke("bd_sandbox_set_concurrency_limit", {limit}),
        get: (): Promise<{limit:number;running:number;}> => core.invoke("bd_sandbox_get_concurrency"),
      }
    };
  }
  
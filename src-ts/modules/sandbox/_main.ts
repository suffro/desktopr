import { AppInterface, BubbledeskAPI } from "@types";
import { U64 } from "suffro-lib";
import { SandboxInterface } from "./_types";

export function buildSandbox(core: { invoke: BubbledeskAPI["invoke"] }): SandboxInterface {
    return {
      run: (input: string): Promise<string> => core.invoke("bd_sandbox_run", {input}),
      modules: {
        list: (): Promise<string[]> => core.invoke("bd_sandbox_list_modules"),
        remove: (name: string): Promise<boolean> => core.invoke("bd_sandbox_remove_module", {name}),
      },
      clearAll: (): Promise<boolean> => core.invoke("bd_sandbox_clear_all"),
      activeJobs: (): Promise<string[]> => core.invoke("bd_sandbox_list_active"),
      ttl: {
        sweep: (): Promise<number> => core.invoke("bd_sandbox_sweep"),
        set: (minutes: U64): Promise<number> => core.invoke("bd_sandbox_set_ttl_minutes", {minutes}),
        get: (): Promise<number> => core.invoke("bd_sandbox_set_ttl_minutes"),
      },
      concurrency: {
        setLimit: (limit: number): Promise<number> => core.invoke("bd_sandbox_set_concurrency_limit", {limit}),
        get: (): Promise<{limit:number;running:number;}> => core.invoke("bd_sandbox_get_concurrency"),
      },
    };
  }
  
import { U64, U8 } from "suffro-lib";


export type SandboxCaps = {
  timeout_ms?: number;     // execution timeout in ms
  memory_mb?: number;      // max linear memory in MB
  cpu_fuel?: number;       // optional (may be ignored in some runtimes)
  stdout_max_kb?: number;  // max stdout captured, in KB
};

export type SandboxCallPayload = {
  fn: string;     // execution timeout in ms
  args: number[];      // max linear memory in MB
  [key: string]: any;       // optional (may be ignored in some runtimes)
};

export type SandboxCallInput = {
  module_path: string;            // e.g. "bd_math_module.wasm"
  payload: SandboxCallPayload;    // payload passed to stdin
  caps?: SandboxCaps;             // optional execution caps
  env?: Record<string, string>;   // optional environment variables
};

export interface SandboxInterface {
    call: (input: SandboxCallInput) => Promise<string>;
    modules: {
        list: () => Promise<string[]>;
        remove: (name: string) => Promise<boolean>;
        addFromBytes: (name: string, contents: U8[]) => Promise<boolean>;
        add: (name: string, maxBytes?: U64 | undefined) => Promise<boolean>;
    };
    clearAll: () => Promise<boolean>;
    activeJobs: () => Promise<string[]>;
    paths: () => Promise<{
        externalModules: string;
        sandbox: string;
    }>;
    ttl: {
        sweep: () => Promise<number>;
        set: (minutes: U64) => Promise<number>;
        get: () => Promise<number>;
    };
    concurrency: {
        setLimit: (limit: number) => Promise<number>;
        get: () => Promise<{
            limit: number;
            running: number;
        }>;
    };
}

export type ModuleMethodInput = {
  "modulePath": string,
  "export": string,                // nome della funzione esportata
  "args": any[],                // argomenti
  "argTypes": string[],     // tipi: i32 | i64 | f32 | f64
  "env"?: any,        // opzionale
  "caps": {                       // opzionale: timeout/mem
    "timeoutMs": 5000,
    "memoryMb": 64
  }
}
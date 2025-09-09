import { U64, U8 } from "suffro-lib";

export interface SandboxInterface {
    call: (input: string) => Promise<string>;
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
import { U64 } from "suffro-lib";

export interface SandboxInterface {
    run: (input: string) => Promise<string>;
    modules: {
        list: () => Promise<string[]>;
        remove: (name: string) => Promise<boolean>;
    };
    clearAll: () => Promise<boolean>;
    activeJobs: () => Promise<string[]>;
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
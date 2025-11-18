import { TauriCore } from "@types";
export declare function extractCore(source: unknown): TauriCore | null;
export declare const ensureCore: () => Promise<TauriCore>;

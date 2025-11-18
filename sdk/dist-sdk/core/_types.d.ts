export type TauriCore = {
    invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T>;
};
export type TauriGlobal = TauriCore | {
    core: TauriCore;
};

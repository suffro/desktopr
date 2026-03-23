import { U64 } from "suffro-lib/utils";

export interface NetworkInterface {
    status: () => Promise<void>;
    ping: (url: string, timeout?: U64) => Promise<void>;
    resolve: (host: string) => Promise<void>;
    estimateBandwidth: (url?: string, sizeHintBytes?: U64, timeout?: U64) => Promise<void>;
    setMonitor: (interval: U64, targets?: string[]) => Promise<void>;
    stopMonitor: () => Promise<void>;
  };

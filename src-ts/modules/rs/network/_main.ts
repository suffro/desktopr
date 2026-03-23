import type { BubbledeskAPI, } from "../../../_types";
import { U64 } from "suffro-lib/utils";
import { NetworkInterface } from "./_types";

export function buildNetwork(core: { invoke: BubbledeskAPI["invoke"] }): NetworkInterface {
  return {
    status: (): Promise<void> => core.invoke("bd_network_get_status"),
    ping: (url: string, timeoutMs?: U64): Promise<void> => core.invoke("bd_network_ping", {url: url??"", timeoutMs}),
    resolve: (host: string): Promise<void> => core.invoke("bd_network_resolve", {host}),
    estimateBandwidth: (url?: string, sizeHintBytes?: U64, timeoutMs?: U64): Promise<void> => core.invoke("bd_network_bandwidth_estimate", {url, sizeHintBytes, timeoutMs}),
    setMonitor: (intervalMs?: U64, targets?: string[]): Promise<void> => core.invoke("bd_network_set_monitor", {intervalMs:intervalMs??3000, targets}),
    stopMonitor: (): Promise<void> => core.invoke("bd_network_stop_monitor"),
  };
}

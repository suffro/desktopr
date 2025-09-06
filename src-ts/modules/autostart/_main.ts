import type { BubbledeskAPI, } from "@types";
import { AutostartInterface, AutostartMode } from "./_types";

export function buildAutostart(core: { invoke: BubbledeskAPI["invoke"] }): AutostartInterface {
  return {
    enable: (): Promise<void> => core.invoke("bd_autostart_enable"),
    disable: (): Promise<void> => core.invoke("bd_autostart_disable"),
    isEnabled: (): Promise<void> => core.invoke("bd_autostart_status"),
    mode: {
      get: (): Promise<void> => core.invoke("bd_get_autostart_mode"),
      set: (mode: AutostartMode): Promise<void> => core.invoke("bd_set_autostart_mode", {mode}),
    }
  };
}
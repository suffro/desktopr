import { APP_URL } from "@constants";
import type { BubbledeskAPI } from "@types";

export function buildWindow(core: { invoke: BubbledeskAPI["invoke"] }) {
  return {
    minimize: (label?: string): Promise<void> => core.invoke("bd_win_minimize", { label: label??"main" }),
    maximizeToggle: (label?: string): Promise<void> => core.invoke("bd_win_maximize", { label: label??"main" }),
    fullscreen: (enable: boolean, label?: string): Promise<void> => core.invoke("bd_win_fullscreen", { enable, label: label??"main" }),
    new: (label: string, options?: {
      width?: number,
      height?: number,
      url?:string
    }): Promise<void> => core.invoke("bd_win_open", {
      label,
      url: (options?.url) ?? APP_URL,
      width: (options?.width) ?? 1024,
      height: (options?.height) ?? 700
    }),
    close: (label: string): Promise<void> => core.invoke("bd_win_close", { label }),
  };
}

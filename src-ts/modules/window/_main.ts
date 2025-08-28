import { APP_URL } from "@constants";
import type { BubbledeskAPI } from "@types";
import { getRandomString } from "suffro-lib";

export function buildWindow(core: { invoke: BubbledeskAPI["invoke"] }) {
  return {
    devTools: {
      toogle: (label?: string): Promise<void> => core.invoke("bd_toogle_devtools", { label: label??"main" }),
      open: (label?: string): Promise<void> => core.invoke("bd_open_devtools", { label: label??"main" }),
      clse: (label?: string): Promise<void> => core.invoke("bd_close_devtools", { label: label??"main" }),
    },
    minimize: (label?: string): Promise<void> => core.invoke("bd_win_minimize", { label: label??"main" }),
    maximizeToggle: (label?: string): Promise<void> => core.invoke("bd_win_maximize", { label: label??"main" }),
    fullscreen: (enable: boolean, label?: string): Promise<void> => core.invoke("bd_win_fullscreen", { enable, label: label??"main" }),
    new: (options?: {
      label?: string,
      fullscreen?: boolean,
      url?:string
    }): Promise<void> => core.invoke("bd_win_open", {
      label: (options?.label)??getRandomString({
        length: 6,
        prefix: "w_"
      }),
      fullscreen: (options?.fullscreen) || false,
      url: (options?.url) ?? "",
    }),
    close: (label: string): Promise<void> => core.invoke("bd_win_close", { label }),
  };
}

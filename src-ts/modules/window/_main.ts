import type { BubbledeskAPI, WindowInterface } from "@types";

export function buildWindow(core: { invoke: BubbledeskAPI["invoke"] }): WindowInterface {
  const randomWindowLabel: string = `w_${Math.random().toString(36).substring(2, 2 + 8)}`
  return {
    devTools: {
      toggle: (label?: string): Promise<void> => core.invoke("bd_toggle_devtools", { label: label??"main" }),
      open: (label?: string): Promise<void> => core.invoke("bd_open_devtools", { label: label??"main" }),
      close: (label?: string): Promise<void> => core.invoke("bd_close_devtools", { label: label??"main" }),
    },
    minimize: (label?: string): Promise<void> => core.invoke("bd_win_minimize", { label: label??"main" }),
    maximizeToggle: (label?: string): Promise<void> => core.invoke("bd_win_maximize", { label: label??"main" }),
    fullscreen: (enable: boolean, label?: string): Promise<void> => core.invoke("bd_win_fullscreen", { enable, label: label??"main" }),
    new: (options?: {
      label?: string,
      fullscreen?: boolean,
      url?:string
    }): Promise<void> => core.invoke("bd_win_open", {
      label: (options?.label)??randomWindowLabel,
      fullscreen: (options?.fullscreen) || false,
      url: (options?.url) ?? "",
    }),
    close: (label: string): Promise<void> => core.invoke("bd_win_close", { label }),
  };
}

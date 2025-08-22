import type { BubbledeskAPI } from "@types";

export function buildWindow(core: { invoke: BubbledeskAPI["invoke"] }) {
  return {
    minimize: (): Promise<void> => core.invoke("bd_win_minimize"),
    maximizeToggle: (): Promise<void> => core.invoke("bd_win_maximize"),
    fullscreen: (enable: boolean): Promise<void> => core.invoke("bd_win_fullscreen", { enable }),
  };
}

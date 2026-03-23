import { COMAPNION_WINDOW_LABEL_PREFIX } from "../../../_constants";
import { DesktoprAPI } from "../../../desktopr/_types";
import { wait } from "suffro-lib/utils";

export const tauriReadyCheck = (): boolean =>
  typeof window !== "undefined" &&
  (window as any).__TAURI__ &&
  (window as any).Desktopr;

export const waitTauri = async () => {
  const interval: number = 500;
  let counter: number = 0;
  while (counter <= 30000 && !tauriReadyCheck()) {
    await wait(interval);
    counter = counter + interval;
  }
};

export const newWindow = async (
  core: { invoke: DesktoprAPI["invoke"] },
  options?: {
    label?: string;
    fullscreen?: boolean;
    url?: string;
  }
) => {
  if (
    options?.label &&
    options.label.trim().toLowerCase().startsWith((COMAPNION_WINDOW_LABEL_PREFIX).trim().toLowerCase())
  )
    throw new Error(
      `[Reserved window label] ${COMAPNION_WINDOW_LABEL_PREFIX}* is an app reserved label`
    );

  const randomWindowLabel: string = `w_${Math.random()
    .toString(36)
    .substring(2, 2 + 8)}`;

  core.invoke("dtr_win_open", {
    label: options?.label ?? randomWindowLabel,
    fullscreen: options?.fullscreen || false,
    url: options?.url ?? "",
  });
};

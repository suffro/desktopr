export * from "./core/_helpers";
export * from "./bubbledesk/_helpers";
export * from "./modules/rs/files/_helpers";
export * from "./modules/rs/events/_helpers";
export * from "./modules/rs/fs/_helpers";
export * from "./modules/rs/clipboard/_helpers";
export * from "./modules/rs/shortcuts/_helpers";
export * from "./modules/rs/notifications/_helpers";
export * from "./modules/rs/app/_helpers";
export * from "./modules/rs/window/_helpers";
export * from "./modules/rs/dragdrop/_helpers";
export * from "./modules/rs/menu/_helpers";
export * from "./modules/rs/diagnostics/_helpers";
export * from "./modules/rs/network/_helpers";
export * from "./modules/rs/autostart/_helpers";
export * from "./modules/rs/badge/_helpers";
export * from "./modules/rs/worker/_helpers";
export * from "./modules/rs/contextMenu/_helpers";
export * from "./modules/rs/companion/_helpers";


import { wait } from "suffro-lib";

export const tauriReadyCheck = (): boolean => (typeof window !== "undefined" && ((window as any).__TAURI__) && ((window as any).Bubbledesk));

export const waitTauri = async () => {
  const interval: number=500;
  let counter: number=0;
  while(counter<=30000 && !tauriReadyCheck()){
    await wait(interval);
    counter=counter+interval;
  }
}


/** 
 * Detects if running inside a native Tauri WebView.
 * - Fast path: checks global objects (__TAURI__ / __TAURI_INTERNALS__)
 * - Fallback: tries invoking a Tauri command (bd_app_info)
 */
export async function isTauri(): Promise<boolean> {
  // --- Fast sync path ---
  if (
    typeof window !== "undefined" &&
    (typeof (window as any).__TAURI__ !== "undefined" ||
     typeof (window as any).__TAURI_INTERNALS__ !== "undefined")
  ) {
    return true;
  }

  // --- Safe async fallback ---
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("bd_app_info");
    return true;
  } catch {
    return false;
  }
}
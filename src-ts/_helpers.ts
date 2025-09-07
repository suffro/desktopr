export * from "./core/_helpers";
export * from "./bubbledesk/_helpers";
export * from "./modules/files/_helpers";
export * from "./modules/events/_helpers";
export * from "./modules/fs/_helpers";
export * from "./modules/clipboard/_helpers";
export * from "./modules/shortcuts/_helpers";
export * from "./modules/notifications/_helpers";
export * from "./modules/app/_helpers";
export * from "./modules/window/_helpers";
export * from "./modules/dragdrop/_helpers";
export * from "./modules/menu/_helpers";
export * from "./modules/diagnostics/_helpers";
export * from "./modules/network/_helpers";
export * from "./modules/autostart/_helpers";
export * from "./modules/badge/_helpers";
export * from "./modules/sandbox/_helpers";

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
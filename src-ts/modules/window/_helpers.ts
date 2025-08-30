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
import type { DesktoprAPI } from "./_types";
declare global { interface Window { Desktopr?: DesktoprAPI, __TAURI__?: any, __DESKTOPR_APP_VERSION__?: string} }
export {};

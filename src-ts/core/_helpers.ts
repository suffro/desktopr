import { TauriCore } from "../_types";

export function extractCore(source: unknown): TauriCore | null {
    if (!source) return null;
    if (typeof source === "object" && "invoke" in (source as any)) {
      const core = source as TauriCore;
      if (typeof core.invoke === "function") return core;
    }
    if (typeof source === "object" && "core" in (source as any)) {
      const maybe = (source as any).core;
      if (maybe && typeof maybe.invoke === "function") return maybe as TauriCore;
    }
    return null;
  }
  
  export const ensureCore = (): Promise<TauriCore> =>
    new Promise((resolve, reject) => {
      const deadline = Date.now() + 10_000;
      (function tick() {
        const core = extractCore((window as any)?.__TAURI__);
        if (core) return resolve(core);
        if (Date.now() > deadline) return reject(new Error("Tauri core.invoke not available"));
        requestAnimationFrame(tick);
      })();
    });

// [Unverified] Universal Proxy for window.__TAURI__
export const windowTauriProxy = new Proxy(
  {},
  {
    get(_target, prop) {
      const tauri = window.__TAURI__;

      if (!tauri) {
        console.warn("window.__TAURI__ is not available");
        return undefined;
      }

      const value = tauri[prop];

      // If it's a function, bind correct 'this'
      if (typeof value === "function") {
        return value.bind(tauri);
      }

      return value;
    }
  }
);
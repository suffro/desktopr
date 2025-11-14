// src-ts/sdk/bubbledesk.ts
import type { BubbledeskAPI } from "@types";

// Internal helper: get a safe window reference
function getWindow(): Window {
  if (typeof window === "undefined") {
    // Avoid using the bridge in SSR or in non-browser environments
    throw new Error("[Bubbledesk] window is not defined. Are you running in SSR?");
  }
  return window;
}

// Internal helper: access the real global bridge
function getGlobalBridge(): BubbledeskAPI {
  const w = getWindow() as any;
  const bridge = w.Bubbledesk;

  if (!bridge) {
    // Bubbledesk bridge is not yet injected by the wrapper
    throw new Error(
      "[Bubbledesk] window.Bubbledesk is not available. Is the desktop wrapper loaded?"
    );
  }

  return bridge as BubbledeskAPI;
}

// Public helper to check if the native Bubbledesk bridge is available.
// This must never throw, even in SSR or when running outside the wrapper.
export function isBubbledeskAvailable(): boolean {
  if (typeof window === "undefined") {
    // In SSR or non-browser environments the bridge is not available.
    return false;
  }

  const w = window as any;
  return !!w.Bubbledesk;
}

// Public SDK object.
// At runtime, this is just a Proxy that forwards everything to window.Bubbledesk,
// but from the developer point of view it is strongly typed as BubbledeskAPI.
export const Bubbledesk: BubbledeskAPI = new Proxy({} as BubbledeskAPI, {
  get(_target, prop, _receiver) {
    const bridge = getGlobalBridge();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const value = (bridge as any)[prop];

    // If the property is a function, bind it to the original object
    if (typeof value === "function") {
      return value.bind(bridge);
    }

    return value;
  },

  set(_target, prop, value) {
    const bridge = getGlobalBridge();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (bridge as any)[prop] = value;
    return true;
  }
});
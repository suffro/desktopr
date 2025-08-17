(() => {
    // Guard if already defined (avoid double-define on SPA navigations)
    if (window.Bubbledesk) return;
  
    // Minimal, sync-available bridge
    const api = {
      isAvailable: true,
      version: "0.1.0",
      // Resolve immediately: native script runs at document start
      ready: Promise.resolve(),
  
      // Optional: proxy to native commands if disponibili
      // Nota: lascio il check sul global Tauri, così non dipendiamo da import esterni.
      invoke: async (cmd, payload) => {
        const g = /** @type {any} */ (window);
        const core = g.__TAURI__?.core || g.__TAURI__;
        if (!core?.invoke) {
          throw new Error("Tauri invoke not available in this context.");
        }
        return core.invoke(cmd, payload);
      }
    };
  
    Object.defineProperty(window, "Bubbledesk", {
      value: api,
      enumerable: false, // non sporca enumerazioni
      configurable: false,
      writable: false
    });
  
    // Per debug: commenta in prod
    // console.debug("[Bubbledesk] bridge injected", api);
  })();
  
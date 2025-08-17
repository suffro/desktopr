(() => {
    if (window.Bubbledesk) return;
  
    // robust core detector (supports IPC fallback on Safari)
    const ensureCore = () =>
      new Promise((resolve, reject) => {
        const deadline = Date.now() + 10000; // 10s
        const check = () => {
          // global tauri object (v2)
          const g = /** @type {any} */ (window);
          const core = g.__TAURI__?.core || g.__TAURI__;
          if (core?.invoke) return resolve(core);
  
          if (Date.now() > deadline) {
            return reject(new Error("Tauri core.invoke not available"));
          }
          // use rAF to yield to browser/event loop (better on Safari)
          requestAnimationFrame(check);
        };
        check();
      });
  
    const api = {
      get isAvailable() { return true; },
      version: "0.1.0",
  
      // ready as a getter that resolves when core is ready (no early capture)
      get ready() { return ensureCore().then(() => {}); },
  
      async invoke(cmd, payload) {
        const core = await ensureCore();
        return core.invoke(cmd, payload);
      },
  
      async on(event, handler) {
        const g = /** @type {any} */ (window);
        const ev = g.__TAURI__?.event || g.__TAURI__;
        if (!ev?.listen) throw new Error("Tauri event API unavailable.");
        return ev.listen(event, handler);
      }
    };
  
    Object.defineProperty(window, "Bubbledesk", {
      value: api,
      enumerable: false,
      configurable: false,
      writable: false
    });
  })();
  
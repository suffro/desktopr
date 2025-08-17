(() => {
    if (window.Bubbledesk) return;
  
    const waitForInvoke = () => new Promise((resolve, reject) => {
      const t0 = Date.now(), max = 5000;
      (function tick() {
        const g = window;
        const core = g.__TAURI__?.core || g.__TAURI__;
        if (core?.invoke) return resolve();
        if (Date.now() - t0 > max) return reject(new Error("Tauri core.invoke not available"));
        queueMicrotask(tick);
      })();
    });
  
    const api = {
      isAvailable: true,
      version: "0.1.0",
      ready: waitForInvoke(),
      invoke: async (cmd, payload) => {
        const g = window;
        const core = g.__TAURI__?.core || g.__TAURI__;
        if (!core?.invoke) throw new Error("Tauri invoke not available in this context.");
        return core.invoke(cmd, payload);
      },
      on: async (event, handler) => {
        const g = window;
        const ev = g.__TAURI__?.event || g.__TAURI__;
        if (!ev?.listen) throw new Error("Tauri event API unavailable.");
        return ev.listen(event, handler);
      }
    };
  
    Object.defineProperty(window, "Bubbledesk", {
      value: api, enumerable: false, configurable: false, writable: false
    });
  })();
  
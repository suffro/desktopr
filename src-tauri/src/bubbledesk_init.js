(() => {
  if (window.Bubbledesk) return;

  const ensureCore = () => new Promise((resolve, reject) => {
    const deadline = Date.now() + 10000;
    (function tick() {
      const g = window; const core = g.__TAURI__?.core || g.__TAURI__;
      if (core?.invoke) return resolve(core);
      if (Date.now() > deadline) return reject(new Error("Tauri core.invoke not available"));
      requestAnimationFrame(tick);
    })();
  });

  const api = {
    get isAvailable() { return true; },
    version: "0.2.0",
    get ready() { return ensureCore().then(() => true); },
    async invoke(cmd, payload) { const core = await ensureCore(); return core.invoke(cmd, payload); },

    // opzionali: helpers typed
    notifications: {
      state: () => api.invoke("bd_notification_state"),
      request: () => api.invoke("bd_request_permission"),
      show: (title, body) => api.invoke("bd_notify", { title, body }),
    },
    clipboard: {
      readText: () => api.invoke("bd_clipboard_read"),
      writeText: (text) => api.invoke("bd_clipboard_write", { text }),
    },
    files: {
      open: (option={multi:false}) => api.invoke("bd_file_open", { multi: option.multi }),
      save: (default_name=null) => api.invoke("bd_file_save", { default_name }),
    },
    app: { info: () => api.invoke("bd_app_info") },
    window: {
      minimize: () => api.invoke("bd_win_minimize"),
      maximizeToggle: () => api.invoke("bd_win_maximize"),
      fullscreen: (enable) => api.invoke("bd_win_fullscreen", { enable }),
    }
  };

  Object.defineProperty(window, "Bubbledesk", { value: api, enumerable:false, configurable:false, writable:false });
})();

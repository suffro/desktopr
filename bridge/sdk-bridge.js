(function (global) {
  function hasTauri() {
    return !!global.__TAURI__ && !!global.__TAURI__.core && typeof global.__TAURI__.core.invoke === "function";
  }

  // getter reattivo: legge __TAURI__ al momento dell’accesso
  let _version = "0.1.0"; // oppure sostituisci via build con env!("CARGO_PKG_VERSION")
  const Bubbledesk = {
    get isAvailable() { return hasTauri(); },
    get version() { return _version; },

    on(event, handler) {
      if (!hasTauri()) throw new Error("Bubbledesk SDK not available");
      return global.__TAURI__.event.listen(event, ({ payload }) => handler(payload));
    },

    async notify(title, body) {
      if (!hasTauri()) throw new Error("Bubbledesk SDK not available");
      return global.__TAURI__.core.invoke("plugin:notification|send", { title, body });
    },

    window: {
      show() { if (!hasTauri()) throw new Error("Bubbledesk SDK not available"); return global.__TAURI__.window.getCurrent().show(); },
      hide() { if (!hasTauri()) throw new Error("Bubbledesk SDK not available"); return global.__TAURI__.window.getCurrent().hide(); },
      setFullscreen(v) { if (!hasTauri()) throw new Error("Bubbledesk SDK not available"); return global.__TAURI__.window.getCurrent().setFullscreen(v); }
    }
  };

  // piccolo poll per “annunciare” quando diventa disponibile
  if (!hasTauri()) {
    const iv = setInterval(() => {
      if (hasTauri()) {
        clearInterval(iv);
        try { global.__TAURI__.event.emit("bubbledesk:ready", { sdk: _version }); } catch {}
      }
    }, 200);
  }

  global.Bubbledesk = Bubbledesk;
})(globalThis);

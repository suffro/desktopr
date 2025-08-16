(function (global) {
    // Available only if this page is loaded inside the Bubbledesk wrapper and the origin is allowed by capabilities
    const hasTauri = !!global.__TAURI__ && !!global.__TAURI__.core && typeof global.__TAURI__.core.invoke === "function";
  
    function requireTauri() {
      if (!hasTauri) throw new Error("Bubbledesk SDK not available in this context");
      return global.__TAURI__;
    }
  
    const Bubbledesk = {
      version: "{{APP_VERSION}}",
      isAvailable: hasTauri,
  
      // Simple wrappers (extend as needed)
      async notify(title, body) {
        const api = requireTauri();
        // uses notification plugin routing
        return api.core.invoke("plugin:notification|send", { title, body });
      },
  
      on(event, handler) {
        return requireTauri().event.listen(event, ({ payload }) => handler(payload));
      },
  
      window: {
        show() { return requireTauri().window.getCurrent().show(); },
        hide() { return requireTauri().window.getCurrent().hide(); },
        setFullscreen(v) { return requireTauri().window.getCurrent().setFullscreen(v); }
      }
    };
  
    global.Bubbledesk = Bubbledesk;
  })(globalThis);
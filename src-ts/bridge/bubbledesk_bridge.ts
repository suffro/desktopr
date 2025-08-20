/* Bubbledesk bridge bootstrap (TypeScript, Tauri v2)
 * - Waits for __TAURI__.core.invoke (or __TAURI__.invoke)
 * - Exposes window.Bubbledesk with typed helpers
 * - Safe to import multiple times (idempotent)
 * NOTE: keep comments in English as per project convention
 */

type TauriCore = {
  invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T>;
};

type TauriGlobal = TauriCore | { core: TauriCore };

type OpenResult = { paths: string[] };

type BubbledeskAPI = {
  /** true if the bridge was injected */
  readonly isAvailable: boolean;
  /** bridge version */
  readonly version: string;
  /** resolves when __TAURI__.core.invoke is available */
  readonly ready: Promise<true>;
  /** raw invoke */
  invoke<T = unknown>(
    cmd: string,
    payload?: Record<string, unknown>
  ): Promise<T>;

  // Helpers
  notifications: {
    state: () => Promise<"granted" | "denied" | "default" | string>;
    request: () => Promise<"granted" | "denied" | string>;
    show: (title: string, body?: string) => Promise<void>;
  };
  clipboard: {
    readText: () => Promise<string>;
    writeText: (text: string) => Promise<void>;
  };
  files: {
    open: (option?: { multi?: boolean }) => Promise<OpenResult>;
    /** empty string if canceled */
    save: (default_name?: string | null) => Promise<string>;
  };
  app: {
    info: () => Promise<unknown>; // refine if you have a concrete shape
  };
  window: {
    minimize: () => Promise<void>;
    maximizeToggle: () => Promise<void>;
    fullscreen: (enable: boolean) => Promise<void>;
  };
  events: {
    emit: (event: string, payload?: unknown) => Promise<unknown>;
    emitTo: (
      window_label: string,
      event: string,
      payload?: unknown
    ) => Promise<unknown>;
    listen: (
      event: string,
      handler: (payload: any) => void
    ) => Promise<() => any>;
  };
};

interface Window {
  __TAURI__?: TauriGlobal;
  Bubbledesk?: BubbledeskAPI;
}

/** Safely extract the Tauri core from window.__TAURI__ */
function extractCore(source: unknown): TauriCore | null {
  if (!source) return null;

  // case 1: source is already TauriCore
  if (typeof source === "object" && "invoke" in (source as any)) {
    const core = source as TauriCore;
    if (typeof core.invoke === "function") return core;
  }

  // case 2: source is { core: TauriCore }
  if (typeof source === "object" && "core" in (source as any)) {
    const maybe = (source as any).core;
    if (maybe && typeof maybe.invoke === "function") {
      return maybe as TauriCore;
    }
  }

  return null;
}

/** Promise that resolves when Tauri core is available (with a timeout) */
const ensureCore = (): Promise<TauriCore> =>
  new Promise((resolve, reject) => {
    const deadline = Date.now() + 10000;

    (function tick() {
      const core = extractCore((window as Window | undefined)?.__TAURI__);

      if (core) return resolve(core);
      if (Date.now() > deadline) {
        return reject(new Error("Tauri core.invoke not available"));
      }
      requestAnimationFrame(tick);
    })();
  });

/** Idempotent bootstrap */
(() => {
  if (typeof window === "undefined" || window.Bubbledesk) return;

  const api: BubbledeskAPI = {
    get isAvailable() {
      return true;
    },
    version: "0.2.1",
    get ready() {
      return ensureCore().then(() => true as const);
    },
    async invoke<T = unknown>(cmd: string, payload?: Record<string, unknown>) {
      const core = await ensureCore();
      return core.invoke<T>(cmd, payload);
    },

    notifications: {
      state: () => api.invoke("bd_notification_state"),
      request: () => api.invoke("bd_request_permission"),
      show: (title: string, body?: string) =>
        api.invoke("bd_notify", { title, body }),
    },

    clipboard: {
      readText: () => api.invoke("bd_clipboard_read"),
      writeText: (text: string) => api.invoke("bd_clipboard_write", { text }),
    },

    files: {
      open: (option) =>
        api.invoke<OpenResult>("bd_file_open", {
          multi: option?.multi ?? false,
        }),
      // NOTE: Rust currently returns String ("" on cancel). If you switch to Option<String>,
      // change this signature to Promise<string | null>.
      save: (default_name) =>
        api.invoke<string>("bd_file_save", {
          default_name: default_name ?? null,
        }),
    },

    app: {
      info: () => api.invoke("bd_app_info"),
    },

    window: {
      minimize: () => api.invoke("bd_win_minimize"),
      maximizeToggle: () => api.invoke("bd_win_maximize"),
      fullscreen: (enable: boolean) =>
        api.invoke("bd_win_fullscreen", { enable }),
    },
    events: {
      emit: (event: string, payload?: unknown) =>
        api.invoke("bd_event_emit", { event, payload }),

      emitTo: (window_label: string, event: string, payload?: unknown) =>
        api.invoke("bd_event_emit_to", { window_label, event, payload }),

      listen: async (event: string, handler: (payload: any) => void) => {
        await api.ready;
        const evt = (window as any).__TAURI__?.event;
        if (!evt?.listen) throw new Error("Tauri event API not available");
        const unlisten = await evt.listen(event, (e: any) =>
          handler(e?.payload)
        );
        return () => unlisten();
      },
    },
  };

  Object.defineProperty(window, "Bubbledesk", {
    value: api,
    enumerable: false,
    configurable: false,
    writable: false,
  });
})();

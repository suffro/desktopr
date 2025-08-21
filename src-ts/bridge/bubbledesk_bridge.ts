/* Bubbledesk bridge bootstrap (TypeScript, Tauri v2)
 * - Waits for __TAURI__.core.invoke (or __TAURI__.invoke)
 * - Exposes window.Bubbledesk with typed helpers
 * - Safe to import multiple times (idempotent)
 * NOTE: keep comments in English as per project convention
 */

const _gs = () => {
  const g = (window as any).__TAURI__?.globalShortcut;
  if (!g) throw new Error("Global Shortcut plugin not available");
  return g;
};

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
    on: (event: string, handler: (payload: any) => void) => Promise<any>;
    once: (event: string) => Promise<any>;
    onMany: (
      events: string[],
      handler: (name: string, payload: any) => void
    ) => Promise<() => any>;
    onShortcut: (
      handler: (payload: any) => void,
      options?: { emitEvent?: boolean }
    ) => Promise<() => any>;
    onDragDrop: (
      handler: (name: string, payload: any) => void,
      options?: { includeHover?: boolean }
    ) => Promise<() => void>;
  };
  globalShortcut: {
    register: (accelerator: any, cb: any) => Promise<void>;
    unregister: (accelerator: any) => Promise<void>;
    unregisterAll: () => Promise<void>;
    isRegistered: (accelerator: any) => Promise<any>;
  };
  fs: {
    listDir: (rel: string) => Promise<unknown>;
    mkdir: (rel: string) => Promise<unknown>;
    rm: (rel: string, recursive?: boolean) => Promise<unknown>;
    stat: (rel: string) => Promise<unknown>;
    base: string;
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

const listenForEvent = async (
  event: string,
  handler: (payload: any) => void
) => {
  const tauri = (window as any).__TAURI__;
  const eventApi = tauri?.event;
  if (!eventApi?.listen) throw new Error("Tauri event API not available");
  const unlisten = await eventApi.listen(event, (e: any) =>
    handler(e?.payload)
  );
  return () => unlisten();
};

const getCurrentWebviewWindow = () => {
  const tauri = (window as any).__TAURI__;
  return tauri?.window?.getCurrent?.();
};
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

      on: async (event: string, handler: (payload: any) => void) =>
        await listenForEvent(event, handler),

      once: (event: string) =>
        new Promise<any>(async (resolve) => {
          const off = await listenForEvent(event, (p) => {
            off();
            resolve(p);
          });
        }),

      onMany: async (
        events: string[],
        handler: (name: string, payload: any) => void
      ) => {
        const offs = await Promise.all(
          events.map((name) => listenForEvent(name, (p) => handler(name, p)))
        );
        return () => offs.forEach((off) => off());
      },
      onShortcut: async (handler: (payload: any) => void) =>
        await listenForEvent("shortcut:event", handler),

      onDragDrop: async (
        handler: (
          name: string,
          payload: {
            kind: "enter" | "drop" | "cancel" | "hover" | string;
            path: string[];
            position: { x: number; y: number };
            [key: string]: any;
          }
        ) => void,
        options?: { includeHover?: boolean }
      ) => {
        const evs = ["dragdrop:enter", "dragdrop:drop", "dragdrop:cancel"];
        if (options?.includeHover) evs.push("dragdrop:hover");
        const offs = await Promise.all(
          evs.map((name) => listenForEvent(name, (p) => handler(name, p)))
        );
        return () => offs.forEach((off) => off());
      },
    },
    globalShortcut: {
      /** eg: "CommandOrControl+Alt+T" */
      register: async (
        accelerator: any,
        cb: Function,
        options?: { emitEvent?: boolean }
      ) => {
        const gs = _gs();
        await gs.register(accelerator, async (e: any) => {
          const _playload: {
            accelerator: any;
            shortcut: string;
            id: number;
            state: "Pressed" | "Released" | string;
            [key: string]: any;
          } = { accelerator, ...e };
          if (options?.emitEvent)
            await api.events.emit("shortcut:event", _playload);
          cb(_playload);
        });
      },
      unregister: async (accelerator: any) => {
        const gs = _gs();
        const reg = await gs.isRegistered(accelerator);
        if (reg) await gs.unregister(accelerator);
      },
      unregisterAll: async () => {
        const gs = _gs();
        await gs.unregisterAll();
      },
      isRegistered: async (accelerator: any) => {
        const gs = _gs();
        return gs.isRegistered(accelerator);
      },
    },
    fs: {
      listDir: (rel: string) => api.invoke("fs_list_dir", { rel }),
      mkdir: (rel: string) => api.invoke("fs_mkdir", { rel }),
      rm: (rel: string, recursive = false) =>
        api.invoke("fs_rm", { rel, recursive }),
      stat: (rel: string) => api.invoke("fs_stat", { rel }),
      base: ".cache", // convenzione: tutto è relativo a base_dir lato Rust
    },
  };

  Object.defineProperty(window, "Bubbledesk", {
    value: api,
    enumerable: false,
    configurable: false,
    writable: false,
  });
})();

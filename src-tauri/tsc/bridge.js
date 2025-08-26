"use strict";
var Bridge = (() => {
  // ../src-ts/core/_helpers.ts
  function extractCore(source) {
    if (!source) return null;
    if (typeof source === "object" && "invoke" in source) {
      const core = source;
      if (typeof core.invoke === "function") return core;
    }
    if (typeof source === "object" && "core" in source) {
      const maybe = source.core;
      if (maybe && typeof maybe.invoke === "function") return maybe;
    }
    return null;
  }
  var ensureCore = () => new Promise((resolve, reject) => {
    const deadline = Date.now() + 1e4;
    (function tick() {
      const core = extractCore(window?.__TAURI__);
      if (core) return resolve(core);
      if (Date.now() > deadline) return reject(new Error("Tauri core.invoke not available"));
      requestAnimationFrame(tick);
    })();
  });

  // ../src-ts/modules/events/_helpers.ts
  var listenForEvent = async (event, handler) => {
    const tauri = window.__TAURI__;
    const eventApi = tauri?.event;
    if (!eventApi?.listen) throw new Error("Tauri event API not available");
    const unlisten = await eventApi.listen(event, (e) => handler(e?.payload));
    return () => unlisten();
  };

  // ../src-ts/modules/shortcuts/_helpers.ts
  var tauriGlobalShortcut = () => {
    const g = window.__TAURI__?.globalShortcut;
    if (!g) throw new Error("Global Shortcut plugin not available");
    return g;
  };

  // ../src-ts/core/_main.ts
  function buildCore() {
    return {
      get ready() {
        return ensureCore().then(() => true);
      },
      async invoke(cmd, payload) {
        const core = await ensureCore();
        return core.invoke(cmd, payload);
      }
    };
  }

  // ../src-ts/modules/files/_main.ts
  function buildFiles(core) {
    return {
      open: (option) => core.invoke("bd_file_open", { multi: option?.multi ?? false }),
      save: (default_name) => core.invoke("bd_file_save", { default_name: default_name ?? null })
    };
  }

  // ../src-ts/modules/events/_main.ts
  function buildEvents(core) {
    return {
      emit: (event, payload) => core.invoke("bd_event_emit", { event, payload }),
      emitTo: (window_label, event, payload) => core.invoke("bd_event_emit_to", { window_label, event, payload }),
      on: async (event, handler) => listenForEvent(event, handler),
      once: (event) => new Promise(async (resolve) => {
        const off = await listenForEvent(event, (p) => {
          off();
          resolve(p);
        });
      }),
      onMany: async (events, handler) => {
        const offs = await Promise.all(events.map((n) => listenForEvent(n, (p) => handler(n, p))));
        return () => offs.forEach((off) => off());
      },
      onDeeplink: async (handler) => listenForEvent("deeplink", handler),
      onShortcut: async (handler) => listenForEvent("shortcut:event", handler),
      onDragDrop: async (handler, options) => {
        const evs = ["dragdrop:enter", "dragdrop:drop", "dragdrop:cancel"];
        if (options?.includeHover) evs.push("dragdrop:hover");
        const offs = await Promise.all(evs.map((n) => listenForEvent(n, (p) => handler(n, p))));
        return () => offs.forEach((off) => off());
      }
    };
  }

  // ../src-ts/modules/fs/_main.ts
  function scope(core, permanent) {
    return {
      listDir: (rel = "") => core.invoke("fs_list_dir", { rel, permanent }),
      mkdir: (rel) => core.invoke("fs_mkdir", { rel, permanent }),
      rm: (rel, recursive = false) => core.invoke("fs_rm", { rel, recursive, permanent }),
      stat: (rel = "") => core.invoke("fs_stat", { rel, permanent }),
      writeText: (rel, contents, opts) => core.invoke("fs_write_text", {
        rel,
        permanent,
        contents,
        createDirs: opts?.createDirs,
        append: opts?.append
      }),
      readText: (rel) => core.invoke("fs_read_text", { rel, permanent }),
      writeBytes: (rel, base64, opts) => core.invoke("fs_write_bytes", {
        rel,
        permanent,
        dataBase64: base64,
        createDirs: opts?.createDirs
      }),
      readBytes: (rel) => core.invoke("fs_read_bytes", { rel, permanent }),
      exists: (rel) => core.invoke("fs_exists", { rel, permanent }),
      move: (src, dest, opts) => core.invoke("fs_move", {
        src,
        dest,
        permanent,
        createDirs: opts?.createDirs,
        overwrite: opts?.overwrite
      }),
      copy: (src, dest, opts) => core.invoke("fs_copy", {
        src,
        dest,
        permanent,
        recursive: opts?.recursive,
        createDirs: opts?.createDirs,
        overwrite: opts?.overwrite
      }),
      path: async () => {
        const p = await core.invoke("fs_paths");
        return permanent ? p.data : p.cache;
      },
      base: permanent ? ".data" : ".cache"
    };
  }
  function buildFs(core) {
    const cache = scope(core, false);
    const data = scope(core, true);
    return {
      cache: { ...cache, clear: () => core.invoke("fs_clear_cache") },
      data,
      paths: () => core.invoke("fs_paths"),
      base: { cache: ".cache", data: ".data" }
    };
  }

  // ../src-ts/modules/clipboard/_main.ts
  function buildClipboard(core) {
    return {
      readText: () => core.invoke("bd_clipboard_read"),
      writeText: (text) => core.invoke("bd_clipboard_write", { text })
    };
  }

  // ../src-ts/modules/shortcuts/_main.ts
  function buildShortcuts(core) {
    return {
      register: async (accelerator, cb, options) => {
        const gs = tauriGlobalShortcut();
        await gs.register(accelerator, async (e) => {
          const payload = { accelerator, ...e };
          if (options?.emitEvent) await core.invoke("bd_event_emit", { event: "shortcut:event", payload });
          cb(payload);
        });
      },
      unregister: async (accelerator) => {
        const gs = tauriGlobalShortcut();
        const reg = await gs.isRegistered(accelerator);
        if (reg) await gs.unregister(accelerator);
      },
      unregisterAll: async () => {
        const gs = tauriGlobalShortcut();
        await gs.unregisterAll();
      },
      isRegistered: async (accelerator) => {
        const gs = tauriGlobalShortcut();
        return gs.isRegistered(accelerator);
      }
    };
  }

  // ../src-ts/modules/notifications/_main.ts
  function buildNotifications(core) {
    return {
      state: () => core.invoke("bd_notification_state"),
      request: () => core.invoke("bd_request_permission"),
      show: (title, body) => core.invoke("bd_notify", { title, body })
    };
  }

  // ../src-ts/modules/app/_main.ts
  function buildAppInfo(core) {
    return {
      info: () => core.invoke("bd_app_info")
    };
  }

  // ../src-ts/bridge.constants.json
  var bridge_constants_default = {
    appUrl: "http://blank.html",
    appVersion: "0.2.1"
  };

  // ../src-ts/_constants.ts
  var APP_URL = bridge_constants_default.appUrl;
  var APP_VERSION = bridge_constants_default.appVersion;

  // ../src-ts/modules/window/_main.ts
  function buildWindow(core) {
    return {
      minimize: (label) => core.invoke("bd_win_minimize", { label: label ?? "main" }),
      maximizeToggle: (label) => core.invoke("bd_win_maximize", { label: label ?? "main" }),
      fullscreen: (enable, label) => core.invoke("bd_win_fullscreen", { enable, label: label ?? "main" }),
      new: (label, options) => core.invoke("bd_win_open", {
        label,
        url: options?.url ?? APP_URL,
        width: options?.width ?? 1024,
        height: options?.height ?? 700
      }),
      close: (label) => core.invoke("bd_win_close", { label })
    };
  }

  // ../src-ts/bridge.ts
  (() => {
    if (typeof window === "undefined" || window.Bubbledesk) return;
    const core = buildCore();
    const api = {
      get isAvailable() {
        return true;
      },
      version: APP_VERSION,
      get ready() {
        return core.ready;
      },
      invoke: core.invoke,
      notifications: buildNotifications(core),
      clipboard: buildClipboard(core),
      files: buildFiles(core),
      app: buildAppInfo(core),
      window: buildWindow(core),
      events: buildEvents(core),
      globalShortcut: buildShortcuts(core),
      fs: buildFs(core)
    };
    Object.defineProperty(window, "Bubbledesk", {
      value: api,
      enumerable: false,
      configurable: false,
      writable: false
    });
  })();
})();
//# sourceMappingURL=bridge.js.map

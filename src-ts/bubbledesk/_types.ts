import type {
  TauriCore,
  TauriGlobal,
  OpenResult,
  DragDropPayload,
  FsEntry,
  FsPaths,
  FsScopeMethods,
  NotificationPermission,
  AppInfo
} from "@types";

export type BubbledeskAPI = {
  readonly isAvailable: boolean;
  readonly version: string;
  readonly ready: Promise<true>;
  invoke<T = unknown>(cmd: string, payload?: Record<string, unknown>): Promise<T>;

  notifications: {
    state: () => Promise<NotificationPermission>;
    request: () => Promise<Exclude<NotificationPermission, "default"> | string>;
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
    info: () => Promise<AppInfo>;
  };

  window: {
    minimize: () => Promise<void>;
    maximizeToggle: () => Promise<void>;
    fullscreen: (enable: boolean) => Promise<void>;
  };

  events: {
    emit: (event: string, payload?: unknown) => Promise<unknown>;
    emitTo: (window_label: string, event: string, payload?: unknown) => Promise<unknown>;
    on: (event: string, handler: (payload: any) => void) => Promise<() => void>;
    once: (event: string) => Promise<any>;
    onMany: (events: string[], handler: (name: string, payload: any) => void) => Promise<() => void>;
    onShortcut: (handler: (payload: any) => void) => Promise<() => void>;
    onDragDrop: (
      handler: (name: string, payload: DragDropPayload) => void,
      options?: { includeHover?: boolean }
    ) => Promise<() => void>;
  };

  globalShortcut: {
    register: (accelerator: string, cb: (e: any) => void, options?: { emitEvent?: boolean }) => Promise<void>;
    unregister: (accelerator: string) => Promise<void>;
    unregisterAll: () => Promise<void>;
    isRegistered: (accelerator: string) => Promise<boolean>;
  };

  fs: {
    cache: FsScopeMethods & { clear: () => Promise<void> };
    data: FsScopeMethods;
    paths: () => Promise<FsPaths>;
    base: { cache: string; data: string };
  };
};
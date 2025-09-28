import type {
  NotificationsInterface,
  ClipboardInterface,
  FilesInterface,
  AppInterface,
  WindowInterface,
  EventsInterface,
  ShortcutsInterface,
  FsInterface,
  MenuInterface,
  DiagnosticsInterface,
  BadgeInterface,
  WorkerInterface,
  ContextMenuInterface,
} from "@types";
import { AutostartInterface } from "modules/rs/autostart/_types";
import { NetworkInterface } from "modules/rs/network/_types";

export type BdPlatform = "macos" | "linux" | "windows";

/**
 * Bubbledesk API exposed in the webview.
 *
 * Each property corresponds to a bridge module backed by Tauri commands.
 * The API is globally mounted on `window.Bubbledesk` when the runtime is ready.
 *
 * Notes:
 * - `badge` is supported **only on macOS**. On Windows and Linux this field
 *   will be `undefined` and must be checked before use.
 * - All other modules are cross-platform.
 */
export type BubbledeskAPI = {
  readonly isAvailable: boolean;
  readonly version: string;
  readonly ready: Promise<true>;
  invoke<T = unknown>(
    cmd: string,
    payload?: Record<string, unknown>
  ): Promise<T>;
  notifications: NotificationsInterface;
  clipboard: ClipboardInterface;
  files: FilesInterface;
  app: AppInterface;
  window: WindowInterface;
  events: EventsInterface;
  globalShortcut: ShortcutsInterface;
  fs: FsInterface;
  menu: MenuInterface;
  diagnostics: DiagnosticsInterface;
  network: NetworkInterface;
  autostart: AutostartInterface;
  badge?: BadgeInterface;
  worker: WorkerInterface;
  contextMenu: ContextMenuInterface & {listening?: boolean, listener?: EventListenerOrEventListenerObject};
};

export interface BubbledeskInstanceInterface {
  ready: () => boolean;
  get: () => BubbledeskAPI;
}

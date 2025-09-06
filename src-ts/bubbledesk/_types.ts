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
} from "@types";
import { AutostartInterface } from "modules/autostart/_types";
import { NetworkInterface } from "modules/network/_types";

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
};

export interface BubbledeskInstanceInterface {
  ready: () => boolean;
  get: () => BubbledeskAPI;
}

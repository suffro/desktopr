import { DragDropPayload } from "@types";

export interface EventsInterface {
    emit: (event: string, payload?: unknown) => Promise<unknown>;
    emitTo: (window_label: string, event: string, payload?: unknown) => Promise<unknown>;
    on: (event: string, handler: (payload: any) => void) => Promise<() => void>;
    once: (event: string) => Promise<any>;
    onMany: (events: string[], handler: (name: string, payload: any) => void) => Promise<() => void>;
    onDeeplink: (handler: (payload: any) => void) => Promise<() => any>
    onShortcut: (handler: (payload: any) => void) => Promise<() => void>;
    onDragDrop: (
      handler: (name: string, payload: DragDropPayload) => void,
      options?: { includeHover?: boolean }
    ) => Promise<() => void>;
    onMenuEvent: (handler: (payload: any) => void) => Promise<() => any>;
    tray: {
      onIconEvent: (handler: (payload: any) => void) => Promise<() => any>;
      onMenuEvent: (handler: (payload: any) => void) => Promise<() => any>;
    };
}
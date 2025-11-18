import { DragDropPayload } from "@types";
export interface EventsInterface {
    emit: (event: string, payload?: unknown) => Promise<unknown>;
    emitToAll: (event: string, payload?: unknown) => Promise<unknown>;
    emitTo: (windowLabel: string, event: string, payload?: unknown) => Promise<unknown>;
    on: (event: string, handler: (payload: any) => void) => Promise<() => void>;
    once: (event: string) => Promise<any>;
    onMany: (events: string[], handler: (name: string, payload: any) => void) => Promise<() => void>;
    onNetworkStatus: (handler: (payload: any) => void) => Promise<() => any>;
    onDeeplink: (handler: (payload: any) => void) => Promise<() => any>;
    onShortcut: (handler: (payload: any) => void) => Promise<() => void>;
    onDragDrop: (handler: (name: string, payload: DragDropPayload) => void, options?: {
        includeHover?: boolean;
    }) => Promise<() => void>;
    onMenuEvent: (handler: (payload: any) => void) => Promise<() => any>;
    onTrayIconEvent: (handler: (payload: any) => void) => Promise<() => any>;
}

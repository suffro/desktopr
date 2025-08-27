import { BubbledeskAPI } from "@types";
import { listenForEvent } from "@helpers";
import type { DragDropPayload } from "@types";

export function buildEvents(core: { invoke: BubbledeskAPI["invoke"] }) {
  return {
    emit: (event: string, payload?: unknown) => core.invoke("bd_event_emit", { event, payload }),
    emitTo: (window_label: string, event: string, payload?: unknown) =>
      core.invoke("bd_event_emit_to", { window_label, event, payload }),

    on: async (event: string, handler: (payload: any) => void) => listenForEvent(event, handler),

    once: (event: string) =>
      new Promise<any>(async (resolve) => {
        const off = await listenForEvent(event, (p) => {
          off();
          resolve(p);
        });
      }),

    onMany: async (events: string[], handler: (name: string, payload: any) => void) => {
      const offs = await Promise.all(events.map((n) => listenForEvent(n, (p) => handler(n, p))));
      return () => offs.forEach((off) => off());
    },

    onDeeplink: async (handler: (payload: any) => void) => listenForEvent("deeplink", handler),

    onShortcut: async (handler: (payload: any) => void) => listenForEvent("shortcut:event", handler),

    onDragDrop: async (
      handler: (name: string, payload: DragDropPayload) => void,
      options?: { includeHover?: boolean }
    ) => {
      const evs = ["dragdrop:enter", "dragdrop:drop", "dragdrop:cancel"];
      if (options?.includeHover) evs.push("dragdrop:hover");
      const offs = await Promise.all(evs.map((n) => listenForEvent(n, (p) => handler(n, p))));
      return () => offs.forEach((off) => off());
    },
    
    onMenuClick: async (handler: (payload: any) => void) => listenForEvent("menu:click", handler),

  };
}

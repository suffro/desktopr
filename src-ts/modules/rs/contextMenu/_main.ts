import { initContextMenuListener, normalizeEntries, removeContextMenuListener } from "../../../_helpers";
import type { BubbledeskAPI, CmNode, CmPopupOptions, ContextMenuInterface } from "../../../_types";

export function buildContextMenu(core: { invoke: BubbledeskAPI["invoke"] }): ContextMenuInterface {
  return {
    show: (entries: CmNode[], options: CmPopupOptions): Promise<string> => core.invoke("bd_context_menu_popup", { items: normalizeEntries(entries), options }),
    handler: {
      init: initContextMenuListener,
      remove: removeContextMenuListener
    }
  };
}

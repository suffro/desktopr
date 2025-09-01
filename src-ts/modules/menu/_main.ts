import type { BubbledeskAPI, } from "@types";

export function buildMenu(core: { invoke: BubbledeskAPI["invoke"] }) {
  return {
    setEnabled: (id: string, enabled: boolean): Promise<void> => core.invoke("bd_menu_set_enabled", { id, enabled }),
    setChecked: (id: string, checked: boolean): Promise<void> => core.invoke("bd_menu_set_checked", { id, checked }),
  };
}

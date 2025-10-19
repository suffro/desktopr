import { applyMenuConfig } from "@helpers";
import type { BubbledeskAPI, MenuConfig, MenuInterface, } from "@types";

export function buildMenu(core: { invoke: BubbledeskAPI["invoke"] }): MenuInterface {
  return {
    setMenu: (config: MenuConfig): Promise<void> => applyMenuConfig(core, config),
    setEnabled: (id: string, enabled: boolean): Promise<void> => core.invoke("bd_menu_set_enabled", { id, enabled }),
    setChecked: (id: string, checked: boolean): Promise<void> => core.invoke("bd_menu_set_checked", { id, checked }),
    // listItems: (): Promise<any[]> => core.invoke("bd_menu_list_items"),
    // reset: (): Promise<void> => core.invoke("bd_menu_reset"),
    // disableAll: (): Promise<void> => core.invoke("bd_menu_disable_all"),
    // enableSection: (section: string): Promise<void> => core.invoke("bd_menu_enable_section", { section_name: section }),
    // disableSection: (section: string): Promise<void> => core.invoke("bd_menu_disable_section", { section_name: section }),
    // setLabel: (id: string, label: string): Promise<void> => core.invoke("bd_menu_set_label", { id, label }),
    // toggleChecked: (id: string): Promise<boolean> => core.invoke("bd_menu_toggle_checked", { id }),
    // getState: (id: string): Promise<any> => core.invoke("bd_menu_get_state", { id }),
    // reloadFromFile: (path: string): Promise<void> => core.invoke("bd_menu_reload_from_file", { path }),
  };
}

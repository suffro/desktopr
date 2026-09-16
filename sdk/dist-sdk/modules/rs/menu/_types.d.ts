import type { MenuConfig } from "../../../config/menu";
export type { MenuConfig, MenuCustomItem, MenuInteraction, MenuItem, MenuPlatform, MenuPredefinedItem, MenuSectionConfig, MenuSectionId, MenuSeparatorItem, MenuSubmenuItem, PredefinedItem, } from "../../../config/menu";
export type MenuCustomItemEventPayload = {
    id: string;
    [key: string]: unknown;
};
/** @deprecated Use MenuCustomItemEventPayload. */
export type MenuCustomItemEventPlayload = MenuCustomItemEventPayload;
export interface MenuInterface {
    setEnabled: (id: string, enabled: boolean) => Promise<void>;
    setChecked: (id: string, checked: boolean) => Promise<void>;
    init: {
        fromConfig: (config: MenuConfig, windowLabel?: string) => Promise<void>;
        fromJsonFile: (filePath: string) => Promise<void>;
    };
}

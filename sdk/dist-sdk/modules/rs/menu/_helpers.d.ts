import { DesktoprAPI, MenuConfig } from "../../../_types";
export declare const initMenuConfig: (core: {
    invoke: DesktoprAPI["invoke"];
}, menuConfig: MenuConfig, windowLabel?: string) => Promise<void>;

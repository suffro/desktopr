import { BubbledeskAPI, MenuConfig } from "../../../_types";
export declare const initMenuConfig: (core: {
    invoke: BubbledeskAPI["invoke"];
}, menuConfig: MenuConfig) => Promise<void>;

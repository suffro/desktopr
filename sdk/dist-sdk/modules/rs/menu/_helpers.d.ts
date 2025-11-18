import { BubbledeskAPI, MenuConfig } from "@types";
export declare const initMenuConfig: (core: {
    invoke: BubbledeskAPI["invoke"];
}, menuConfig: MenuConfig) => Promise<void>;

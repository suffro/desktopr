import type { BubbledeskAPI } from "@types";
import { AutostartInterface } from "./_types";
export declare function buildAutostart(core: {
    invoke: BubbledeskAPI["invoke"];
}): AutostartInterface;

import type { BubbledeskAPI } from "../../../_types";
import { NetworkInterface } from "./_types";
export declare function buildNetwork(core: {
    invoke: BubbledeskAPI["invoke"];
}): NetworkInterface;

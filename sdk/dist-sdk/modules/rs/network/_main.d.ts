import type { DesktoprAPI } from "../../../_types";
import { NetworkInterface } from "./_types";
export declare function buildNetwork(core: {
    invoke: DesktoprAPI["invoke"];
}): NetworkInterface;

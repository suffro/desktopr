import { BubbledeskAPI } from "@types";
import type { EventsInterface } from "@types";
export declare function buildEvents(core: {
    invoke: BubbledeskAPI["invoke"];
}): EventsInterface;

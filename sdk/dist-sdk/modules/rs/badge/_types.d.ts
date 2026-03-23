import { U32 } from "suffro-lib/utils";
export interface BadgeInterface {
    set: (count: U32) => Promise<void>;
    clear: () => Promise<void>;
}

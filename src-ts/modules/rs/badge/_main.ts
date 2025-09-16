import { AppInfo, BadgeInterface, BubbledeskAPI } from "@types";
import { U32 } from "suffro-lib";

export function buildBadge(core: { invoke: BubbledeskAPI["invoke"] }): BadgeInterface {
    return {
      set: async (count: U32): Promise<void> => core.invoke("bd_badge_set", {count}),
      clear: async (): Promise<void> => core.invoke("bd_badge_clear"),
    };
  }
  
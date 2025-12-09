import { launchCompanion } from "../../../_helpers";
import type { BubbledeskAPI, CompanionConfig, CompanionInterface } from "../../../_types";

export function buildCompanion(core: { invoke: BubbledeskAPI["invoke"] }): CompanionInterface {
  return {
    launch: (appConfig?: CompanionConfig): Promise<void> => launchCompanion(core, appConfig),
    state: {}
  };
}

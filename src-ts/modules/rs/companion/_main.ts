import { launchCompanion } from "./_helpers";
import type { BubbledeskAPI, CompanionInterface } from "../../../_types";

export function buildCompanion(core: { invoke: BubbledeskAPI["invoke"] }): CompanionInterface {
  return {
    launch: (appConfig?: object): Promise<void> => launchCompanion(core, appConfig),
  };
}

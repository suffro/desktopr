import { launchCompanion } from "../../../_helpers";
import type { DesktoprAPI, CompanionConfig, CompanionInterface } from "../../../_types";

export function buildCompanion(core: { invoke: DesktoprAPI["invoke"] }): CompanionInterface {
  return {
    launch: (appConfig?: CompanionConfig): Promise<void> => launchCompanion(core, appConfig),
    state: {}
  };
}

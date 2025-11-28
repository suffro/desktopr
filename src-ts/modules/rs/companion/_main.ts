import type { BubbledeskAPI, CompanionInterface } from "../../../_types";

export function buildCompanion(core: { invoke: BubbledeskAPI["invoke"] }): CompanionInterface {
  return {
    launch: (appConfig: object): Promise<void> => core.invoke("bd_launch_companion", { appConfig }),
  };
}

import type { BubbledeskAPI, CompanionInterface } from "@types";

export function buildCompanion(core: { invoke: BubbledeskAPI["invoke"] }): CompanionInterface {
  return {
    launch: (appConfig: object, menuConfig: object = {}): Promise<void> => core.invoke("bd_launch_companion", { appConfig, menuConfig }),
  };
}

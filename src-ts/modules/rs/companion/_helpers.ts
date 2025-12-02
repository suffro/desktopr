import { BubbledeskAPI } from "@types"

export const launchCompanion = async (core: { invoke: BubbledeskAPI["invoke"] }, config?: object) => {

    let appConfig: any = config || {};

    core.invoke("bd_launch_companion", { appConfig });
}
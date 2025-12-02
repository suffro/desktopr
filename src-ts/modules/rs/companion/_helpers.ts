import { BubbledeskAPI } from "@types"

export const launchCompanion = async (core: { invoke: BubbledeskAPI["invoke"] }, appConfig?: object) => {
    const icon = [
      "icons/icon-companion.png"
    ]
    let config: any = appConfig || {};

    config["$schema"]="https://schema.tauri.app/config/2";

    if(!(config?.bundle?.icon)) config["bundle"]["icon"] = icon;

    core.invoke("bd_launch_companion", { config });
}
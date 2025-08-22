import { AppInfo, BubbledeskAPI } from "@types";

export function buildAppInfo(core: { invoke: BubbledeskAPI["invoke"] }) {
    return {
      info: (): Promise<AppInfo> => core.invoke("bd_app_info"),
    };
  }
  
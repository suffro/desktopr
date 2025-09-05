import { AppInfo, BubbledeskAPI } from "@types";
import { I32 } from "suffro-lib";

export function buildAppInfo(core: { invoke: BubbledeskAPI["invoke"] }) {
    return {
      info: (): Promise<AppInfo> => core.invoke("bd_app_info"),
      exit: (code?: I32): Promise<AppInfo> => core.invoke("bd_app_exit", {code: code??0}),
    };
  }
  
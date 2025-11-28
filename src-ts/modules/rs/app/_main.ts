import { AppInfo, AppInterface, BubbledeskAPI } from "../../../_types";
import { I32 } from "suffro-lib";

export function buildAppInfo(core: { invoke: BubbledeskAPI["invoke"] }): AppInterface {
    return {
      info: (): Promise<AppInfo> => core.invoke("bd_app_info"),
      exit: (code?: I32): Promise<void> => core.invoke("bd_app_exit", {code: code??0})
    };
  }
  
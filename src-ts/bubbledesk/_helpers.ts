import { AppInfo, BdPlatform } from "@types";
import { isWindowAvailable, validate } from "suffro-lib";

export const normalizeString = (
  str: string,
  options: { toLowerCase: boolean; spacesFiller: string } = {
    toLowerCase: true,
    spacesFiller: "",
  }
): string => {
  if (!validate.nonEmptyString(str)) return "";
  let normalized = "";
  if (options.toLowerCase)
    normalized = String(str)
      .toLowerCase()
      .trim()
      .replace(/\s+/g, options.spacesFiller);
  else normalized = String(str).trim().replace(/\s+/g, options.spacesFiller);
  return normalized;
};

export const platformSpecifcFilter = async (platforms: BdPlatform[]): Promise<void> => {
  const appInfo: AppInfo = await window.Bubbledesk?.app.info() as AppInfo;
  if(!appInfo) throw "Failed to check platform";
  const plat = appInfo.os as BdPlatform;
  if(!(platforms.includes(plat))) throw `[unsupported platform] this method is not supported on ${plat}`;
}
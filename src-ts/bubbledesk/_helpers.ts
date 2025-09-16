import { validate } from "suffro-lib";

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

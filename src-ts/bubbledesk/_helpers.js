"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.platformSpecifcFilter = exports.normalizeString = void 0;
const suffro_lib_1 = require("suffro-lib");
const normalizeString = (str, options = {
    toLowerCase: true,
    spacesFiller: "",
}) => {
    if (!suffro_lib_1.validate.nonEmptyString(str))
        return "";
    let normalized = "";
    if (options.toLowerCase)
        normalized = String(str)
            .toLowerCase()
            .trim()
            .replace(/\s+/g, options.spacesFiller);
    else
        normalized = String(str).trim().replace(/\s+/g, options.spacesFiller);
    return normalized;
};
exports.normalizeString = normalizeString;
const platformSpecifcFilter = async (platforms) => {
    const appInfo = await window.Bubbledesk?.app.info();
    if (!appInfo)
        throw "Failed to check platform";
    const plat = appInfo.os;
    if (!(platforms.includes(plat)))
        throw `[unsupported platform] this method is not supported on ${plat}`;
};
exports.platformSpecifcFilter = platformSpecifcFilter;

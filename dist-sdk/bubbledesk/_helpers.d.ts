import { BdPlatform } from "@types";
export declare const normalizeString: (str: string, options?: {
    toLowerCase: boolean;
    spacesFiller: string;
}) => string;
export declare const platformSpecifcFilter: (platforms: BdPlatform[]) => Promise<void>;

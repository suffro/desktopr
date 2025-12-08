import { BubbledeskAPI, CompanionConfig } from "../../../_types";
/**
 * Validate whether a string is:
 * - a valid absolute URL (http/https)
 * - a valid internal path ("/something")
 *
 * Returns:
 *  - { kind: "url", value }
 *  - { kind: "path", value }
 *  - { kind: "invalid" }
 */
export declare function validateUrlOrPath(input?: string): {
    kind: "url" | "path" | "invalid";
    value?: string;
};
export declare const launchCompanion: (core: {
    invoke: BubbledeskAPI["invoke"];
}, config?: CompanionConfig) => Promise<void>;

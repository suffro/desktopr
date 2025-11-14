import { BubbledeskAPI } from "@types";
import { DiagnosticsTestFunctions, PrivacySettings } from "./_types";
export declare const diagnosticsSettings: (core: {
    invoke: BubbledeskAPI["invoke"];
}, settings?: PrivacySettings) => Promise<unknown>;
export declare function buildDiagnosticsTestFunctions(core: {
    invoke: BubbledeskAPI["invoke"];
}): DiagnosticsTestFunctions;

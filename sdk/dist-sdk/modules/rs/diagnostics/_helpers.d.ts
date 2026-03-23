import { DesktoprAPI } from "../../../_types";
import { DiagnosticsTestFunctions, PrivacySettings } from "./_types";
export declare const diagnosticsSettings: (core: {
    invoke: DesktoprAPI["invoke"];
}, settings?: PrivacySettings) => Promise<unknown>;
export declare function buildDiagnosticsTestFunctions(core: {
    invoke: DesktoprAPI["invoke"];
}): DiagnosticsTestFunctions;

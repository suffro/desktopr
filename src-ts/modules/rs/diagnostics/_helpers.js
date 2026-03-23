"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.diagnosticsSettings = void 0;
exports.buildDiagnosticsTestFunctions = buildDiagnosticsTestFunctions;
const suffro_lib_1 = require("suffro-lib");
const diagnosticsSettings = async (core, settings) => {
    if (settings?.retentionDaysAnalytics && !suffro_lib_1.Num.isU32(settings.retentionDaysAnalytics))
        throw ("[retentionDaysAnalytics] the value must be a U32 integer number");
    if (settings?.retentionDaysLogs && !suffro_lib_1.Num.isU32(settings.retentionDaysLogs))
        throw ("[retentionDaysLogs] the value must be a U32 integer number");
    if (settings?.retentionDaysCrashes && !suffro_lib_1.Num.isU32(settings.retentionDaysCrashes))
        throw ("[retentionDaysCrashes] the value must be a U32 integer number");
    return await core.invoke("dtr_logs_set_privacy", {
        patch: {
            analytics_enabled: settings?.analyticsEnabled,
            crash_reports_enabled: settings?.crashReportsEnabled,
            retention_days_analytics: settings?.retentionDaysAnalytics,
            retention_days_logs: settings?.retentionDaysLogs,
            retention_days_crashes: settings?.retentionDaysCrashes,
        }
    });
};
exports.diagnosticsSettings = diagnosticsSettings;
// ==============================
// Test functions
// ==============================
function buildDiagnosticsTestFunctions(core) {
    return {
        testGenerateRecords: (n = 200) => core.invoke("dtr_logs_test_record_n", { n }),
        // Questa può semplicemente lanciare un errore JS: non serve invoke.
        testThrowJsError: async () => {
            throw new Error("DEV: test JS error");
        },
        testPanicRust: () => core.invoke("dtr_logs_test_panic", {}),
        testExportZip: (path) => core.invoke("dtr_logs_export_zip", { target_zip_path: path }),
        testForceRetention: (area) => core.invoke("dtr_logs_test_force_retention", { area }),
    };
}

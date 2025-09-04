// src-ts/_dev_diagnostics.ts
import { BubbledeskAPI } from "@types";
import { DiagnosticsTestFunctions, PrivacySettings } from "./_types";
import { Num } from "suffro-lib";

export const diagnosticsSettings = async(core: { invoke: BubbledeskAPI["invoke"] }, settings?: PrivacySettings)=> {
  if(settings?.retention_days_analytics && !Num.isU32(settings.retention_days_analytics)) throw("[retention_days_analytics] the value must be a U32 integer number");
  if(settings?.retention_days_logs && !Num.isU32(settings.retention_days_logs)) throw("[retention_days_logs] the value must be a U32 integer number");
  if(settings?.retention_days_crashes && !Num.isU32(settings.retention_days_crashes)) throw("[retention_days_crashes] the value must be a U32 integer number");

  await core.invoke("bd_logs_set_privacy", {
    analytics_enabled: settings?.analytics_enabled,
    crash_reports_enabled: settings?.crash_reports_enabled,
    retention_days_analytics: settings?.retention_days_analytics,
    retention_days_logs: settings?.retention_days_logs,
    retention_days_crashes: settings?.retention_days_crashes,
  });
}

// ==============================
// Test functions
// ==============================

export function buildDiagnosticsTestFunctions(core: { invoke: BubbledeskAPI["invoke"] }): DiagnosticsTestFunctions {
  return {
    testGenerateRecords: (n = 200) =>
      core.invoke<void>("bd_logs_test_record_n", { n }),

    // Questa può semplicemente lanciare un errore JS: non serve invoke.
    testThrowJsError: async () => {
      throw new Error("DEV: test JS error");
    },

    testPanicRust: () => core.invoke<void>("bd_logs_test_panic", {}),

    testExportZip: (path: string) =>
      core.invoke<void>("bd_logs_export_zip", { target_zip_path: path }),

    testForceRetention: (area: "logs" | "analytics" | "crashes") =>
      core.invoke<void>("bd_logs_test_force_retention", { area }),
  };
}

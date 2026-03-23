// src-ts/_dev_diagnostics.ts
import { DesktoprAPI } from "../../../_types";
import { DiagnosticsTestFunctions, PrivacySettings } from "./_types";
import { Num } from "suffro-lib/utils";

export const diagnosticsSettings = async (core: { invoke: DesktoprAPI["invoke"] }, settings?: PrivacySettings)=> {
  if(settings?.retentionDaysAnalytics && !Num.isU32(settings.retentionDaysAnalytics)) throw("[retentionDaysAnalytics] the value must be a U32 integer number");
  if(settings?.retentionDaysLogs && !Num.isU32(settings.retentionDaysLogs)) throw("[retentionDaysLogs] the value must be a U32 integer number");
  if(settings?.retentionDaysCrashes && !Num.isU32(settings.retentionDaysCrashes)) throw("[retentionDaysCrashes] the value must be a U32 integer number");

  return await core.invoke("dtr_logs_set_privacy", {
    patch: {
      analytics_enabled: settings?.analyticsEnabled,
      crash_reports_enabled: settings?.crashReportsEnabled,
      retention_days_analytics: settings?.retentionDaysAnalytics,
      retention_days_logs: settings?.retentionDaysLogs,
      retention_days_crashes: settings?.retentionDaysCrashes,
    }
  });
}

// ==============================
// Test functions
// ==============================

export function buildDiagnosticsTestFunctions(core: { invoke: DesktoprAPI["invoke"] }): DiagnosticsTestFunctions {
  return {
    testGenerateRecords: (n = 200) =>
      core.invoke<void>("dtr_logs_test_record_n", { n }),

    // Questa può semplicemente lanciare un errore JS: non serve invoke.
    testThrowJsError: async () => {
      throw new Error("DEV: test JS error");
    },

    testPanicRust: () => core.invoke<void>("dtr_logs_test_panic", {}),

    testExportZip: (path: string) =>
      core.invoke<void>("dtr_logs_export_zip", { target_zip_path: path }),

    testForceRetention: (area: "logs" | "analytics" | "crashes") =>
      core.invoke<void>("dtr_logs_test_force_retention", { area }),
  };
}

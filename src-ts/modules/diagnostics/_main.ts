import { BubbledeskAPI } from "@types";
import { buildDiagnosticsTestFunctions } from "./_helpers";
import { AnalyticsPayload, DiagnosticsInterface, ErrorPayload, ListedFile, PrivacySettings, RecordPayload } from "./_types";

export function buildDiagnostics(core: { invoke: BubbledeskAPI["invoke"] }): DiagnosticsInterface {
  return {
    settings: {
      // set: mappa ai parametri snake_case attesi da Rust (tutti opzionali)
      set: (settings?: PrivacySettings) =>
        core.invoke("bd_logs_set_privacy", {
          analytics_enabled: settings?.analytics_enabled,
          crash_reports_enabled: settings?.crash_reports_enabled,
          retention_days_logs: settings?.retention_days_logs,
          retention_days_analytics: settings?.retention_days_analytics,
          retention_days_crashes: settings?.retention_days_crashes,
        }),
      get: () => core.invoke("bd_logs_get_privacy", {}),
    },

    // SOLO analytics (Rust vuole AnalyticsRecord), usa key record_type
    newRecord: (
      record_type: string,
      payload: AnalyticsPayload,
      env: "js" | "native" | string,
      app_version: string
    ) =>
      core.invoke("bd_logs_new_record", {
        record_type,
        payload,
        env,
        app_version,
      }),

    newError: {
      js: (payload: ErrorPayload, app_version: string) =>
        core.invoke("bd_logs_record_js_error", { payload, app_version }),

      native: (payload: ErrorPayload, app_version: string) =>
        core.invoke("bd_logs_record_native_error", { payload, app_version }),

      // Rust richiede 'env' obbligatorio: di default "generic" se non passato
      generic: (payload: ErrorPayload, app_version: string, env = "generic") =>
        core.invoke("bd_logs_record_error", { payload, app_version, env }),
    },

    readRecordsFile: (rel_path: string, max_bytes?: number) =>
      core.invoke<Uint8Array>("bd_logs_read_file", { rel_path, max_bytes }),

    // qui avevi chiamato bd_logs_read_file: correggo su bd_logs_list_files
    listRecordsFiles: (area: "logs" | "crashes") =>
      core.invoke("bd_logs_list_files", { area }),

    runRetention: () => core.invoke("bd_logs_run_retention", {}),

    export: (target_zip_path: string) =>
      core.invoke("bd_logs_export_zip", { target_zip_path }),

    test: () => buildDiagnosticsTestFunctions(core),
  };
}

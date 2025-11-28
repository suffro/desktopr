import { BubbledeskAPI } from "../../../_types";
import { buildDiagnosticsTestFunctions, diagnosticsSettings } from "./_helpers";
import { AnalyticsPayload, DiagnosticsInterface, ErrorPayload, PrivacySettings } from "./_types";
import { APP_VERSION } from "../../../_constants";

export function buildDiagnostics(core: { invoke: BubbledeskAPI["invoke"] }): DiagnosticsInterface {
  return {
    settings: {
      // set: mappa ai parametri snake_case attesi da Rust (tutti opzionali)
      set: (settings?: PrivacySettings) => diagnosticsSettings(core, settings),
      get: () => core.invoke("bd_logs_get_privacy", {}),
    },

    // SOLO analytics (Rust vuole AnalyticsRecord), usa key record_type
    newRecord: (
      recordType: string,
      payload: AnalyticsPayload,
      env: "js" | "native" | string,
    ) =>
      core.invoke("bd_logs_new_record", {
        recordType,
        payload,
        env,
        appVersion: APP_VERSION,
      }),

    newError: {
      js: (payload: ErrorPayload) =>
        core.invoke("bd_logs_record_js_error", { payload, appVersion: APP_VERSION }),

      native: (payload: ErrorPayload) =>
        core.invoke("bd_logs_record_native_error", { payload, appVersion: APP_VERSION }),

      // Rust richiede 'env' obbligatorio: di default "generic" se non passato
      generic: (payload: ErrorPayload, env = "generic") =>
        core.invoke("bd_logs_record_error", { payload, appVersion: APP_VERSION, env }),
    },

    readRecordsFile: (relPath: string, maxBytes?: number) =>
      core.invoke<Uint8Array>("bd_logs_read_file", { relPath, maxBytes }),

    // qui avevi chiamato bd_logs_read_file: correggo su bd_logs_list_files
    listRecordsFiles: (area: "logs" | "crashes") =>
      core.invoke("bd_logs_list_files", { area }),

    runRetention: () => core.invoke("bd_logs_run_retention", {}),

    export: (targetZipPath: string) =>
      core.invoke("bd_logs_export_zip", { targetZipPath }),

    test: buildDiagnosticsTestFunctions(core),
  };
}

import { DesktoprInstance } from "../../../_main";
import { DesktoprAPI } from "../../../_types";
import { buildDiagnosticsTestFunctions, diagnosticsSettings } from "./_helpers";
import { AnalyticsPayload, DiagnosticsInterface, ErrorPayload, PrivacySettings } from "./_types";

export function buildDiagnostics(core: { invoke: DesktoprAPI["invoke"] }): DiagnosticsInterface {
  return {
    settings: {
      // set: mappa ai parametri snake_case attesi da Rust (tutti opzionali)
      set: (settings?: PrivacySettings) => diagnosticsSettings(core, settings),
      get: () => core.invoke("dtr_logs_get_privacy", {}),
    },

    // SOLO analytics (Rust vuole AnalyticsRecord), usa key record_type
    newRecord: (
      recordType: string,
      payload: AnalyticsPayload,
      env: "js" | "native" | string,
    ) =>
      core.invoke("dtr_logs_new_record", {
        recordType,
        payload,
        env,
        get appVersion() { return (window as any).__DESKTOPR_APP_VERSION__ },
      }),

    newError: {
      js: (payload: ErrorPayload) =>
        core.invoke("dtr_logs_record_js_error", { payload, get appVersion() { return (window as any).__DESKTOPR_APP_VERSION__ } }),

      native: (payload: ErrorPayload) =>
        core.invoke("dtr_logs_record_native_error", { payload, get appVersion() { return (window as any).__DESKTOPR_APP_VERSION__ } }),

      // Rust richiede 'env' obbligatorio: di default "generic" se non passato
      generic: (payload: ErrorPayload, env = "generic") =>
        core.invoke("dtr_logs_record_error", { payload, get appVersion() { return (window as any).__DESKTOPR_APP_VERSION__ }, env }),
    },

    readRecordsFile: (relPath: string, maxBytes?: number) =>
      core.invoke<Uint8Array>("dtr_logs_read_file", { relPath, maxBytes }),

    // qui avevi chiamato dtr_logs_read_file: correggo su dtr_logs_list_files
    listRecordsFiles: (area: "logs" | "crashes") =>
      core.invoke("dtr_logs_list_files", { area }),

    runRetention: () => core.invoke("dtr_logs_run_retention", {}),

    export: (targetZipPath: string) =>
      core.invoke("dtr_logs_export_zip", { targetZipPath }),

    test: buildDiagnosticsTestFunctions(core),
  };
}

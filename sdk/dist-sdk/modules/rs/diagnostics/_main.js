"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildDiagnostics = buildDiagnostics;
const _helpers_1 = require("./_helpers");
function buildDiagnostics(core) {
    return {
        settings: {
            // set: mappa ai parametri snake_case attesi da Rust (tutti opzionali)
            set: (settings) => (0, _helpers_1.diagnosticsSettings)(core, settings),
            get: () => core.invoke("dtr_logs_get_privacy", {}),
        },
        // SOLO analytics (Rust vuole AnalyticsRecord), usa key record_type
        newRecord: (recordType, payload, env) => core.invoke("dtr_logs_new_record", {
            recordType,
            payload,
            env,
            get appVersion() { return window.__DESKTOPR_APP_VERSION__; },
        }),
        newError: {
            js: (payload) => core.invoke("dtr_logs_record_js_error", { payload, get appVersion() { return window.__DESKTOPR_APP_VERSION__; } }),
            native: (payload) => core.invoke("dtr_logs_record_native_error", { payload, get appVersion() { return window.__DESKTOPR_APP_VERSION__; } }),
            // Rust richiede 'env' obbligatorio: di default "generic" se non passato
            generic: (payload, env = "generic") => core.invoke("dtr_logs_record_error", { payload, get appVersion() { return window.__DESKTOPR_APP_VERSION__; }, env }),
        },
        readRecordsFile: (relPath, maxBytes) => core.invoke("dtr_logs_read_file", { relPath, maxBytes }),
        // qui avevi chiamato dtr_logs_read_file: correggo su dtr_logs_list_files
        listRecordsFiles: (area) => core.invoke("dtr_logs_list_files", { area }),
        runRetention: () => core.invoke("dtr_logs_run_retention", {}),
        export: (targetZipPath) => core.invoke("dtr_logs_export_zip", { targetZipPath }),
        test: (0, _helpers_1.buildDiagnosticsTestFunctions)(core),
    };
}

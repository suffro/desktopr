import { U32, U64 } from "suffro-lib";

export interface DiagnosticsInterface {
  settings: {
      set: (settings?: PrivacySettings) => Promise<unknown>;
      get: () => Promise<unknown>;
  };
  newRecord: (record_type: string, payload: AnalyticsPayload, env: "js" | "native" | string, app_version: string) => Promise<unknown>;
  newError: {
      js: (payload: ErrorPayload, app_version: string) => Promise<unknown>;
      native: (payload: ErrorPayload, app_version: string) => Promise<unknown>
      generic: (payload: ErrorPayload, app_version: string, env?: string) => Promise<unknown>
  };
  readRecordsFile: (rel_path: string, max_bytes?: number) => Promise<Uint8Array<ArrayBufferLike>>;
  listRecordsFiles: (area: "logs" | "crashes") => Promise<unknown>;
  runRetention: () => Promise<unknown>;
  export: (target_zip_path: string) => Promise<unknown>;
  test: () => DiagnosticsTestFunctions;
}

export type DiagnosticsTestFunctions = {
  testGenerateRecords: (n?: number) => Promise<void>;
  testThrowJsError: () => Promise<never>;
  testPanicRust: () => Promise<void>;
  testExportZip: (path: string) => Promise<void>;
  testForceRetention: (area: "logs" | "analytics" | "crashes") => Promise<void>;
}


export type ErrorPayload = {
  message: string;
  filename?: string;
  lineno?: number; // u32 in Rust
  colno?: number;  // u32 in Rust
  stack?: string;
};

export type AnalyticsPayload = {
  name: string;
  props?: Record<string, unknown>; // serde_json::Value
};

export type ListedFile = {
  rel_path: string;   // String
  bytes: number;      // u64 -> JS number (ok per file < 9PB; i nostri log sono piccoli)
  modified_ms: number; // u64 epoch ms
};

export type RecordPayload = ErrorPayload | AnalyticsPayload;

export type PrivacySettings = {
  analytics_enabled?: boolean;
  crash_reports_enabled?: boolean;
  retention_days_analytics?: U32;
  retention_days_logs?: U32;
  retention_days_crashes?: U32;
}
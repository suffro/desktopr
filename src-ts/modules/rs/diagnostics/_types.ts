import { U32, U64 } from "suffro-lib/utils";

export interface DiagnosticsInterface {
  settings: {
      set: (settings?: PrivacySettings) => Promise<unknown>;
      get: () => Promise<unknown>;
  };
  newRecord: (recordType: string, payload: AnalyticsPayload, env: "js" | "native" | string) => Promise<unknown>;
  newError: {
      js: (payload: ErrorPayload) => Promise<unknown>;
      native: (payload: ErrorPayload) => Promise<unknown>
      generic: (payload: ErrorPayload, env?: string) => Promise<unknown>
  };
  readRecordsFile: (relPath: string, maxBytes?: number) => Promise<Uint8Array<ArrayBufferLike>>;
  listRecordsFiles: (area: "logs" | "crashes") => Promise<unknown>;
  runRetention: () => Promise<unknown>;
  export: (targetZipPath: string) => Promise<unknown>;
  test: DiagnosticsTestFunctions;
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
  relPath: string;   // String
  bytes: number;      // u64 -> JS number (ok per file < 9PB; i nostri log sono piccoli)
  modified_ms: number; // u64 epoch ms
};

export type RecordPayload = ErrorPayload | AnalyticsPayload;

export type PrivacySettings = {
  analyticsEnabled?: boolean;
  crashReportsEnabled?: boolean;
  retentionDaysAnalytics?: U32;
  retentionDaysLogs?: U32;
  retentionDaysCrashes?: U32;
}
import { U32 } from "suffro-lib";
export interface DiagnosticsInterface {
    settings: {
        set: (settings?: PrivacySettings) => Promise<unknown>;
        get: () => Promise<unknown>;
    };
    newRecord: (recordType: string, payload: AnalyticsPayload, env: "js" | "native" | string) => Promise<unknown>;
    newError: {
        js: (payload: ErrorPayload) => Promise<unknown>;
        native: (payload: ErrorPayload) => Promise<unknown>;
        generic: (payload: ErrorPayload, env?: string) => Promise<unknown>;
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
};
export type ErrorPayload = {
    message: string;
    filename?: string;
    lineno?: number;
    colno?: number;
    stack?: string;
};
export type AnalyticsPayload = {
    name: string;
    props?: Record<string, unknown>;
};
export type ListedFile = {
    relPath: string;
    bytes: number;
    modified_ms: number;
};
export type RecordPayload = ErrorPayload | AnalyticsPayload;
export type PrivacySettings = {
    analyticsEnabled?: boolean;
    crashReportsEnabled?: boolean;
    retentionDaysAnalytics?: U32;
    retentionDaysLogs?: U32;
    retentionDaysCrashes?: U32;
};

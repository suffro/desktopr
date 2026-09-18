---
title: Diagnostics Module
description: Manage diagnostics settings, records, errors, retention, and export operations.

prev:
    text: File System
    link: guide/bridge/api/file-system

next:
    text: Shortcuts
    link: guide/bridge/api/shortcuts
---

# Diagnostics Module

The `Diagnostics` module provides utilities for managing analytics records, normal error logs, crash-like events, retention policies, and diagnostic export files.

It is designed to help your app collect diagnostic information in a structured way, while also giving you control over privacy settings and retention behavior.

Diagnostics data is stored inside the app diagnostics area and is organized into three main areas:

| Area | Description |
|------|-------------|
| `logs` | Monthly JSONL files containing analytics records, normal error records, and dirty shutdown entries. |
| `crashes` | Crash-like events only, such as native panics and dirty shutdown crash reports. |
| `runtime` | Runtime marker files used internally by the heartbeat and clean shutdown system. |

## Methods

| Method | Description | Return Type |
|-------|-------------|-------------|
| `settings.set(settings?: PrivacySettings)` | Updates the diagnostics privacy settings. | `Promise<PrivacySettings>` |
| `settings.get()` | Returns the current diagnostics privacy settings. | `Promise<PrivacySettings>` |
| `newRecord(recordType: string, payload: AnalyticsPayload, env: string, appVersion?: string)` | Creates a new analytics record for the specified environment. | `Promise<void>` |
| `newError.js(payload: ErrorPayload, appVersion?: string)` | Records a new JavaScript error entry in `logs`. | `Promise<void>` |
| `newError.native(payload: ErrorPayload, appVersion?: string)` | Records a new native error entry in `logs`. | `Promise<void>` |
| `newError.generic(payload: ErrorPayload, env?: string, appVersion?: string)` | Records a new generic error entry in `logs` with an optional environment label. | `Promise<void>` |
| `readRecordsFile(relPath: string)` | Reads a diagnostics file and returns its raw text contents. | `Promise<string>` |
| `listRecordsFiles(area: "logs" \| "crashes" \| "runtime")` | Lists available diagnostics files in the given area. | `Promise<ListedFile[]>` |
| `runRetention()` | Runs retention cleanup for stored diagnostics data. | `Promise<void>` |
| `export(targetZipPath: string)` | Exports diagnostics data to a ZIP archive at the given target path. | `Promise<void>` |

## Test Methods

The `test` API is intended for development builds and testing scenarios only.

| Method | Description | Return Type |
|-------|-------------|-------------|
| `test.testGenerateRecords(n?: number)` | Generates a batch of test analytics records. | `Promise<void>` |
| `test.testThrowJsError()` | Triggers a JavaScript test error. | `Promise<never>` |
| `test.testPanicRust()` | Triggers a native Rust panic for testing the crash pipeline. | `Promise<void>` |
| `test.testExportZip(path: string)` | Generates a test diagnostics ZIP export. | `Promise<void>` |
| `test.testForceRetention(area: "logs" \| "crashes" \| "runtime")` | Forces cleanup for all files in the specified diagnostics area. | `Promise<void>` |

## Behavior and Settings

The diagnostics module behavior depends on the current `PrivacySettings`.

| Setting | Default | Behavior |
|--------|---------|----------|
| `analyticsEnabled` | `false` | Controls whether `newRecord()` writes analytics records. If disabled, `newRecord()` resolves successfully but does not write a record. |
| `crashReportsEnabled` | `true` | Controls whether crash-like events are written to `crashes`. This applies to Rust panics and dirty shutdown crash reports. |
| `retentionDaysLogs` | `180` | Controls how long files in `logs` are kept when retention runs. Values are clamped to at least `30`. |
| `retentionDaysAnalytics` | `180` | Reserved for analytics retention settings. Analytics records currently share the `logs` area. |
| `retentionDaysCrashes` | `365` | Controls how long files in `crashes` are kept when retention runs. Values are clamped to at least `90`. |

Normal errors recorded with `newError.js()`, `newError.native()`, or `newError.generic()` are written to `logs` only. They are not written to `crashes`.

Crash-like events are handled separately:

| Event | Written to `logs` | Written to `crashes` |
|------|-------------------|----------------------|
| `newRecord()` | Yes, only when `analyticsEnabled` is `true` | No |
| `newError.js()` | Yes | No |
| `newError.native()` | Yes | No |
| `newError.generic()` | Yes | No |
| Dirty shutdown | Yes | Yes, only when `crashReportsEnabled` is `true` |
| Rust panic | No | Yes, only when `crashReportsEnabled` is `true` |

Retention currently runs for `logs` and `crashes`. Runtime files are managed by the heartbeat system and are not affected by `runRetention()`.

## DiagnosticsInterface

```ts
interface DiagnosticsInterface {
  settings: {
    set: (settings?: PrivacySettings) => Promise<PrivacySettings>;
    get: () => Promise<PrivacySettings>;
  };

  newRecord: (
    recordType: string,
    payload: AnalyticsPayload,
    env: "js" | "native" | string,
    appVersion?: string
  ) => Promise<void>;

  newError: {
    js: (payload: ErrorPayload, appVersion?: string) => Promise<void>;
    native: (payload: ErrorPayload, appVersion?: string) => Promise<void>;
    generic: (
      payload: ErrorPayload,
      env?: string,
      appVersion?: string
    ) => Promise<void>;
  };

  readRecordsFile: (relPath: string) => Promise<string>;

  listRecordsFiles: (area: DiagnosticsArea) => Promise<ListedFile[]>;

  runRetention: () => Promise<void>;

  export: (targetZipPath: string) => Promise<void>;

  test: DiagnosticsTestFunctions;
}
```

## Types

### DiagnosticsArea

```ts
type DiagnosticsArea = "logs" | "crashes" | "runtime";
```

### DiagnosticsTestFunctions

```ts
type DiagnosticsTestFunctions = {
  testGenerateRecords: (n?: number) => Promise<void>;
  testThrowJsError: () => Promise<never>;
  testPanicRust: () => Promise<void>;
  testExportZip: (path: string) => Promise<void>;
  testForceRetention: (area: DiagnosticsArea) => Promise<void>;
};
```

### ErrorPayload

```ts
type ErrorPayload = {
  message: string;
  filename?: string;
  lineno?: number;
  colno?: number;
  stack?: string;
};
```

### AnalyticsPayload

```ts
type AnalyticsPayload = {
  name: string;
  props?: Record<string, unknown>;
};
```

### ListedFile

`ListedFile` uses the same field names returned by the native bridge.

```ts
type ListedFile = {
  rel_path: string;
  bytes: number;
  modified_ms: number;
};
```

### RecordPayload

```ts
type RecordPayload = ErrorPayload | AnalyticsPayload;
```

### PrivacySettings

```ts
type PrivacySettings = {
  analyticsEnabled?: boolean;
  crashReportsEnabled?: boolean;
  retentionDaysAnalytics?: number;
  retentionDaysLogs?: number;
  retentionDaysCrashes?: number;
};
```

## Example Usage

### Update privacy settings

```js
await Desktopr.diagnostics.settings.set({
  analyticsEnabled: true,
  crashReportsEnabled: true,
  retentionDaysAnalytics: 30,
  retentionDaysLogs: 30,
  retentionDaysCrashes: 90
});

const settings = await Desktopr.diagnostics.settings.get();
console.log("Diagnostics settings:", settings);
```

### Create an analytics record

```js
await Desktopr.diagnostics.newRecord(
  "user-action",
  {
    name: "button_clicked",
    props: {
      section: "settings",
      button: "save"
    }
  },
  "js"
);
```

### Record an error

```js
await Desktopr.diagnostics.newError.js({
  message: "Something went wrong",
  filename: "app.js",
  lineno: 42,
  colno: 13,
  stack: "Error: Something went wrong..."
});
```

### List and read log files

```js
const files = await Desktopr.diagnostics.listRecordsFiles("logs");

if (files.length > 0) {
  const content = await Desktopr.diagnostics.readRecordsFile(files[0].rel_path);
  console.log(content);
}
```

### Export diagnostics data

```js
await Desktopr.diagnostics.export("/tmp/diagnostics-export.zip");
```

## Notes

- `newRecord()` is intended for structured analytics or diagnostic events and depends on `analyticsEnabled`.
- `newError` helpers are for normal error logging and write to `logs`, not `crashes`.
- `crashes` is reserved for crash-like events, such as Rust panics and dirty shutdown reports.
- `readRecordsFile()` returns text content, not bytes.
- `listRecordsFiles()` returns native field names such as `rel_path` and `modified_ms`.
- The `runtime` area is mainly useful for debugging the heartbeat and shutdown marker system.
- The `test` API is intended for development and testing scenarios only.

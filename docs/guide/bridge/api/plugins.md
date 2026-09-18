---
title: Plugin Module
description: Execute WebAssembly (WASM) plugins securely inside Desktopr's native Tauri/WASI runtime.
prev:
    text: App
    link: guide/bridge/api/app
next:
    text: Badge
    link: guide/bridge/api/badge
---

# Plugins Module

This module allows you to manage and execute **Desktopr plugins** loaded in your app.

Desktopr plugins are [WebAssembly (WASM)](https://webassembly.org/) modules executed by Desktopr inside a native **Tauri + Wasmtime + WASI** runtime. They are useful for adding local processing, data utilities, file conversion, validation, computations, and other advanced features without rebuilding the whole desktop app.

:::info How to create a new plugin
You can find the guide to develop a plugin [here](/guide/plugin-development.md).
:::

## Overview

Desktopr plugins do **not** run inside the browser page and do **not** run inside a browser Web Worker.

They are executed by the Rust side of the Desktopr wrapper through Wasmtime and WASI.

The runtime provides:

- **Native WASI execution**
- **WASM module validation**
- **JSON-based stdin/stdout communication**
- **Captured stderr diagnostics**
- **Persistent per-plugin storage**
- **Sandboxed environment**
- **Automatic timeout handling**
- **Cross-platform consistency on macOS, Windows, and Linux**

Plugins are managed via the JavaScript bridge at `Desktopr.plugins`.

## Architecture

The system is composed of three layers:

| Layer | Language | Responsibility |
|--------|-----------|----------------|
| **Frontend** | JavaScript or TypeScript | Calls plugins through the Desktopr Bridge API |
| **Desktopr Wrapper** | Rust + Tauri | Manages plugin installation, WASM validation, execution, timeout, stdin/stdout/stderr, and storage |
| **Execution Runtime** | Wasmtime + WASI | Executes the loaded WebAssembly module in a sandboxed native runtime |

The plugin itself is a `.wasm` file compiled for the WASI target.

## API Reference

| Method | Parameters | Returns | Description |
|---------|-------------|----------|--------------|
| **call(modulePath, payload, timeoutMs?)** | `modulePath: string` – The module filename, for example `"math.wasm"`.<br>`payload:` [`PluginCallPayload`](#plugincallpayload) – Object indicating the function to run and its arguments.<br>`timeoutMs?: number` – Optional execution timeout in milliseconds. | `Promise<PluginCallResult>` | Executes a function inside a loaded WASM plugin. |
| **status()** | – | `Promise<PluginStatusResult>` | Returns the current native plugin runtime status. |
| **list()** | – | `Promise<string[]>` | Lists all installed WASM modules available to the plugin runtime. |
| **remove(name)** | `name: string` – Module filename to delete. | `Promise<boolean>` | Removes a module from the installed modules directory. |
| **addFromBytes(name, contents)** | `name: string` – Name for the new module.<br>`contents: Uint8Array` – Raw WASM bytes to store. | `Promise<any>` | Adds a module directly from memory bytes. |
| **add(name, maxBytes?)** | `name: string` – Filename to assign.<br>`maxBytes?: number` – Optional file size limit. | `Promise<PluginAddResult>` | Opens a native file picker to select and store a `.wasm` file. |
| **killJobs()** | - | `Promise<boolean>` | Kills all running plugin runtimes. |

Each method is asynchronous and returns a Promise.

## Types

### PluginCallPayload

| Field | Type | Description |
|--------|------|-------------|
| **fn** | `string` | The function name to execute inside the WASM module. |
| **args** | `array or object` | Arguments passed to the target function. Both positional and named arguments are supported. |
| **[key: string]** | `any` | Optional additional fields included in the JSON payload sent to the plugin. |

```
{
  fn: string;
  args?: unknown[] | Record<string, unknown>;
  [key: string]: unknown;
};
```

### PluginCallResult

A successful Desktopr plugin call response may look like this:

```
{
  durationMs: number;
  error: string|null|undefined;
  id: string;
  ok: boolean;
  stderr: string;
  stdout: string;
  value: {
    error?: string,
    value?: string|number,
    ok: boolean
  };
};
```

Notes:

- `stdout` contains the raw text printed by the plugin to stdout.
- `stderr` contains debug or diagnostic output printed by the plugin to stderr.
- `value` is Desktopr's parsed JSON value from `stdout`, when parsing succeeds.
- `error` contains a runtime-level error if the call failed before or during execution.

### PluginAddResult

```
{
  bytes: number;
  name: string;
  path: string;
  saved: boolean;
};
```

### PluginStatusResult

```
{
    ready: true,
    runtime: string
};
```


## Input/Output Standard

Desktopr plugins communicate through standard execution streams.

- **`stdin`** receives the JSON request.
- **`stdout`** must contain the final JSON response.
- **`stderr`** may contain debug logs and diagnostic messages.

### Input: Bridge → Plugin

The bridge serializes the payload as JSON and sends it to the plugin through `stdin`.

Example:

```js
await Desktopr.plugins.call("math.wasm", {
  fn: "add",
  args: [12, 2]
});
```

The plugin receives:

```json
{ "fn": "divide", "args": [12, 2] }
```

### Output: Plugin → Bridge

The plugin must write a JSON response to `stdout`.

Success:

```json
{"ok":true,"value":6}
```

Error:

```json
{"ok":false,"error":"division by zero"}
```

:::tip MANDATORY FORMAT
The final plugin result must be valid JSON printed to `stdout`.

Extra non-JSON output on `stdout` may cause the result parsing to fail.
:::

## Module Lifecycle Example

### 1. Add a WASM module

```js
await Desktopr.plugins.add("math");
```

This opens a native file picker and stores the selected `.wasm` module in:

```text
_external_modules/math.wasm
```

### 2. List installed plugins

```js
const plugins = await Desktopr.plugins.list();
console.log(plugins);
```

### 3. Execute a function

```js
const result = await Desktopr.plugins.call("math.wasm", {
  fn: "divide",
  args: [12, 2]
});
```

Example response:

```json
{
  "id": "plugin_req2",
  "ok": true,
  "stdout": "{\"ok\":true,\"value\":6}",
  "stderr": "",
  "value": {
    "ok": true,
    "value": 6
  },
  "error": null,
  "durationMs": 24
}
```

## Access a plugin storage with the Desktopr File System module

Desktopr can expose a plugin-scoped filesystem API through the bridge.

Conceptually:

```ts
const pfs = await Desktopr.fs.pluginFs("math.wasm");

await pfs.writeText("config.json", "{}");
const config = await pfs.readText("config.json");
```

This targets:

```text
app_data_dir/.desktopr/.main/_external_modules_storage/math.wasm/config.json
```

The plugin itself can access the same file with:

```rs
std::fs::read_to_string("config.json")?;
```

### Delete behavior

Plugin storage is technical storage owned by the plugin.

When using the plugin filesystem scope, remove operations delete files directly and do not use the `data` trash system.

## Security and Sandbox

Desktopr plugin execution is sandboxed through the WASI runtime.

The runtime provides:

- no network access
- no access to arbitrary user files
- no access to the full app filesystem
- access only to directories explicitly provided by Desktopr
- timeout-based interruption for long-running jobs
- stdout/stderr capture limits
- WASM validation before execution

:::warning
Avoid infinite loops or blocking operations inside your WASM module. Long-running executions may be interrupted by the runtime timeout.
:::

## Load a Plugin Dynamically

If you want to load a module dynamically without opening the file picker, use `addFromBytes`:

```ts
await Desktopr.plugins.addFromBytes("my-plugin.wasm", wasmBytes);
```

`wasmBytes` must be a `Uint8Array` containing the raw bytes of the plugin WASM module.


## Keywords Reference

| Term | Meaning |
|------|---------|
| **Plugin** | A WASM module executed by Desktopr |
| **WASM** | WebAssembly binary module |
| **WASI** | System interface used by the WASM module |
| **Wasmtime** | Native runtime used by Desktopr to execute WASM |
| **Persistent plugin storage** | Per-plugin storage under `_external_modules_storage/<plugin>.wasm/` |
| **stdin** | JSON input sent from Desktopr to the plugin |
| **stdout** | Final JSON response printed by the plugin |
| **stderr** | Debug/diagnostic output captured by Desktopr |

## WebAssembly

[WebAssembly (WASM)](https://webassembly.org/) is a portable binary format that enables high-performance code written in languages such as Rust, C, or C++ to run inside sandboxed runtimes.

Desktopr uses WASM plugins to run local, isolated, cross-platform logic inside the desktop wrapper.

## New Plugin

You can find the guide to develop a plugin [here](/guide/plugin-development.md).

## Notes

:::tip DEVELOPMENT NOTES
- Use `wasm32-wasip1` for Rust plugins.
- Do not use network access; it is blocked by the runtime.
- Use relative filesystem paths for persistent plugin files.
- Do not print extra output to `stdout`.
- The final JSON response must be printed to `stdout`.
- Use `stderr` only for short debug logs and diagnostics.
- Large outputs should be written to plugin storage, not to `stdout` or `stderr`.
- All modules must follow the JSON input/output protocol documented above.
:::

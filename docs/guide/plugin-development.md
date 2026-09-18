---
title: WASM Plugins
description: Execute WebAssembly (WASM) plugins securely inside Desktopr's native Tauri/WASI runtime.
---

# Develop Desktopr Plugins

Desktopr’s **Plugins** provide a secure, sandboxed, and flexible runtime for executing [WebAssembly (WASM)](https://webassembly.org/) modules inside the desktop wrapper.

Plugins let developers extend an app with custom native-side logic, local file processing, data utilities, conversions, validation, computations, and other advanced features without rebuilding the whole app.

:::tip BRIDGE API PLUGINS MODULE
Desktopr plugins are managed and called via the [plugins module](/guide/bridge/api/plugins.md) of the Desktopr bridge API.
:::

## Overview

Desktopr plugins are WASM modules executed by the Desktopr wrapper through a native **Tauri + Wasmtime + WASI** runtime.

Plugins do **not** run inside the browser page. They are executed by the Rust side of the Desktopr wrapper, inside a sandboxed WebAssembly runtime.

The runtime provides:

- **Native WASI execution through Wasmtime**
- **JSON-based stdin/stdout communication**
- **Persistent per-plugin storage**
- **Sandboxing**
- **Automatic timeout handling**
- **Cross-platform consistency on macOS, Windows, and Linux**

:::details NON-RUST BASED WASM
**This guide shows examples and implementations using Rust**, but you can use any language as long as you build a wasm32-wasip1 WASI so that can be execute it with Wasmtime and respects the structure and runtime contract.
:::

## Architecture

The system is composed of three main layers:

| Layer | Language | Responsibility |
|--------|-----------|----------------|
| **Frontend** | JavaScript or TypeScript | Calls plugins through the Desktopr Bridge API |
| **Desktopr Wrapper** | Rust + Tauri | Manages plugin installation, WASM validation, plugin execution, timeout, stdin/stdout, and storage |
| **Execution Runtime** | Wasmtime + WASI | Executes the loaded WebAssembly module in a sandboxed native runtime |

The plugin itself is a `.wasm` file compiled for the WASI target.

```text
Desktopr app
 └─ Tauri/Rust backend
     └─ plugins.rs
         └─ Wasmtime runtime
             └─ my-plugin.wasm
```

## Where Plugins Are Stored

Installed plugin modules are stored in:

```text
app_data_dir/.desktopr/.main/_external_modules/
```

Example:

```text
app_data_dir/.desktopr/.main/_external_modules/my-plugin.wasm
```

Each plugin also gets its own persistent storage directory:

```text
app_data_dir/.desktopr/.main/_external_modules_storage/<plugin>.wasm/
```

Example:

```text
app_data_dir/.desktopr/.main/_external_modules_storage/my-plugin.wasm/
```

This storage is dedicated to that plugin and persists across multiple plugin calls.

## Runtime and filesystem model

A plugin execution is temporary and happens in memory inside Wasmtime.

The plugin’s variables, stack, heap, stdin, stdout, stderr, and runtime state are volatile and disappear when the execution ends.

The plugin filesystem is different:

| Plugin path | Meaning |
|------------|---------|
| Relative paths like `file.txt` or `folder/file.txt` | Resolve inside the plugin’s persistent storage directory |
| Runtime memory | Lives only for the current execution |

Example inside a Rust plugin:

```rs
std::fs::write("config/settings.json", "{}")?;
```

This writes to:

```text
app_data_dir/.desktopr/.main/_external_modules_storage/my-plugin.wasm/config/settings.json
```

Relative filesystem paths are persistent.

## Input/Output Standard

### What `stdin`, `stdout`, and `stderr` mean

Desktopr plugins communicate through standard process streams:

- **`stdin`** = standard input → where the plugin receives the JSON request
- **`stdout`** = standard output → where the plugin must write the final JSON response
- **`stderr`** = standard error → where the plugin may write debug logs or diagnostic messages

In Desktopr plugins:

- the wrapper sends the JSON input to the plugin through **`stdin`**
- the plugin must write the final JSON response to **`stdout`**
- debug logs may be written to **`stderr`**

:::warning
The final plugin result must always be written to `stdout`.

Do not use `stderr` for the final result.
:::

### Input received by the plugin

Desktopr sends a JSON object through `stdin`:

```ts
{
  fn: string;
  args: Array<unknown> | Record<string, unknown>;
}
```

Example with positional arguments:

```json
{ "fn": "divide", "args": [20, 5] }
```

Example with named arguments:

```json
{ "fn": "divide", "args": { "a": 20, "b": 5 } }
```

### Output returned by the plugin

The plugin must print a JSON object to `stdout`.

Success:

```json
{"ok":true,"value":4}
```

Error:

```json
{"ok":false,"error":"division by zero"}
```

The Desktopr runtime captures the raw `stdout`, raw `stderr`, and also tries to parse `stdout` as JSON.

Example Desktopr plugin call response:

```json
{
  "id": "plugin_req2",
  "ok": true,
  "stdout": "{\"ok\":true,\"value\":{\"result\":42}}",
  "stderr": "entered _start\nreading input\ndispatching function\n",
  "value": {
    "ok": true,
    "value": {
      "result": 42
    }
  },
  "error": null,
  "durationMs": 37
}
```

:::tip MANDATORY FORMAT
The plugin must follow the JSON input/output protocol described above.

Invalid, malformed, non-JSON, or noisy `stdout` output may cause the call result to fail or be parsed incorrectly.
:::

## Logging with `stderr`

`stderr` is reserved for debug logs and diagnostic messages.

Anything written to `stderr` is captured by Desktopr and returned in the plugin call response under the `stderr` field.

Example plugin code:

```rs
let _ = std::io::stderr().write_all(b"log message 1\n");
let _ = std::io::stderr().write_all(b"log message 2\n");
```

Example output:

```json
{
//   "id": "plugin_req2",
//   "ok": true,
//   "stdout": "{\"ok\":true,\"value\":{\"result\":42}}",
     "stderr": "log message 1\nlog message 2\n",
//   "value": {
//     "ok": true,
//     "value": {
//       "result": 42
//     }
//   },
//   "error": null,
//   "durationMs": 37
}
```

Use `stderr` for short debug logs only. Do not use it for large outputs, files, datasets, or final results.

## Function Examples

Examples of functions written in Rust for a plugin WASM module:

```rs
use serde_json::{Value, json};

// Named arguments example:
// { "a": 16, "b": 2 }

pub fn eg_divide(args: &Value) -> Result<Value, String> {
    let a = args.get("a").and_then(|x| x.as_f64()).unwrap_or(0.0);
    let b = args.get("b").and_then(|x| x.as_f64()).unwrap_or(0.0);

    if b == 0.0 {
        return Err("division by zero".to_string());
    }

    Ok(json!(a / b))
}

pub fn eg_greet(args: &Value) -> Result<Value, String> {
    let name = args.get("name").and_then(|x| x.as_str()).unwrap_or("world");
    Ok(json!(format!("Hello {}", name)))
}

// Positional arguments example:
// [16, 2]

pub fn eg_divide_positional(args: &Value) -> Result<Value, String> {
    let a = args.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0);
    let b = args.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0);

    if b == 0.0 {
        return Err("division by zero".to_string());
    }

    Ok(json!(a / b))
}
```

## Persistent Storage Example

Relative paths are resolved inside the plugin’s persistent storage directory.

```rs
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn write_file(args: &Value) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap_or("example.txt");

    let contents = args
        .get("contents")
        .and_then(|x| x.as_str())
        .unwrap_or("Hello from Desktopr plugin storage");

    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    fs::write(path, contents).map_err(|e| e.to_string())?;

    Ok(json!({
        "written": true,
        "path": path,
        "bytes": contents.as_bytes().len()
    }))
}

pub fn read_file(args: &Value) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap_or("example.txt");

    let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;

    Ok(json!({
        "read": true,
        "path": path,
        "contents": contents
    }))
}
```

If the plugin writes:

```rs
fs::write("example/hello.txt", "Hello, world!")?;
```

the file is stored at:

```text
app_data_dir/.desktopr/.main/_external_modules_storage/my-plugin.wasm/example/hello.txt
```

The same file can later be managed from the bridge using the plugin filesystem API.

## Access plugin storage from the Bridge API

Desktopr Bridge API exposes version of the [File System](/guide/bridge/api/file-system.md) module scoped to a specific plugin storage.

Conceptually:

```ts
const pfs = await Desktopr.fs.pluginFs("my-plugin.wasm");

await pfs.writeText("hello.txt", "Hello, world!");
const config = await pfs.readText("hello.txt");
```

This targets:

```text
app_data_dir/.desktopr/.main/_external_modules_storage/my-plugin.wasm/hello.txt
```

The plugin itself can access the same file with:

```rs
std::fs::read_to_string("hello.txt")?;
```

### Delete behavior

Plugin storage is technical storage owned by the plugin.

When using the plugin filesystem scope, remove operations delete files directly.

## Security and Sandbox

Desktopr plugin execution is sandboxed through the WASI runtime.

The runtime provides:

- no network access
- no access to arbitrary user files
- no access to the full application data directory
- access only to the directories explicitly provided by Desktopr
- timeout-based interruption for long-running jobs
- stdout/stderr capture limits

WASM modules are validated before execution and must start with the standard `\0asm` magic bytes.

:::warning
Avoid infinite loops or blocking operations inside your WASM module. Long-running executions may be interrupted by the runtime timeout.
:::

## WASM Plugin Structure

Each plugin is an independent WASM module compiled for the `wasm32-wasip1` target and executed as a WASI-compatible program.

:::tip Rust-based examples
This guide shows examples and implementations using **Rust**.
:::

Example structure:

```text
my-desktopr-plugins/
├─ my-plugin/
│   ├─ dist/
│   │   └─ my-plugin.wasm
│   ├─ src/
│   │   ├─ functions/
│   │   │  ├─ example.rs
│   │   │  └─ mod.rs
│   │   ├─ dispatcher.rs
│   │   └─ main.rs
│   └─ Cargo.toml
├─ .gitignore
├─ build.js
└─ package.json
```

## Requirements

A development environment that can compile to **WASI**.

For Rust:

```bash
rustup target add wasm32-wasip1
```

You may also need Node.js if you are using the Desktopr plugin template build scripts.

## Build

Each module is built into `.wasm`.

Using the Desktopr plugin template:

```bash
npm run build <moduleDirName>
```

Example:

```bash
npm run build 'my-plugin'
```

Result:

```text
my-plugin/dist/my-plugin.wasm
```

## → Develop a Plugin

1. Download the plugin [template repo](https://github.com/Desktopr/dtr-plugin-template):

   <small> [**CLICK HERE TO DOWNLOAD THE TEMPLATE**](https://github.com/Desktopr/dtr-plugin-template/archive/refs/heads/main.zip) </small>

2. Rename the project however you like, for example:

   ```text
   my-new-plugin
   ```

3. Edit `Cargo.toml` with the plugin name and description.

4. Implement your functions inside `functions/` and export them from `functions/mod.rs`.

5. Register them in `dispatcher.rs`.

6. Build:

   ```bash
   npm run build 'my-new-plugin'
   ```

7. Load the generated `.wasm` file into your Desktopr app.

8. Call the plugin through the Desktopr Bridge API.

## Dispatcher

The dispatcher maps the `fn` field from the JSON input to a Rust function.

Example:

```rs
use serde_json::{Value, json};
use crate::functions::*;

pub fn dispatch(op: &str, args: &Value) -> Result<Value, String> {
    match op {
        "divide" => divide(args),
        "readFile" => read_file(args),
        "writeFile" => write_file(args),
        _ => Err(format!("unknown function: {}", op)),
    }
}
```

:::warning DISPATCHER CONTRACT
Do not rename `pub fn dispatch` or change its signature:

```rs
pub fn dispatch(op: &str, args: &Value) -> Result<Value, String>
```

Desktopr’s plugin template runtime expects this dispatcher function to exist.
:::

## Template Runtime Files

Some files in the plugin template are part of the runtime contract.

For example:

```text
main.rs
dispatcher.rs
```

Do not change runtime files unless you know exactly what you are changing.

User functions should be added in the dedicated functions module.

Changing template runtime files may break:

- stdin handling
- stdout JSON response formatting
- stderr capture
- function dispatch
- Desktopr plugin compatibility

## Load a Plugin Dynamically

If you want to load a module dynamically without opening a file picker, you can add it from bytes.

Conceptually:

```ts
await Desktopr.plugins.addFromBytes("my-plugin.wasm", wasmBytes);
```

`wasmBytes` must be a `Uint8Array` containing the raw WASM bytes of the module.

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

## Notes

:::tip DEVELOPMENT NOTES
- Keep functions deterministic where possible.
- Do not use network access; it is blocked by the runtime.
- Use relative filesystem paths for persistent plugin files.
- Do not print extra output to `stdout`.
- The final JSON response must be printed to `stdout`.
- Use `stderr` only for debug logs and diagnostics.
- Large outputs should be written to plugin storage, not to `stdout` or `stderr`.
- All modules must follow the JSON input/output protocol documented above.
:::

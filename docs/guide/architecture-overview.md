---
title: Architecture Overview
description: Understanding the internal structure and runtime model of Desktopr
---



# Architecture Overview

## Overview
Desktopr combines a **web frontend**, a **Rust-based [Tauri](./what-is-tauri.md) backend**, and a **web/desktop bridge API** that allows deep OS‑level integrations directly from your web app.

It follows a layered design that separates user interface, system capabilities, and build automation.

## Core Components

| Layer | Description |
|-------|--------------|
| **Frontend Runtime (Webview)** | The user’s web app runs inside a OS native WebView (WebView2 on Windows, WebKit on macOS, WebKitGTK on Linux). |
| **Bridge Layer (Rust/JS)** | The native Rust code handles secure execution of system tasks such as file system, events, files, window management, WASM plugin sandboxing etc. And exposes commands that can be called with JS with the Desktopr internal API using `desktopr` Node module. |
| **Builder Engine (Tauri)** | Wraps the web app, compiles per‑platform executables, and handles signing, icons, and packaging. |
| **Build Automation (GitHub Actions)** | The `build.yml` workflow in the repository builds every platform, signs with your own secrets and publishes the installers as artifacts or a GitHub Release. Nothing leaves your GitHub account. |

## Execution Flow
 
1. **Testing** → Download the [**Desktopr Companion app**](/guide/companion), enter your web app URL and test the Desktopr native features using the `desktopr` Node module.
2. **App Setup** → Fork or clone the [repository](https://github.com/suffro/desktopr) and set your web app URL, app name, identifier and the other options described in [Configuration](/guide/configuration/app-settings).
3. **Build Phase** → Build locally with `npm run build`, or run the **build** workflow in your fork to compile platform‑specific binaries, optionally signed with your own certificates. The app and its build files are entirely yours.

## Bridge Model
Each bridge module corresponds to a Rust subsystem exposed as asynchronous commands in the webview.  
Modules include **files**, **fs**, **window**, **shortcuts**, **network**, **autostart**, **badge**, **plugins**, and more. Check the [bridge documentation](./bridge/overview.md).

Each call invokes a corresponding Rust command managed by Tauri under the hood.

## Plugins System
Desktopr plugins are WebAssembly (WASM) modules executed by Desktopr inside a native Tauri + Wasmtime + WASI runtime.
[Plugins](/guide/plugin-development.md) can implement custom computation logic while maintaining full isolation from the host system.

## Security & Isolation
- Plugins execute in isolated sandboxed environments, without network access and with configurable timeouts and memory caps.
- Access through the File System module is strictly limited to the Desktopr isolated environment. It does not provide access to the global file system of the host machine, ensuring security and sandboxing.
- No system call is directly exposed to the webview; all actions pass through Rust‑level validation.

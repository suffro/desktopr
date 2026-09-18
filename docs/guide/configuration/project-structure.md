---
title: Project Structure
description: How the Desktopr repository is organised and which directory does what.
---

# Project Structure

Desktopr is a single repository that contains the runtime, the bridge, the SDK, the WebAssembly plugin workspace and the build automation.

| Path | Contents |
| --- | --- |
| `src-tauri/` | Rust/Tauri runtime and the native bridge commands |
| `src-ts/` | TypeScript bridge injected into your web app, and the SDK source |
| `sdk/` | The publishable `desktopr` npm package for your web application |
| `wasm/` | WASI plugin template and example modules |
| `apps/companion/` | The [Desktopr Companion](/guide/companion) app |
| `docs/` | This documentation site |
| `conf-templates/`, `scripts/` | Configuration generation for dev and production builds |
| `.github/workflows/build.yml` | Cross-platform build, optional signing and releases |
| `.github/workflows/ci.yml` | Checks and tests on every push and pull request |

## Generated files

Several files in `src-tauri/` are **generated** from `conf-templates/` by the scripts in `scripts/`, and are overwritten on every build:

| Generated file | Generated from |
| --- | --- |
| `src-tauri/tauri.conf.json` | `conf-templates/tauri.conf.template.prod.json` (or `.dev.json`) |
| `src-tauri/capabilities/remote.json` | `conf-templates/remote.template.json` |
| `src-tauri/Cargo.toml` | `conf-templates/Cargo.template.toml` |
| `src-tauri/window.env` | `scripts/prod-conf.sh` |
| `src-ts/bridge.constants.json` | `conf-templates/bridge.constants.template.json` |

Edit the **templates**, not the generated files, and regenerate with `npm run devconf` (development) or `npm run prodconf` (production).

## The bridge bundle

The TypeScript bridge is bundled to `src-tauri/tsc/bridge.js` and embedded into the Rust binary at compile time. It is not committed, so run `npm run ts:compile:bridge` before any `cargo` command:

```sh
npm run ts:compile:bridge
cd src-tauri && cargo check --locked
```

## Where your web app fits

Your web application is **not** part of this repository. Install the [`desktopr` SDK](/guide/bridge/overview) in it to call native features, then choose how the app reaches it:

- **as a URL**, with `APP_URL` — the app loads your site, and only that origin is granted the bridge;
- **embedded**, with `APP_FRONTEND=bundled` and `APP_FRONTEND_DIST` pointing at your build output — the app runs offline from its own files. In CI the [build action](/guide/configuration/github-actions) builds your app in its own repository and hands the directory over.

With neither, Desktopr ships the bundled local page, which is what the default configuration and the Companion app do.

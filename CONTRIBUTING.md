# Contributing to Desktopr

Thanks for your interest in improving Desktopr. Bug reports, fixes,
documentation and new native capabilities are all welcome.

## Before you start

- For bugs, open an issue with your platform, the Desktopr commit or version,
  and steps to reproduce.
- For larger changes (new bridge APIs, configuration changes, workflow changes),
  open an issue first so the approach can be agreed before you write the code.
- Security problems must be reported privately, see [SECURITY.md](SECURITY.md).

## Setup

Follow the requirements in the [README](README.md#requirements), then:

```sh
npm ci
npm run dev
```

## How the pieces fit together

- A native capability usually spans three places: a Rust command in
  `src-tauri/src/bridge/`, registered in `src-tauri/src/main.rs` and allowed in
  `src-tauri/permissions/desktopr-bridge.toml`; a bridge module in
  `src-ts/modules/rs/<module>/`; and its wiring into the `Desktopr` API in
  `src-ts/desktopr/`, which the SDK exposes through a typed proxy. Keep command
  names and argument shapes in sync.
- Shared configuration contracts (such as the menu schema in
  `src-ts/config/`) must match the Rust structs that deserialize them.
- `src-tauri/Cargo.toml`, `tauri.conf.json`, `capabilities/remote.json`,
  `window.env` and `src-ts/bridge.constants.json` are generated from
  `conf-templates/` by the scripts in `scripts/`. Edit the templates, then
  regenerate with `npm run devconf`.
- WASM plugins live in the `wasm/` Cargo workspace; see
  [wasm/README.md](wasm/README.md).

## Checks

Run what applies to your change before opening a pull request:

```sh
npm run typecheck
npm run ts:compile:bridge
(cd src-tauri && cargo check --locked)
npm run check:standalone
npm run check:private-deps
npm run check:menu-contract
npm run check:bridge-permissions  # when adding or changing bridge commands
npm run test:wasm-runtime        # when touching the plugin host or wasm/
```

When you change `.github/workflows/`, lint it with
[actionlint](https://github.com/rhysd/actionlint) (with `shellcheck` installed).

## Ground rules

- Desktopr must keep working without any hosted service. Do not add required
  network endpoints, telemetry, or default remote origins.
- Optional features (updater, signing, external app URLs) stay opt-in and
  configured by the developer.
- Never commit credentials, and never pass secrets as workflow inputs.
- Do not add restricted `com.apple.developer.*` entitlements; unsigned macOS
  builds would no longer launch.
- Only add dependencies from public registries, and keep lockfiles committed.

## Pull requests

- Keep each pull request focused on one change and describe what it does and
  how you verified it.
- Add or update tests and documentation when behavior changes.
- Make sure the build workflow still passes on all platforms when you touch the
  runtime, configuration or workflows.

By contributing, you agree that your contributions are licensed under the
[Apache License, Version 2.0](LICENSE).

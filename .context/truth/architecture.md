# Architecture

## Overview

`tauri-skeleton` is the target Desktopr monorepo. The runtime stays at the
repository root during the migration to avoid a broad move before a standalone
build is established. The repository root is also the npm workspace root.

## Major components

- `src-tauri/`: Rust/Tauri runtime and native bridge commands.
- `src-ts/`: TypeScript bridge source and SDK source.
- `src-ts/config/`: shared, standalone configuration contracts used by the
  bridge and exported by the SDK. The menu contract mirrors Rust deserialization
  and owns the JSON Schema used at the TypeScript boundary.
- `sdk/`: publishable `desktopr` JavaScript/TypeScript SDK workspace. Its
  output (`sdk/dist-sdk/`, not committed) is generated from `src-ts/` through
  `tsconfig.sdk.json` and rebuilt before publishing.
- WASM plugins run only through the native Wasmtime host in
  `src-tauri/src/bridge/plugins.rs`; the old JavaScript worker runner and its
  `dtr_worker_*` bridge module were removed in phase 9.
- `src-tauri/src/bridge/acl.rs`: grants capabilities at runtime. The static
  `remote` capability covers only `main`; windows opened with `dtr_win_open` get
  a copy for their label and companion windows a reduced one. Filesystem
  commands derive their scope from the calling window (see
  `decisions/window-isolation-and-capabilities.md`).
- `wasm/`: private npm and Cargo workspace for WASI Preview 1 modules. It owns
  the module template, the math example, one Cargo lockfile and the build helper
  that publishes ignored per-module artifacts.
- `apps/companion/`: Desktopr Companion SvelteKit app (npm workspace), bundled
  with `APP_FRONTEND=companion` and released by
  `.github/workflows/companion-release.yml` (see
  `decisions/companion-app.md`).
- `src-tauri/standalone/`: bundled local fallback UI used when a developer does
  not configure an external application URL.
- `conf-templates/` and `scripts/`: runtime configuration generation.
- `.github/workflows/build.yml`: cross-platform build, optional signing,
  GitHub Actions artifacts and optional GitHub Release. It builds this checkout
  directly; see `decisions/github-actions-build.md`.
- `.github/workflows/ci.yml`: checks and tests on every push to `main` and pull
  request. Repository checks, TypeScript, bridge/SDK tests
  (`scripts/test-bridge.mjs`, which loads the bundled bridge in a VM with a
  fake Tauri global) and configuration generation tests
  (`scripts/test-config-generation.mjs`) run on Linux; rustfmt, clippy and Rust
  tests on Linux, Windows and macOS.

The source repositories have these roles (all four are archived on GitHub):

- `project-globals`: phase 4 absorbed only the useful menu contract into
  `src-ts/config/menu.ts`. The old general Tauri types were stale and unused;
  Firebase, Stripe, hosted-build and Bubbledesk service data remain excluded.
- `wasm-module-template`: absorbed into `wasm/module-template/` in phase 5.
- `wasm-modules`: its useful `math` example was absorbed into
  `wasm/modules/math/` in phase 5.
- `companion`: imported into `apps/companion/` in phase 12 without history or
  lockfile.
- `github-actions`: legacy build/sign/distribution workflows, replaced in
  phase 7 by the in-repository `build.yml`. No hosted distribution, private
  wrapper checkout or logging worker logic was copied.

## Data flow

The TypeScript bridge is bundled to `src-tauri/tsc/bridge.js` and embedded by
the Rust runtime. The SDK exposes the same bridge surface to web applications.
The Rust runtime executes WASI modules through Wasmtime and exposes native
filesystem, window, clipboard, notification, menu, shortcut, diagnostics,
networking, plugin storage, deep-link and autostart commands.

WASM plugins target `wasm32-wasip1`, read a `{ "fn", "args" }` request from
stdin and emit one JSON response to stdout. The host invokes `_start`, enforces
I/O and time limits, disables networking, and preopens the temporary job
directory plus dedicated persistent plugin storage.

## External systems

- Runtime dependencies come from public npm and crates.io registries.
- The default runtime loads bundled local content and grants no remote origin.
- The runtime can load a developer-provided web application URL; configuration
  generation grants only that origin and its subdomains.
- The updater is excluded from default Cargo features. A developer can opt in
  with the `updater` feature and must supply both their own HTTPS endpoint and
  signing public key.
- CI builds use GitHub-hosted runners, first-party `actions/*` actions,
  crates.io and npm. Build outputs go only to GitHub Actions artifacts and
  optional GitHub Releases. Signing credentials are the consumer repository's
  own secrets.
- Generic network diagnostics currently use public Cloudflare and Google
  probes. These are not Desktopr SaaS endpoints. App-chosen probe URLs may target
  local and LAN hosts; they are limited to HTTP(S) with bounded timeout,
  download size and polling rate.

## Important constraints

- Do not move or refactor the runtime broadly before the standalone build gate.
- Do not import `suffro-lib`, credentialed Git URLs, Firebase/Stripe types or
  hosted-service constants into the monorepo.
- Keep shared TypeScript configuration contracts aligned with the Rust structs
  that deserialize them.
- Keep `docs` separate.
- The updater must remain disabled unless explicitly configured by a developer.

# Desktopr OSS migration inventory

## Current focus

Verified inventory through phase 6 of the Desktopr OSS migration. This document
records source-repository boundaries; `tauri-skeleton` remains the only target.

## Recent relevant changes

### Cross-repository map

- `tauri-skeleton` already contains the Rust/Tauri runtime, TypeScript bridge
  and `desktopr` SDK source. It has no package or Cargo dependency on the other
  four repositories.
- `github-actions/.github/workflows/build.yml` checks out a configurable wrapper
  repository using `TAURI_WRAPPER_REPO` and a private-repository token. It is
  operationally coupled to the wrapper, R2 uploads, the Desktopr CDN and a
  hosted updater manifest. `sign.yml` and `dist.yml` are also R2-dependent.
- `project-globals` points at the `Bubbledesk/github-actions` workflow through
  constants, but it is not consumed by the target. Its generic `src/tauri/`
  types and menu schemas are self-contained candidate code; its root exports,
  environment values and Firestore types are SaaS-specific.
- `wasm-module-template` and `wasm-modules/math` share the same Rust/WASI JSON
  stdin/stdout protocol. They do not have a package dependency on each other or
  on the target. The target already provides the Wasmtime host and worker API.
- The legacy `frontend/` depends on the published `desktopr` SDK rather than the
  local SDK. It is companion work and remains intentionally out of scope.

### Private dependency findings

- The target npm and Cargo manifests/locks contain only public registry
  dependencies and no credentialed URLs.
- The separate `project-globals` manifest and lockfile contain `suffro-lib` via
  a credentialed Git URL. The credential value was not printed or copied.
- `project-globals` uses these `suffro-lib` types outside `src/tauri/`:
  `ColorInfo`, `FirebaseTimestamp`, `StringLiteralJoin`, `AppUser`, `AppUserDoc`,
  `CoreDocFields`, `DocBlueprint`, and `SubcolelctionDocBlueprint`.
- No replacement is needed in the target for phases 1-3: all files using those
  types are SaaS/business data that must not be imported. The future Tauri/menu
  subset has no `suffro-lib` dependency.

### Hosted-service findings

- Target runtime/config: retired companion defaults, unconditional remote
  capability and mandatory updater inputs were removed in phase 6. Public
  Cloudflare/Google probes remain as opt-in generic diagnostics rather than
  Desktopr infrastructure.
- Legacy Actions: private wrapper checkout, Desktopr CDN updater URL, R2 build,
  signing and distribution storage, and signing credentials passed as normal
  workflow inputs.
- `project-globals`: Bubbledesk domains, Firebase/App Hosting, Firestore,
  Stripe, R2/CDN and hosted build-runner data.
- WASM repositories: their useful code is integrated under `wasm/` without the
  source repositories' hosted links or unused npm `wasmtime` dependency.

### Build snapshot

- Target: `npm ci`, runtime/SDK TypeScript typechecks, bridge bundle and
  `cargo check --locked` pass. Rust emits existing warnings.
- Integrated WASM template and `math` module: checks, release builds and protocol
  execution pass.
- `project-globals` was not installed or built because doing so would consume
  the embedded private credential and install SaaS dependencies.
- Legacy Actions were inspected but not executed.

## Next

Phases 4-6 are complete. Phase 7 replaced the legacy Actions with the
in-repository `.github/workflows/build.yml`; its first GitHub run is pending.

## Blockers

- Revoke/rotate and remove the exposed credential from the separate
  `project-globals` repository and its history before sharing that repository.
- The separate legacy Actions repository still contains private wrapper
  checkout and hosted R2/CDN distribution. None of it was copied; it can be
  archived once the new workflow is verified.
- The legacy `frontend/` companion UI still has a retired dashboard link and is
  deferred to the companion phase.

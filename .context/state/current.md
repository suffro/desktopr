# Current State

## Current focus

Desktopr OSS migration phase 6 completed: the base runtime now starts from
bundled local content and has no required Desktopr hosted-service dependency.

## Recent relevant changes

- Inventoried `tauri-skeleton`, `github-actions`, `project-globals`,
  `wasm-module-template` and `wasm-modules`; `docs` and `companion` were not
  inspected or changed.
- Confirmed that the target runtime and SDK have no `suffro-lib`, Git or private
  package dependency. Added `npm run check:private-deps` as a non-secret-leaking
  manifest/lockfile guard.
- Found `suffro-lib` plus an embedded GitHub credential in the separate
  `project-globals` manifest and lockfile. That repository was not copied or
  modified; only the independently reviewed menu contract was reimplemented in
  the target during phase 4.
- Marked the repository root private and initialized npm workspaces with the
  existing `sdk/` package. The runtime stays in place.
- Verified `npm ci`, TypeScript runtime/SDK typechecks, bridge bundling and
  `cargo check --locked`. Rust succeeds with existing warnings after the bridge
  bundle is generated.
- Added the canonical Desktopr menu types and JSON Schema under
  `src-ts/config/menu.ts`, adapted to the actual Rust runtime rather than copied
  with Bubbledesk naming.
- Replaced the duplicated manual menu validation with schema validation before
  IPC. Required fields and predefined items now match the Rust deserializer;
  recursive submenus and separators remain supported.
- Exported the menu types and schema from the generated SDK and added
  `npm run check:menu-contract` as a focused regression check.
- Deliberately did not import the old `BdTauriConfig` types: they are incomplete
  for the current Tauri configuration and have no target consumer.
- Confirmed that target source/build inputs contain no dependency or reference
  to `project-globals`, `suffro-lib`, or `BdTauri` symbols.
- Fixed `npm run sdk:build` so it no longer rewrites SDK package version `2.2.0`
  to bridge API version `2.0.3`. Package release version and runtime API version
  are now intentionally independent.
- Added the private `@desktopr/wasm` workspace under `wasm/`, with a shared Cargo
  workspace, one lockfile and deterministic build helper.
- Integrated the original module template and math module without their stale
  hosted links, standalone npm packaging or unused `wasmtime` npm dependency.
- Preserved math array/object argument compatibility and the template's example
  function names while tightening stdin handling to the runtime's 1 MiB limit.
- Built both crates for `wasm32-wasip1` and exercised success/error responses
  with the local Wasmtime CLI. The artifacts use the runtime's existing
  `{ "fn", "args" }` stdin and single-JSON stdout protocol.
- Replaced the retired companion URL in build defaults, generated capabilities
  and dev configuration with a bundled standalone page and an empty external
  application URL.
- External web applications remain supported only when a developer explicitly
  supplies `APP_URL`; the matching remote capability is generated from that
  developer-owned origin.
- Removed mandatory production updater inputs. The updater dependency is no
  longer a default Cargo feature and is registered only with `--features
  updater`; endpoint, public key, artifact generation and permissions are added
  only when both developer-owned updater inputs are supplied.
- Removed hosted Desktopr/Bubbledesk links from core source and SDK material.
  Added `npm run check:standalone` to guard the standalone defaults without
  printing matched content.
- Verified default and explicit updater configuration generation, TypeScript,
  SDK generation, bridge bundling, default Rust compilation and Rust compilation
  with the updater feature.
- Aligned the production generator's default version with the checked-in Rust
  package/lockfile version so the standalone production profile passes
  `cargo check --locked`.

## Next

Stop before phase 7. The next planned step, when explicitly requested, is phase
7: replace the legacy hosted Actions flow with in-repository build/signing
workflows and GitHub-hosted artifacts/releases.

## Blockers

- The separate `project-globals` repository contains a credential in current
  manifest/lockfile data. It must be revoked/rotated and scrubbed in that source
  repository and its history before that repository can be shared. The value
  was never printed or copied.
- Phase 7 still needs to replace the separate Actions workflows, which require
  a private wrapper token and R2/CDN distribution.
- The legacy `frontend/` companion UI still links the retired dashboard. It is
  intentionally excluded until the planned companion phase and is not required
  by the standalone runtime.
- `cargo check` requires `npm run ts:compile:bridge` first because the generated
  bridge bundle is intentionally ignored by Git.

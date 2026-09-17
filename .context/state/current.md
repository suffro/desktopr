# Current State

## Current focus

Desktopr OSS migration phase 8 completed for unsigned builds: the standalone
runtime builds and launches on Linux, Windows and macOS through
`.github/workflows/build.yml`, which signs optionally from repository secrets
and publishes only GitHub artifacts or an optional GitHub Release.

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

- Phases 1-6 were committed and pushed to `main` as `c659dcf`.
- Added `.github/workflows/build.yml` (`workflow_dispatch` and
  `workflow_call`) replacing the legacy build/sign/dist workflows without R2,
  CDN, private wrapper checkout, hosted logging or credential inputs.
- Signing uses Tauri's native Windows certificate-store and macOS
  `APPLE_*` mechanisms; unsigned macOS builds are ad-hoc signed. Updater
  artifacts are built only with both updater inputs and the private key secret.
- CI keeps the Cargo package name/version from `Cargo.lock` and applies the app
  version to `tauri.conf.json`; verified that coupling them breaks `--locked`.
- The production template now also bundles `dmg`.
- Windows builds can opt into an MSIX package through the `msix_*` inputs; it
  is signed only when the Windows certificate subject matches the MSIX
  publisher.
- First GitHub run (35125323826, commit `7563fc9`) succeeded on Linux
  (~19 min), macOS (~21 min) and Windows (~47 min, including an uncached
  `cargo install tauri-cli`). Artifacts: AppImage/deb/rpm, NSIS setup and an
  unsigned MSIX with the expected manifest, and an ad-hoc signed DMG with the
  requested identifier/version. Package checksums verified.
- That run exposed a self-referencing `SHA256SUMS` entry (fixed) and a Node 20
  deprecation warning (artifact actions bumped to `upload-artifact@v7` and
  `download-artifact@v8`).
- A Linux verification run (35139886909) confirmed both fixes but showed the
  cached `src-tauri/target` leaking bundles from the previous run into the
  artifact; bundles are now removed before building and before the cache save.
  Run 35140696294 (commit `de1b219`) verified it: only the current packages,
  valid checksums, no annotations, ~5 min with a warm cache.
- Verified: `actionlint` with `shellcheck` passes (and reports injected
  errors); the macOS job sequence reproduced locally produced an ad-hoc signed
  DMG with the requested version/identifier. The local run used tauri-cli
  2.8.4, while CI pins 2.11.1.

- Phase 8 validation from a clean install passed: `npm ci`, typecheck, bridge,
  SDK build with no drift, `cargo check --locked` (default and `updater`),
  repository checks, WASM check/build.
- The WASM executor was extracted from `AppHandle` and covered by tests:
  invalid bytes, missing `_start`, timeout interruption, and the real math and
  template modules (protocol, `/storage` persistence, temporary job files).
  `npm run test:wasm-runtime` builds the modules and runs them.
- Found that unsigned/ad-hoc macOS builds were killed at launch because
  `Entitlements.plist` declared the restricted
  `com.apple.developer.usernotifications.time-sensitive` entitlement (unused by
  the code). Removed it; `check:standalone` now rejects `com.apple.developer.*`.
- Found that Linux startup failed when `update-desktop-database`/`xdg-mime` are
  missing, because deep link registration errors aborted the setup hook. It now
  logs a warning and continues.
- The custom panic hook replaced the default one, so panics were silent unless
  crash reports were enabled. It now chains to the default hook.
- `build.yml` launches the built app on every platform for 15 seconds. Run
  35168452799 (commit `b592858`) passed on Linux, Windows and macOS with MSIX,
  verified checksums and no developer entitlements in the DMG app.

## Next

Phase 8 is complete except signing with real certificates, which needs
developer secrets. Phase 9 (OSS cleanup) is next. Candidate hardening notes
found during phase 8: Linux AppImage bundling downloads an unpinned
`linuxdeploy-plugin-gtk.sh` at build time (one run failed on a network reset);
each WASM call keeps a timeout thread sleeping for the full timeout.

## Blockers

- The separate `project-globals` repository contains a credential in current
  manifest/lockfile data. It must be revoked/rotated and scrubbed in that source
  repository and its history before that repository can be shared. The value
  was never printed or copied.
- Real-certificate signing, notarization, the updater build and the GitHub
  Release job have not run; they need developer secrets or an explicit release.
  Local test launches of GUI apps require running outside the command sandbox.
- Running it on this private repository consumes GitHub Actions minutes
  (macOS and Windows runners are billed at higher multipliers).
- The legacy `frontend/` companion UI still links the retired dashboard. It is
  intentionally excluded until the planned companion phase and is not required
  by the standalone runtime.
- `cargo check` requires `npm run ts:compile:bridge` first because the generated
  bridge bundle is intentionally ignored by Git.

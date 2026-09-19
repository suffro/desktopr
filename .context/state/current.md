# Current State

## Current focus

Desktopr OSS migration phases 12-14 completed: the repository is public as
`suffro/desktopr`, the companion 3.0.0 release is published and the docs are part
of the monorepo. The first OSS release of the runtime itself is still pending. The standalone runtime builds and launches on Linux, Windows and
macOS through `.github/workflows/build.yml`; signing with real certificates is
still unverified.

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
- The custom panic hook replaced the default one, so panics were only written
  to diagnostics crash files (crash reports are enabled by default) and never
  reached stderr. It now chains to the default hook.
- Phase 9 cleanup: removed tracked `.DS_Store` files, `deprecated/`, the unused
  `src-tauri/blank.html`, committed SDK build output (now ignored), the legacy
  JavaScript WASM worker (`src-ts/worker`, `dtr_worker_*` bridge module,
  `build:worker`, `@wasmer/wasi`, `buffer`), the disabled startup menu loaded
  from `menu.config.json` (templates, resource file, dev-conf copy), unregistered
  `dtr_fs_sandbox_*`/diagnostics write commands, other compiler-reported dead
  helpers and unused imports. Rust warnings went from 56 to 19; remaining ones
  are unused command parameters (part of the IPC argument names), the unused
  glob re-exports in `bridge/mod.rs`, the `IsMenuItem` trait import kept for
  non-macOS menu code, the `section` menu field kept for the JSON contract, and
  two small state accessors.
- Kept on purpose: the commented `tauri-plugin-prevent-default` setup (with its
  still-declared dependency) and companion `[DEBUG]` logging, pending the
  hardening and companion phases.
- Renamed the local dev identifier from `app.desktopr.dashboard` to
  `app.desktopr.local`.
- Added `LICENSE` (official Apache 2.0 text), `README.md` (replacing a copy of
  the SDK README), `SECURITY.md` (GitHub private vulnerability reporting and the
  trust model) and `CONTRIBUTING.md`, plus `Apache-2.0` license fields in the npm
  and Cargo manifests.
- Run 35170008647 (commit `775803a`) verified the cleanup: all three platforms
  build and pass the launch smoke test, with valid artifacts and checksums.
- `build.yml` launches the built app on every platform for 15 seconds. Run
  35168452799 (commit `b592858`) passed on Linux, Windows and macOS with MSIX,
  verified checksums and no developer entitlements in the DMG app.
- Phase 10 hardening: window identity is derived in Rust (injected
  `WebviewWindow`); filesystem commands, including clear/trash/paths, use the
  caller's scope and reject a mismatching `windowLabel`; companion windows cannot
  use plugin storage. The static `remote` capability now covers only `main`, and
  `bridge/acl.rs` grants runtime capabilities to windows from `dtr_win_open`
  (full) and companions (reduced `desktopr-bridge-companion`, no autostart,
  updater or debug permissions). `dtr_win_open` rejects the companion label
  prefix, no longer fails without a declared main window config and no longer
  has a stray `getrandom` assert; companion launch validates the URL before
  creating a sandbox and cleans up on failure instead of panicking.
- Added `npm run check:bridge-permissions` (TS invocations vs `main.rs` vs the
  permission sets). It surfaced debug diagnostics commands that were never
  allowed; they now live in `desktopr-bridge-debug`, granted only by the dev
  config scripts.
- Network probes accept only HTTP(S) URLs, clamp timeouts to 30 s, stop the
  bandwidth download at 10 MB and enforce a 1 s monitor interval with at most 10
  targets. LAN/localhost stay reachable by decision.
- The WASM timeout thread now exits when the call finishes. Removed the unused
  `tauri-plugin-prevent-default` and `getrandom` dependencies.
- Actions are pinned to commit SHAs with Dependabot updates; Linux builds
  pre-provision SHA-256-verified AppImage tools (scripts at fixed commits,
  `linuxdeploy-plugin-appimage` at release `1-alpha-20250213-1` instead of
  `continuous`).
- Verified locally: unit tests (each new guard also observed failing), a debug
  build self-test page with 24 expected allowed/denied calls across main, a
  runtime-opened window and a companion, plus all repository checks and
  actionlint. SECURITY.md documents the resulting trust model.
- Run 35209135359 (commit `bca61a3`) passed on Linux, Windows and macOS with
  launch smoke tests, MSIX, valid checksums, all five AppImage tools verified
  and no bundler downloads from remote sources.

- The absorbed source repositories `project-globals`, `wasm-module-template`,
  `wasm-modules` and `github-actions` were archived on GitHub (read-only, not
  deleted). The `project-globals` credential still needs to be revoked.
- Phase 11: added `.github/workflows/ci.yml` (push to `main`, pull requests).
  Applied rustfmt to the runtime crate and made clippy pass with `-D warnings`
  on Linux, Windows and macOS; the first run caught a macOS-only field that
  was dead code elsewhere. New npm scripts: `test:bridge`, `test:config`,
  `test:rust`, `lint:rust`.
- `scripts/test-config-generation.mjs` runs the prod/dev/local generators in
  temporary copies (defaults, `APP_URL`, companion mode, updater validation)
  and fails when the generated `Cargo.toml` drifts from the checked-in one.
- `scripts/test-bridge.mjs` loads the bundled bridge in a Node VM with a fake
  Tauri global (frozen `window.Desktopr`, idempotent init, `dtrReady`, IPC
  argument names, companion prefix shared with Rust) and tests the SDK outside
  the wrapper. It found that the documented `isDesktoprAvailable()` was not
  exported; it now is, and missing-bridge access throws a descriptive error.
- Added Rust tests for scope label validation, plugin storage module names
  and WASM upload validation. Each new test was observed failing against a
  broken guard.
- CI run 35211551764 (commit `cd36dbf`) failed clippy on Linux and Windows
  (`macos_root` only read on macOS). Run 35212428698 (commit `1a1d20d`) passed
  all jobs: repository checks and TypeScript, WASM modules, and the runtime on
  Linux, Windows and macOS.

- Phase 12: imported the companion app into `apps/companion/` (see
  `decisions/companion-app.md`) and removed `frontend/`, the local-dev profile
  and its template. `APP_FRONTEND=companion` bundles it; `build.yml` gained
  `frontend`, `release_tag_prefix` and `release_notes_file`;
  `companion-release.yml` publishes signed releases. CI runs
  `companion:check` and `companion:build`. Companion windows without a URL now
  open `/` instead of the missing `/cache-only/blank.html`.
- CI run 35244079211 (commit `2071629`) passed. Companion build run 35244107334
  passed on Linux, Windows and macOS with launch smoke tests, valid checksums
  and an MSIX with the Store identity (`Desktopr.Desktopr`, display name
  `Desktopr`). A local macOS build launched and created the companion window;
  the UI and loading a web app were not checked visually.

- The Apple signing secrets are set in the repository. Notarization first failed
  with Apple's misleading `403 ... required agreement is missing`: the cause was
  a stale app-specific password, not an agreement. `APPLE_PASSWORD` now holds a
  working one.
- Companion release run 35247666202 (commit `4405603`) created the draft release
  `companion-v3.0.0` with the Linux, Windows and macOS artifacts. Verified from
  the release: checksums match, the MSIX is `3.0.0.0` with the Store identity,
  and the DMG's app is Developer ID signed, notarized and stapled
  (`spctl: source=Notarized Developer ID`).
- GitHub renames assets with spaces, which broke `SHA256SUMS` verification for
  downloaders. The release job now renames artifacts and rewrites the checksum
  files before publishing; the 3.0.0 checksum assets were replaced by hand.

- The `companion-v3.0.0` release is published (9 assets). The DMG's app is
  Developer ID signed, notarized and stapled; the MSIX is `3.0.0.0` with the
  Store identity. Notarization first failed with Apple's misleading
  `403 ... required agreement is missing`, which was a stale app-specific
  password; `APPLE_PASSWORD` now holds a working one.
- The repository was renamed and moved to `suffro/desktopr` and made public
  before the phase 13 scan. The scan then found the revoked `suffro-lib` token
  in 5 old commits; the history was rewritten with `git filter-repo` and
  force-pushed (same tree, same 216 commits, new SHAs), with a backup in
  `~/desktopr-history-backup-20260918-0344.git`.
- Phase 14: the documentation was absorbed into `docs/` as the `@desktopr/docs`
  workspace and rewritten for the open-source project (see
  `decisions/documentation-site.md`). `npm run docs:build` runs in CI and fails
  on dead links. The site is deployed to Cloudflare by the user.
- Cargo cache keys now include the repository name: Tauri writes absolute paths
  into its generated build artifacts, so caches saved as `tauri-skeleton` broke
  every runtime job after the move. CI run 35348570372 is green on all
  platforms.

- Every per-platform build step moved from `build.yml` into the composite action
  `action.yml` at the repository root; the workflow shrank from 925 to ~450
  lines and now only validates inputs, resolves the matrix, calls the action
  with `uses: ./` and releases. Other repositories use it as
  `suffro/desktopr@v1` against their own web app (see
  `decisions/external-build-action.md`). The root is where GitHub Marketplace
  requires the metadata file, and only one action per repository can be listed;
  signing is part of this action, not a separate one.
- `prod-conf.sh` gained the `bundled` frontend mode (`APP_FRONTEND_DIST`), which
  embeds a developer-built web app with no remote origin and the `'self'` CSP.
  `build.yml` exposes it as `frontend: bundled` + `frontend_dist` for a
  directory already present in the checkout.
- The Cargo cache key now includes a hash of the runtime checkout path, which
  also covers the action being checked out under `_actions/<owner>/<repo>/<ref>`.
- Verified locally: `actionlint` (no new finding; the pre-existing
  `workflow_dispatch` input-count warning is unchanged), `shellcheck` on the
  action's bash steps and on `prod-conf.sh`, the action/workflow input and
  output cross-check, runtime-root resolution for both action paths,
  `npm run test:config` (6 new `bundled` scenarios, each guard observed
  failing), `check:standalone`, `check:private-deps` and `docs:build`.
- Run 35383004743 (branch `build-action`, commit `59711df`) passed on all three
  platforms through the action: Linux 12 min, macOS 10 min, Windows 14 min, with
  AppImage/deb/rpm, an aarch64 DMG and the NSIS setup, and the 15-second launch
  smoke test green everywhere. The macOS app was signed with the real Developer
  ID certificate and notarization finished `Accepted`, so passing secrets as
  action inputs works.
- Run 35383013081 exercised `frontend=bundled` with
  `frontend_dist=src-tauri/standalone`: the action staged the directory,
  `prod-conf.sh` reported `frontend -> frontend-dist`, the generated
  `remote.urls` stayed empty and the app launched.
- Run 35386270081 (commit `0e454b7`) confirmed the move to the repository root:
  the action resolved its runtime to the workspace and built and launched a
  `frontend=bundled` app on Linux. The `v1` tag now points at that commit.
- Added `sign: false` to the build action and the standalone
  `.github/actions/sign` action for artifacts built elsewhere. Signing stays per
  platform and driven by which credentials the caller passes.
- Still unverified: the cross-repository path, where another repository uses
  `suffro/desktopr@v1`. Every run so far used the local `./` path; the
  `_actions/<owner>/<repo>/<ref>` layout was checked by simulating the path,
  not by a real run.

## Next

Run a `frontend_dist` build from a separate repository to exercise the
cross-repository action path. Publishing the action in GitHub Marketplace is
then a manual step in the GitHub UI (a release with the Marketplace box ticked,
plus 2FA and the Marketplace Developer Agreement); it is optional, since a
public repository's action already works for everyone. Then cut the first OSS release of the runtime
itself (tag `v<version>` through `build.yml` with
`release: true`), and decide whether to archive the source `docs` and
`companion` repositories. Phase 13 (secret scan) comes
after, only when requested. The runtime ACL self-test from phase 10 is still
manual (it needs a real webview). Known remaining items: diagnostics reads stay available to companion windows.

## Blockers

- The `suffro-lib` credential was revoked by the user. It still exists in the
  history of the private `project-globals` and `companion` repositories, which
  must stay private or be scrubbed before sharing.
- Real-certificate signing, notarization, the updater build and the GitHub
  Release job have not run; they need developer secrets or an explicit release.
  Local test launches of GUI apps require running outside the command sandbox.
- `ci.yml` runs on every push to `main` and pull request (Markdown and
  `.context/` changes are skipped), including Windows and macOS runners.
- `build.yml` and `ci.yml` consume GitHub Actions minutes; public repositories
  get free minutes, but macOS and Windows runners remain the slow part.
- The source `companion` repository also contains the `suffro-lib` credential
  in its manifest (and likely lockfile and history). It was not copied; the
  credential must be revoked.
- `SECURITY.md` points reporters to GitHub private vulnerability reporting,
  which must be enabled in the repository settings now that the repository is
  public.
- `cargo check` requires `npm run ts:compile:bridge` first because the generated
  bridge bundle is intentionally ignored by Git.

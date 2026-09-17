# Conventions

## Repository conventions

- The repository root owns the runtime and the npm workspace configuration.
- New JavaScript/TypeScript packages must be public-dependency-only workspaces.
- Run `npm run check:private-deps` after changing npm, Cargo or registry
  manifests. The check reports locations and categories without echoing secret
  values.
- Run `npm run check:standalone` after changing runtime defaults, capabilities,
  configuration generation or updater wiring.
- Do not commit generated output: `sdk/dist-sdk/`, `src-tauri/tsc/` and WASM
  `dist/` directories are rebuilt by their scripts (`check:menu-contract`
  builds the SDK itself).
- Contributor-facing rules live in `CONTRIBUTING.md` and `SECURITY.md`; keep
  them consistent with these conventions.
- Do not copy generated output, SaaS data models or hosted distribution logic
  merely to preserve the layout of a source repository.
- WASM crates belong to the shared `wasm/Cargo.toml` workspace and must target
  `wasm32-wasip1`. Register new build artifacts in `wasm/build.mjs`; do not add
  a JavaScript Wasmtime runtime dependency.
- WASM modules must emit only their JSON response on stdout. Keep diagnostics on
  stderr and retain the 1 MiB stdin boundary used by the host.

## Development workflow

- Compile the bridge before Rust checks because `src-tauri/src/desktopr.rs`
  embeds `src-tauri/tsc/bridge.js`.
- `.github/workflows/ci.yml` is the verification gate for pushes to `main` and
  pull requests. Locally run what applies: `npm ci`, `npm run
  check:private-deps`, `npm run typecheck`, `npm run check:menu-contract`,
  `npm run test:bridge`, `npm run test:config`, `npm run lint:rust` and
  `npm run test:rust` (builds the WASM modules and runs all Rust tests).
- Rust code must pass `cargo fmt --check` and `cargo clippy --all-targets -D
  warnings` on Linux, Windows and macOS. Code compiled only on some platforms
  can look unused elsewhere: use a targeted `cfg_attr(..., allow(...))` instead
  of renaming parameters. Never rename `#[tauri::command]` parameters to silence
  lints; they are the IPC argument names.
- Edit `conf-templates/Cargo.template.toml` together with `src-tauri/Cargo.toml`;
  `npm run test:config` fails when the generated manifest drifts.
- When adding or changing a bridge command, update `desktopr-bridge.toml`
  (and `desktopr-bridge-companion` unless the command is companion-denied in
  `scripts/check-bridge-permissions.mjs`), then run `npm run
  check:bridge-permissions`.
- Never take a window identity or filesystem scope from frontend input; inject
  the calling `WebviewWindow` into the command instead.
- Keep the static `remote` capability limited to `main`; grant capabilities to
  runtime-created windows through `bridge/acl.rs`.
- Never add restricted `com.apple.developer.*` entitlements to
  `src-tauri/Entitlements.plist`: without a provisioning profile macOS refuses
  to launch the app. `check:standalone` enforces this.
- Default configuration must use the bundled standalone page, contain no remote
  origin and exclude updater configuration and permissions.
- Enabling updates requires both developer-owned updater inputs plus the Cargo
  `updater` feature. Never restore an implicit or hosted Desktopr updater.
- Run `syngraphe check` after updating repository context.
- Lint workflow changes with `actionlint` (with `shellcheck` on `PATH` so
  embedded scripts are checked).

## CI workflow rules

- Credentials are secrets only, never workflow inputs, and must not be echoed.
  Signing stays optional: absent secrets produce an unsigned build, partial
  secret sets fail early.
- Keep the default token read-only; grant `contents: write` only to the job
  that creates a GitHub Release.
- Pin actions to commit SHAs with the version in a comment (Dependabot keeps
  them updated) and pin downloaded tools with SHA-256 checks; do not add
  `curl | bash` installers or uploads to external storage.
- Keep the Cargo package name/version from `Cargo.lock` in CI; the app version
  belongs in `tauri.conf.json` so `--locked` stays valid.

## Important rules

- Never print or copy credential values found in source manifests or lockfiles.
- Do not modify `docs`, `companion`, repository visibility, or repository name
  during the core monorepo migration.
- Preserve the existing runtime surface until a standalone cross-platform build
  exists.

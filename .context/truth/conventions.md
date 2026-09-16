# Conventions

## Repository conventions

- The repository root owns the runtime and the npm workspace configuration.
- New JavaScript/TypeScript packages must be public-dependency-only workspaces.
- Run `npm run check:private-deps` after changing npm, Cargo or registry
  manifests. The check reports locations and categories without echoing secret
  values.
- Run `npm run check:standalone` after changing runtime defaults, capabilities,
  configuration generation or updater wiring.
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
- Minimum local verification for migration changes:
  `npm ci`, `npm run check:private-deps`, `npm run typecheck`,
  `npm run check:menu-contract`, `npm run ts:compile:bridge`, and
  `cargo check --locked` in `src-tauri/`.
- For WASM changes also run `npm run wasm:check`, build the affected module, and
  execute at least one request against the resulting artifact.
- Default configuration must use the bundled standalone page, contain no remote
  origin and exclude updater configuration and permissions.
- Enabling updates requires both developer-owned updater inputs plus the Cargo
  `updater` feature. Never restore an implicit or hosted Desktopr updater.
- Run `syngraphe check` after updating repository context.

## Important rules

- Never print or copy credential values found in source manifests or lockfiles.
- Do not modify `docs`, `companion`, repository visibility, or repository name
  during the core monorepo migration.
- Preserve the existing runtime surface until a standalone cross-platform build
  exists.

# Desktopr monorepo layout

## Context

The runtime and SDK already live in `tauri-skeleton`, while the source material
for Actions, shared types and WASM modules is spread across four repositories.
Moving the runtime into `packages/runtime` now would create path churn before a
standalone build has been established.

## Decision

Use `tauri-skeleton` as the monorepo root and keep the Rust/Tauri runtime in its
current root-level directories. Mark the root npm package private and begin with
`sdk/` as the only npm workspace.

Later phases may add `wasm/module-template`, `wasm/modules` and repository-local
workflows. `project-globals` will not be copied wholesale: only the generic
Tauri/menu definitions may be absorbed, without `suffro-lib` or SaaS types.

## Alternatives considered

- Move the runtime immediately to `packages/runtime`: rejected for now because
  it would require a broad path and build-script refactor.
- Copy all source repositories in the first pass: rejected because their
  hosted-service and private-dependency boundaries have not yet been removed.
- Keep independent repositories: rejected because the migration plan requires
  one technical source of truth.

## Consequences

- Existing runtime paths and build behavior remain stable.
- Root `npm ci` links the local `desktopr` SDK workspace.
- The legacy companion frontend remains outside the workspace until its planned
  final integration.
- WASM and Actions integration stay explicit later phases rather than being
  mixed into the initial consolidation.

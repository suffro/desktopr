# WASM workspace integration

## Decision

Keep Desktopr modules in a dedicated `wasm/` Cargo workspace that is also a
private npm workspace. The Rust workspace owns dependency versions, release
profile and lockfile; the small Node build helper owns artifact naming and
placement.

The initial members are:

- `wasm/module-template/`, absorbed from `wasm-module-template`;
- `wasm/modules/math/`, absorbed from `wasm-modules`.

Both modules target `wasm32-wasip1` and preserve the existing host protocol:
one `{ "fn", "args" }` JSON request on stdin and one JSON response on stdout.

## Rationale

- One Cargo workspace removes duplicated dependency and build configuration.
- Keeping the runtime in `src-tauri/` avoids an unrelated refactor.
- A repository-owned build helper replaces the source repositories' standalone
  npm packages and avoids their stale links and unused JavaScript runtime.
- Ignored `dist/` artifacts keep generated binaries out of source control.

## Consequences

- Developers need the `wasm32-wasip1` Rust target.
- New modules must be added to the Cargo workspace and the explicit build map.
- Package release version and bridge API version remain separate concepts; SDK
  builds do not rewrite package metadata.

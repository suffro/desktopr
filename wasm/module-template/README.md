# Desktopr module template

This crate is a minimal WASI Preview 1 Desktopr module. Add functions in
`src/functions.rs`, expose them from `src/dispatcher.rs`, then build it from the
repository root with `npm run wasm:build:template`.

The entrypoint owns the JSON/stdin/stdout protocol and enforces the runtime's
1 MiB input boundary. Keep business logic out of `main.rs` so a copied module
can retain the protocol unchanged.

Relative paths are scoped to the temporary job directory. Persist files only
under `/storage`, the dedicated directory preopened by the Desktopr host.

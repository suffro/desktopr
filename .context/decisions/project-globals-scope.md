# Project globals absorption scope

## Context

`project-globals` mixes reusable Tauri/menu definitions with Firebase,
Firestore, Stripe, hosted build-service constants and a private `suffro-lib`
dependency. The target already owns its runtime types, and the old
`BdTauriConfig` shape does not cover the current Tauri configuration.

## Decision

Absorb only the menu types and JSON Schema. Store them as a Desktopr-owned
contract in `src-ts/config/menu.ts`, align them with the Rust deserializer, use
the schema to validate public menu inputs before IPC, and export the contract
from the SDK.

Do not copy the `project-globals` package, root exports, environment constants,
Firestore types, SaaS dependencies, deprecated Tauri option types, or the
unused/incomplete `BdTauriConfig` skeleton.

## Alternatives considered

- Copy `src/tauri/` unchanged: rejected because it retains Bubbledesk naming,
  includes unused Tauri config types and has schema/runtime discrepancies.
- Copy the complete package and remove dependencies later: rejected because it
  would import private credentials and SaaS-only code into the target.
- Keep the existing handwritten menu validator only: rejected because it had
  already diverged from the Rust-required fields and supported enum values.

## Consequences

- The target has no build/runtime dependency on `project-globals`.
- Menu config has one TypeScript contract shared by bridge and SDK.
- Validation is stricter at the public boundary and matches successful Rust
  deserialization rather than accepting incomplete objects until IPC.
- Future Tauri config typing should be derived from actual target consumers or
  current Tauri schemas, not the discarded legacy shape.

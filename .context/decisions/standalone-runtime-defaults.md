# Standalone runtime defaults

## Decision

Desktopr's default dev and production configurations load the bundled
`src-tauri/standalone/index.html` page. `MAIN_WINDOW_URL`, bridge `appUrl` and
the capability's remote origins are empty unless a developer supplies
`APP_URL`.

The Tauri updater dependency remains available but is removed from Cargo's
default features. It is registered only with the explicit `updater` feature.
Configuration generation adds updater endpoint, public key, artifacts and
permissions only when both `UPDATE_ENDPOINT` and `ED25519_PUBKEY` are supplied.
Partial updater configuration is rejected.

## Rationale

- A base build must not contact or require retired Desktopr infrastructure.
- Developer-owned remote applications and updater services remain supported.
- Compile-time feature gating prevents an updater client from being included
  accidentally while preserving the existing optional capability.

## Verification

`npm run check:standalone` checks the generated defaults and scans the core
monorepo for hosted Desktopr/Bubbledesk URLs. It excludes `.context/`, `docs/`
and `frontend/`; the latter remains companion-phase work.

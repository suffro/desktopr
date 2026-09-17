# Desktopr Companion app

## Decision

The Desktopr Companion is a desktop app built with Desktopr for trying the
runtime. Its SvelteKit source lives in `apps/companion/` (npm workspace
`@desktopr/companion`) and replaces the legacy `frontend/` copy. It was imported
from the separate `companion` repository as current source only: no Git
history and no lockfile, because both contain a credentialed `suffro-lib`
dependency.

- The UI is bundled into the app (`APP_FRONTEND=companion` in
  `scripts/prod-conf.sh`, `frontend: companion` in `build.yml`), so it needs no
  hosted dashboard. Companion builds always use `COMPANION_MODE=true`.
- A URL entered by the user is loaded in the **main window** with the full
  bridge and every permission of the main capability. The companion exists so
  developers can try every feature; the reduced capability of runtime
  "companion windows" (`dtr_launch_companion`) is unrelated and was rejected
  here because it would hide features.
- Distribution is a GitHub Release only (`companion-release.yml`, tag
  `companion-v<version>`, notes in `apps/companion/RELEASE_NOTES.md`) plus the
  existing Microsoft Store listing, updated with the MSIX from the same run
  (identity `Desktopr.Desktopr`, publisher
  `CN=B27DF8BD-5700-47C4-9719-CA85B3F0938A`, display names `Desktopr`). The
  macOS build is signed and notarized; the release workflow refuses to run
  without the `APPLE_*` secrets. The Windows installer is unsigned and the
  release notes explain the SmartScreen prompt.
- The MSIX version is the app version plus `.0`. The Store listing was at
  `2.3.0.0` when the migration started, so the first release from this
  repository is companion `3.0.0` (MSIX `3.0.0.0`); every later release must
  increase it.
- The web landing page with download buttons, hosted links (dashboard,
  terms, privacy, docs) and `suffro-lib` were removed. Playground snippets use
  `example.com` instead of hosted Desktopr URLs.
- The playground types are generated from the `sdk/` workspace build, and
  `vite.config.ts` applies CommonJS interop to the linked SDK.

## Rejected

- Running the companion from source (`npm run companion` in dev mode) as the
  way to try Desktopr: it needs the full Rust/Tauri toolchain, which the
  intended users do not otherwise need.
- Keeping a hosted companion web page or download landing.
- Opening user URLs in reduced companion windows.
- Importing the source repository history.

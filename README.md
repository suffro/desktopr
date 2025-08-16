# Bubbledesk Tauri Wrapper Template

This repository is a template used by Bubbledesk's CI to build native wrappers for arbitrary web apps (by URL) on Windows, macOS and Linux. No customer repository is accessed.

## How it works
- CI fetches this template into a clean workspace.
- Replaces placeholders like `{{APP_NAME}}`, `{{APP_URL}}`, `{{APP_ORIGIN}}`, etc.
- Builds with Tauri v2 and optional plugins enabled via Cargo features.
- Uploads artifacts to the configured storage.

## Placeholders
- `{{APP_NAME}}`          – Product display name
- `{{APP_ID}}`            – Reverse-DNS identifier (e.g. `app.bubbledesk.desktop`)
- `{{APP_VERSION}}`       – Semver version
- `{{APP_URL}}`           – Full URL to load (e.g. `https://app.customer.tld/`)
- `{{APP_ORIGIN}}`        – Origin derived from APP_URL (e.g. `https://app.customer.tld`)
- `{{CREATE_UPDATER_ARTIFACTS}}` – `true|false`
- `{{UPDATER_PUBKEY}}`    – Updater public key (optional)
- `{{UPDATER_ENDPOINT}}`  – JSON endpoint for updates (optional)

## Notes
- Capabilities restrict which remote origins can use the Tauri IPC.
- `build.removeUnusedCommands = true` is enabled to shrink binaries.
- Plugins are gated by Cargo features to avoid compiling what you don't use.
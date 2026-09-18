---
title: Tauri Config
description: How Desktopr generates tauri.conf.json and the capability file, and how to change them safely.
---

# Tauri Config

Desktopr generates the Tauri configuration instead of keeping it hand-edited, so that the same repository can produce a development build, a production build for your web app and the Companion app.

## tauri.conf.json

`src-tauri/tauri.conf.json` is generated from `conf-templates/tauri.conf.template.prod.json` (or `.dev.json`). The generation step fills in:

- `productName`, `identifier` and `version` from your [app settings](/guide/configuration/app-settings);
- `build.frontendDist`, which points at the bundled local page, or at `apps/companion/dist` for the Companion;
- the Content Security Policy, derived from `APP_URL` (or permissive in companion mode);
- the deep link scheme, when you set one;
- the updater section, only when you configure both updater inputs.

Change the **template**, then regenerate:

```sh
npm run prodconf   # production
npm run devconf    # development
```

## Capabilities

`src-tauri/capabilities/remote.json` is the Tauri capability that grants the bridge to your web app. It is generated from `conf-templates/remote.template.json` and contains:

- `windows`/`webviews`: always just `main`. Windows created at runtime get their own capability from the Rust side.
- `permissions`: the `desktopr-bridge` permission set plus the Tauri plugin permissions the bridge needs.
- `remote.urls`: added only when you set `APP_URL` — the origin and its subdomains.

With no `APP_URL` there is no remote origin at all: the app only runs its bundled page.

## Permissions

The bridge commands are grouped in `src-tauri/permissions/desktopr-bridge.toml`:

| Permission set | Granted to |
| --- | --- |
| `desktopr-bridge` | the main window and windows opened with `Desktopr.window.new()` |
| `desktopr-bridge-companion` | companion (cache-only) windows: no window creation, no plugin install, no autostart or diagnostics changes |
| `desktopr-bridge-debug` | debug builds only, added by the dev configuration scripts |

If you add or change a bridge command, update that file and run `npm run check:bridge-permissions`, which verifies that TypeScript invocations, registered Rust commands and permissions all match.

## Entitlements (macOS)

`src-tauri/Entitlements.plist` holds the macOS entitlements. Do not add restricted `com.apple.developer.*` entitlements: without a provisioning profile macOS refuses to launch the app, and `npm run check:standalone` fails the build if one appears.

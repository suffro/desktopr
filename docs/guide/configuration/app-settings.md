---
title: App Settings
description: Every option you can set when building a Desktopr app - name, identifier, version, window, deep links and updates.
---

# App Settings

Desktopr apps are configured with environment variables read by `scripts/prod-conf.sh` (production) and `scripts/dev-conf.sh` (development), or with the matching inputs of the **build** workflow.

## Application

| Variable | Workflow input | Default | Description |
| --- | --- | --- | --- |
| `APP_URL` | `app_url` | *(empty)* | Absolute URL of your web app. Empty loads the bundled local page. |
| `APP_VERSION` | `app_version` | `0.2.1` | App version, `X.Y.Z`. |
| `APP_IDENTIFIER` | `app_identifier` | `app.desktopr.app` | Bundle identifier in reverse-domain form, e.g. `com.example.app`. |
| `MAIN_WINDOW_TITLE` | `app_name` | `Desktopr` | Product name and main window title. |
| `DEEPLINK_SCHEME` | `deeplink_scheme` | *(empty)* | Custom protocol scheme, e.g. `example` for `example://…`. |
| `APP_FRONTEND` | `frontend` | `standalone` | `standalone` bundles the local page (or loads `APP_URL`); `bundled` embeds your own built web app; `companion` bundles the [Companion](/guide/companion) app. |
| `APP_FRONTEND_DIST` | `frontend_dist` | *(empty)* | With `bundled`: directory of the built web app, relative to the repository root. Must contain `index.html`. |

Only the origin of `APP_URL` and its subdomains are granted access to the native bridge. Do not point `APP_URL` at a domain whose subdomains third parties control.

A build that shows a bundled frontend keeps `'unsafe-inline'` in `script-src` and tells Tauri not to rewrite that directive. Nonces and hashes there would also apply to the scripts that inject the bridge, and the app would start without `window.Desktopr`.

## Main window

| Variable | Default | Description |
| --- | --- | --- |
| `MAIN_WINDOW_WIDTH` | `1200` | Initial width in pixels. |
| `MAIN_WINDOW_HEIGHT` | `800` | Initial height in pixels. |
| `MAIN_WINDOW_BG_COLOR` | `#ffffff` | Background colour shown before the page paints. |
| `MAIN_WINDOW_RESIZABLE` | `true` | Whether the window can be resized. |
| `MAIN_WINDOW_OPEN_FULLSCREEN` | `false` | Open the window in fullscreen. |
| `MAIN_WINDOW_URL` | value of `APP_URL` | Override the URL of the main window only. |

Example:

```sh
APP_URL=https://app.example.com \
APP_VERSION=1.0.0 \
APP_IDENTIFIER=com.example.app \
MAIN_WINDOW_TITLE="Example" \
MAIN_WINDOW_WIDTH=1440 \
DEEPLINK_SCHEME=example \
npm run build
```

## Icons

Icons live in `src-tauri/icons/`. Generate a full set from one square source image with the Tauri CLI:

```sh
cd src-tauri && cargo tauri icon ../path/to/icon.svg
```

In the build workflow, pass the repository path of the source image as the `icon` input and the icons are generated for you.

## Updates

The updater is **off by default** and is not even compiled in. To enable it you need your own HTTPS update manifest and signing key:

- **GitHub Actions:** set the `update_endpoint` and `updater_public_key` inputs and the `TAURI_SIGNING_PRIVATE_KEY` secret.
- **Locally:** set `UPDATE_ENDPOINT` and `ED25519_PUBKEY` for `npm run prodconf`, then build with `cargo tauri build --features updater` and `TAURI_SIGNING_PRIVATE_KEY` in the environment.

Both values are required together, and they are yours: Desktopr never points your app at an update server it controls. See [Tauri's updater guide](https://v2.tauri.app/plugin/updater/) for the manifest format.

## Companion mode

`COMPANION_MODE=true` accepts bridge calls from any HTTPS origin and from localhost. It exists for the [Companion](/guide/companion) app and for development tooling — never ship a production app built this way.

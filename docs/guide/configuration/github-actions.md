---
title: Building with GitHub Actions
description: Build Desktopr installers in CI - from a web app URL or from a web app you build in your own repository.
---

# Building with GitHub Actions

There are two ways to build Desktopr apps in CI, and they differ only in where your code lives.

| Your situation | Use |
| --- | --- |
| You forked Desktopr, or your app is a URL | the **build workflow** |
| Your web app lives in its own repository | the **build action** |

Both produce the same installers, run the same smoke test and upload the same artifacts. Nothing is uploaded outside GitHub.

## The build action

`suffro/desktopr/.github/actions/build` builds a desktop app from the repository that calls it. You do not clone or vendor Desktopr: GitHub checks the runtime out together with the action.

The action builds for the runner it is running on, so a matrix gives you the three platforms:

```yaml
name: desktop

on: workflow_dispatch

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-22.04, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    timeout-minutes: 90
    steps:
      - uses: actions/checkout@v5

      # Build your web application however you normally do.
      - uses: actions/setup-node@v6
        with:
          node-version: 22
      - run: npm ci
      - run: npm run build

      - uses: suffro/desktopr/.github/actions/build@main
        with:
          app_name: Example
          app_identifier: com.example.app
          app_version: 1.0.0
          frontend_dist: dist
```

Each job uploads a `desktopr-<platform>` artifact containing the installers and a `SHA256SUMS-<platform>.txt` file.

Pin the action to a tag or commit rather than `@main` once you depend on it.

### Bundled app, or remote URL

The action builds from **one** of two sources:

- `frontend_dist: dist` — the directory your own build step produced. Its files are copied into the runtime and embedded in the app, which then runs entirely offline from `tauri://localhost`.
- `app_url: https://app.example.com` — no files are embedded; the app loads that URL, and only that origin and its subdomains are granted access to the native bridge.

Setting both is an error. With neither, the app ships the bundled placeholder page.

::: warning
Embedded files are privileged: they run with the app's own origin and can call every bridge API the app is allowed to use. Only bundle a build you produce yourself.
:::

### Routing in a bundled app

The embedded files are served from the root of the directory you pass, so absolute paths like `/assets/app.js` resolve normally and no special `base` is needed.

What does not work is asking the asset server for a route that is not a file: opening the app directly at `/settings` fails unless your build wrote a `settings.html`. Use hash routing, or make your router start from `/` and navigate in the client.

`index.html` must exist in `frontend_dist`; the action fails early if it does not.

## Inputs

### Application

| Input | Default | Description |
| --- | --- | --- |
| `app_name` | `Desktopr` | Product name and main window title. |
| `app_identifier` | `app.desktopr.app` | Bundle identifier, reverse-domain. |
| `app_version` | *(runtime default)* | `X.Y.Z`, optionally with a suffix. |
| `frontend_dist` | *(empty)* | Directory of your built web app, relative to your checkout. |
| `app_url` | *(empty)* | External web app URL. |
| `icon` | *(runtime icon)* | Square source icon (PNG or SVG) in your checkout. |
| `deeplink_scheme` | *(empty)* | Custom protocol scheme, e.g. `example`. |

Every other application setting is documented in [App Settings](/guide/configuration/app-settings).

### Build behaviour

| Input | Default | Description |
| --- | --- | --- |
| `smoke_test` | `true` | Launch the built app for 15 seconds to prove it starts. |
| `upload_artifact` | `true` | Upload the packages as a workflow artifact. |
| `artifact_name` | `desktopr-<platform>` | Name of that artifact. |
| `node_version` | `22` | Node.js used for the bridge and your frontend build. |
| `tauri_cli_version` | `2.11.1` | Tauri CLI version. |

Outputs: `platform`, `dist_dir` (where the packages are), `artifact_name` and `updater`.

### Signing

Signing is optional and uses only your repository's own secrets. A platform whose inputs are absent is built unsigned; macOS is then ad-hoc signed. Half a set fails the build early.

| Input | For |
| --- | --- |
| `windows_certificate`, `windows_certificate_password` | Windows signing |
| `apple_certificate`, `apple_certificate_password`, `apple_signing_identity` | macOS signing |
| `apple_id`, `apple_password`, `apple_team_id` | macOS notarization |
| `tauri_signing_private_key`, `tauri_signing_private_key_password` | updater artifacts |

```yaml
      - uses: suffro/desktopr/.github/actions/build@main
        with:
          app_name: Example
          frontend_dist: dist
          apple_certificate: ${{ secrets.APPLE_CERTIFICATE }}
          apple_certificate_password: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          apple_signing_identity: ${{ secrets.APPLE_SIGNING_IDENTITY }}
```

See [Signing](/guide/signing/what) for how to obtain those certificates.

### Updater and Microsoft Store

`update_endpoint` and `updater_public_key` enable the updater; both are required, along with `tauri_signing_private_key`. The four `msix_*` identity inputs from Partner Center add an MSIX package to the Windows build. See [Distribution](/guide/configuration/distribution).

## The build workflow

If you forked Desktopr, the repository's own **build** workflow already wraps the action and adds the platform matrix and the GitHub Release job. Run it from the Actions tab, or call it:

```yaml
jobs:
  desktop:
    uses: suffro/desktopr/.github/workflows/build.yml@main
    with:
      platforms: linux,windows,macos
      app_name: Example
      app_identifier: com.example.app
      app_version: 1.0.0
      app_url: https://app.example.com
    secrets: inherit
```

Inside a fork you can also bundle a web app that is committed in the checkout, with `frontend: bundled` and `frontend_dist: <directory>`. The workflow builds nothing itself, so the directory has to be there already — if your web app needs a build step, use the action instead.

## Build times

The first run compiles the whole Rust runtime: roughly 20 minutes on Linux and macOS, longer on Windows. Later runs reuse the Cargo cache and are much faster. The cache key includes the checkout path, because Tauri records absolute paths in its build artifacts.

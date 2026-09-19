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

`suffro/desktopr` builds a desktop app from the repository that calls it. You do not clone or vendor Desktopr: GitHub checks the runtime out together with the action.

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

      - uses: suffro/desktopr@v1
        with:
          app_name: Example
          app_identifier: com.example.app
          app_version: 1.0.0
          frontend_dist: dist
```

Each job uploads a `desktopr-<platform>` artifact containing the installers and a `SHA256SUMS-<platform>.txt` file.

`v1` moves forward with every compatible change, so you get fixes without editing your workflow. Pin a commit SHA instead if you want a build that never changes underneath you — the tag carries the Desktopr runtime as well as the action.

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

### Signing credentials

| Input | For |
| --- | --- |
| `sign` | set to `false` to build without signing even when credentials are available |
| `windows_certificate`, `windows_certificate_password` | Windows signing |
| `apple_certificate`, `apple_certificate_password`, `apple_signing_identity` | macOS signing |
| `apple_id`, `apple_password`, `apple_team_id` | macOS notarization |
| `tauri_signing_private_key`, `tauri_signing_private_key_password` | updater artifacts |

How signing behaves is explained below.

### Updater and Microsoft Store

`update_endpoint` and `updater_public_key` enable the updater; both are required, along with `tauri_signing_private_key`. The four `msix_*` identity inputs from Partner Center add an MSIX package to the Windows build. See [Distribution](/guide/configuration/distribution).

## Signing

Signing is what tells macOS and Windows who made your app. Without it, people who download it get a warning telling them it comes from an unknown developer — see [Signing](/guide/signing/what) for what that looks like and how to get the certificates.

The build action signs while it builds. You do not switch it on: you hand it your certificates, and it uses the ones it gets.

| What you give it | macOS | Windows |
| --- | --- | --- |
| The three `apple_*` values (and notarization) | signed and notarized | not signed |
| The two `windows_*` values | basic signature | signed |
| Both | signed and notarized | signed |
| Neither | basic signature | not signed |

So you can sign one platform and not the other simply by giving one set of credentials and not the other. Nothing else to configure.

Two things worth knowing:

- **On macOS there is no "no signature".** An app without any signature is killed the moment it opens, so a macOS build without your certificate still gets a basic one (called *ad-hoc*). The app runs, but people still see the unknown-developer warning.
- **Half a set stops the build.** If you pass two of the three Apple values, the build fails immediately and tells you what is missing, instead of quietly handing you an app you thought was signed.

If your workflow hands every secret to the action automatically (`secrets: inherit`) and you just want a quick test build, set `sign: false` and nothing will be signed.

## Signing an app that is already built

Sometimes the app was built elsewhere: in an earlier job, on your own machine, or in a release you are re-publishing. The **sign action** takes finished artifacts and signs them, without building anything.

```yaml
- uses: suffro/desktopr/.github/actions/sign@v1
  with:
    artifacts: out/Example.app
    apple_certificate: ${{ secrets.APPLE_CERTIFICATE }}
    apple_certificate_password: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
    apple_signing_identity: ${{ secrets.APPLE_SIGNING_IDENTITY }}
    apple_id: ${{ secrets.APPLE_ID }}
    apple_password: ${{ secrets.APPLE_PASSWORD }}
    apple_team_id: ${{ secrets.APPLE_TEAM_ID }}
```

Point `artifacts` at a file or at a folder, and it handles what it finds:

- **a macOS `.app`** — signs it, wraps it in a `.dmg` ready to publish, sends that to Apple to be checked, and attaches the result to the file so the app opens even on a computer that is offline;
- **a macOS `.dmg`** — signs and checks it as it is;
- **a Windows `.exe`, `.msi` or `.msix`** — signs it and verifies the signature afterwards.

It signs what the machine it runs on can sign: macOS artifacts on a macOS runner, Windows artifacts on a Windows runner. On Linux it does nothing and reports so, which means you can leave it in a three-platform matrix without special cases.

::: warning
Do not build unsigned and then sign the installer afterwards. A `.dmg` is a container: signing it does not sign the app inside, and an installer built for Windows already has the program embedded in it. Apple's check will reject the result, and Windows will still warn about the program itself. If you build with this project, give the build action your certificates and let it sign while it packages. The sign action is for a finished app that was signed nowhere yet — typically a `.app` you hand it directly.
:::

### Only having it checked by Apple

If you already signed the app yourself and only need Apple's check (notarization), leave the certificate out and pass just the three notarization values:

```yaml
- uses: suffro/desktopr/.github/actions/sign@v1
  with:
    artifacts: out/Example.dmg
    apple_id: ${{ secrets.APPLE_ID }}
    apple_password: ${{ secrets.APPLE_PASSWORD }}
    apple_team_id: ${{ secrets.APPLE_TEAM_ID }}
```

The file has to be signed already, otherwise Apple rejects it.

### Its inputs

| Input | What it is |
| --- | --- |
| `artifacts` | what to sign: a file or a folder, relative to your checkout |
| `apple_certificate`, `apple_certificate_password`, `apple_signing_identity` | your macOS certificate; leave out to only have the app checked by Apple |
| `apple_id`, `apple_password`, `apple_team_id` | the Apple account used for that check; all three or none |
| `macos_entitlements` | an entitlements file, if your app needs one |
| `windows_certificate`, `windows_certificate_password` | your Windows certificate |
| `timestamp_url` | the time-stamping service, so signatures keep working after the certificate expires |

::: tip
An `.msix` is signed only if your certificate matches the publisher recorded inside the package. If they differ, the action leaves the file alone and says so, because Windows refuses to install a package signed by anyone else. Packages you submit to the Microsoft Store are signed by the Store itself, so you do not need to sign those at all.
:::

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

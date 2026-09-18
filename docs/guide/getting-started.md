---
title: Getting Started
description: Build your first desktop app with Desktopr - clone the repository, point it at your web app and build for macOS, Windows and Linux.
head:
  - - meta
    - property: og:title
      content: "Getting Started with Desktopr"
  - - meta
    - property: og:description
      content: "Step-by-step guide to turning your web app into a native desktop app with Desktopr. Open source, no account, no hosted service."
  - - meta
    - property: og:image
      content: "https://desktopr.dev/assets/logo/logo-color.png"
  - - meta
    - name: twitter:title
      content: "Getting Started with Desktopr"
  - - meta
    - name: twitter:description
      content: "Step-by-step guide to turning your web app into a native desktop app with Desktopr. Open source, no account, no hosted service."
---


# Getting Started with Desktopr

> Easily build and sign lightweight desktop applications using web stacks, with Desktopr GitHub actions and a straightforward Node API.

**Desktopr** turns your web application into a cross-platform desktop application for macOS, Windows and Linux, using Rust and Tauri to produce lightweight, native apps.

Desktopr is free and open source (Apache 2.0). You build it yourself, on your machine or with GitHub Actions, and the resulting installers are yours. There is no account, backend or hosted service.

For more info check out: [What is Desktopr?](/guide/)

## Requirements

- Node.js 22 and npm
- Rust stable (`rustup`)
- Tauri CLI 2.11: `cargo install tauri-cli --version 2.11.1 --locked`
- `jq`
- The platform prerequisites from the [Tauri guide](https://v2.tauri.app/start/prerequisites/). On Debian/Ubuntu:
  `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libxdo-dev build-essential`

## Creating Your First Project

1. Fork or clone the repository:

   ```sh
   git clone https://github.com/suffro/desktopr.git
   cd desktopr
   npm ci
   ```

2. Run it against your web app in development:

   ```sh
   APP_URL=http://localhost:5173 npm run dev
   ```

   Without `APP_URL` the window shows the bundled local page in `src-tauri/standalone/`.
   Only the origin of `APP_URL` is granted access to the native bridge.

3. Install the SDK in your web app (`npm install desktopr`) to call native features. See the [Bridge API](/guide/bridge/overview).

4. Build the installers:

   ```sh
   APP_URL=https://app.example.com \
   APP_VERSION=1.0.0 \
   APP_IDENTIFIER=com.example.app \
   MAIN_WINDOW_TITLE="Example" \
   npm run build
   ```

   Bundles are written to `src-tauri/target/release/bundle/`. Other options (window size, deep link scheme, updater) are environment variables read by `scripts/prod-conf.sh`.

::: tip
Try the bridge before wiring your own app: the [Desktopr Companion](/guide/companion) loads any web app URL in a Desktopr window and includes a playground for the API.
:::

## Building with GitHub Actions

The repository ships a **build** workflow that builds the platforms you select, launches each app briefly as a smoke test and uploads the installers as workflow artifacts. Run it from the Actions tab of your fork, or call it from another workflow:

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

Set `release: true` (with `app_version`) to also create a draft GitHub Release. Nothing is uploaded anywhere else.

Signing is optional and uses only your own repository secrets; without them the build is unsigned. See [Signing](/guide/signing/what).

## Building for Platforms

Desktopr supports building your app for multiple platforms with the following output formats:

| Platform | Output Format                       |
| -------- | ----------------------------------- |
| macOS    | `.dmg`                              |
| Windows  | `.exe` (NSIS), optional `.msix`     |
| Linux    | `.AppImage`, `.deb`, `.rpm`         |

## Notes

:::tip
Explore the documentation for the Bridge modules to extend your app’s functionality and integrate native features seamlessly.
:::

:::warning
The first build takes a while because Rust compiles the whole runtime. Later builds reuse the `src-tauri/target` cache and are much faster.
:::

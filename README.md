# Desktopr

Desktopr turns a web application into a native desktop app for Linux, Windows
and macOS. It is built on [Tauri 2](https://v2.tauri.app/) and gives your web
code a typed JavaScript bridge to native features: files and dialogs, windows,
menus and context menus, tray, clipboard, notifications, global shortcuts,
deep links, autostart, diagnostics, networking and sandboxed WebAssembly
plugins.

Desktopr is self-contained. It needs no account, backend or hosted service:
you build it from this repository, on your machine or with GitHub Actions, and
you ship the result yourself.

## Repository layout

| Path | Contents |
| --- | --- |
| `src-tauri/` | Rust/Tauri runtime and native commands |
| `src-ts/` | TypeScript bridge injected into the app, and the SDK source |
| `sdk/` | The `desktopr` npm package for your web application ([README](sdk/README.md)) |
| `wasm/` | WASI plugin template and example modules ([README](wasm/README.md)) |
| `conf-templates/`, `scripts/` | Configuration generation for dev and production builds |
| `.github/workflows/build.yml` | Cross-platform build, optional signing and releases |
| `.github/workflows/companion-release.yml` | Signed Desktopr Companion release |
| `apps/companion/` | Desktopr Companion, a desktop app for trying the runtime and the bridge API |

## Requirements

- Node.js 22 and npm
- Rust stable (`rustup`), plus `rustup target add wasm32-wasip1` for WASM plugins
- Tauri CLI 2.11: `cargo install tauri-cli --version 2.11.1 --locked`
- `jq`
- Platform prerequisites from the
  [Tauri guide](https://v2.tauri.app/start/prerequisites/). On Debian/Ubuntu:
  `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libxdo-dev build-essential`

## Development

```sh
npm ci
npm run dev
```

`npm run dev` bundles the bridge, generates the dev configuration and starts
`cargo tauri dev`. By default the window shows the bundled page in
`src-tauri/standalone/`. To wrap your own web app, set its URL:

```sh
APP_URL=http://localhost:5173 npm run dev
```

Only the origin of `APP_URL` is granted access to the native bridge.

## Production builds

### Locally

```sh
APP_URL=https://app.example.com \
APP_VERSION=1.0.0 \
APP_IDENTIFIER=com.example.app \
MAIN_WINDOW_TITLE="Example" \
npm run build
```

Bundles are written to `src-tauri/target/release/bundle/`. Other options
(window size, deep link scheme, updater) are read from environment variables in
[`scripts/prod-conf.sh`](scripts/prod-conf.sh).

### With GitHub Actions

Run the **build** workflow from the Actions tab, or call it from another
workflow with `uses: <owner>/<repo>/.github/workflows/build.yml@<ref>`. It
builds the selected platforms, launches each app briefly as a smoke test, and
uploads the installers as workflow artifacts:

| Platform | Outputs |
| --- | --- |
| Linux | AppImage, `.deb`, `.rpm` |
| Windows | NSIS installer, optional MSIX |
| macOS | DMG |

Set `release: true` (with `app_version`) to also create a draft GitHub Release.
Nothing is uploaded anywhere else.

Signing is optional and uses only your repository secrets. Without them the
build is unsigned; if a set is incomplete, the workflow fails before building.

| Purpose | Secrets |
| --- | --- |
| Windows code signing | `WINDOWS_CERTIFICATE` (base64 `.pfx`), `WINDOWS_CERTIFICATE_PASSWORD` |
| macOS code signing | `APPLE_CERTIFICATE` (base64 `.p12`), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` |
| macOS notarization | `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), `APPLE_TEAM_ID` |
| Updater signatures | `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |

For the Microsoft Store or sideloading, provide the four `msix_*` identity
inputs from Partner Center to add an MSIX package to the Windows build.

#### Unsigned macOS builds

Without Apple certificates the app is ad-hoc signed and not notarized, so
Gatekeeper blocks it on first launch. Users can open **System Settings →
Privacy & Security** and choose **Open Anyway**, or run:

```sh
xattr -dr com.apple.quarantine "/Applications/Your App.app"
```

## Updates

The updater is off by default and is not compiled in. To enable it, host your
own update manifest and provide both an HTTPS update endpoint and the updater
public key; the configuration step then adds the updater config, artifacts and
permissions.

- **GitHub Actions:** set the `update_endpoint` and `updater_public_key` inputs
  and the `TAURI_SIGNING_PRIVATE_KEY` secret. The workflow builds with the
  `updater` Cargo feature.
- **Locally:** set `UPDATE_ENDPOINT` and `ED25519_PUBKEY` for `npm run prodconf`,
  then build with `cargo tauri build --features updater` from `src-tauri/`, with
  `TAURI_SIGNING_PRIVATE_KEY` in the environment.

See [Tauri's updater guide](https://v2.tauri.app/plugin/updater/) for generating
keys and the manifest format.

## Desktopr Companion

`apps/companion/` is a SvelteKit app bundled into a Desktopr build with
`frontend=companion` (`APP_FRONTEND=companion` for `npm run prodconf`). It loads
any web app URL in the main window with the full bridge, and includes a bridge
playground. Releases are published by `companion-release.yml` with the notes in
[`apps/companion/RELEASE_NOTES.md`](apps/companion/RELEASE_NOTES.md), which
include the first-launch steps for each platform.

## Checks

`.github/workflows/ci.yml` runs these on every push to `main` and every pull
request (Rust on Linux, Windows and macOS):

```sh
npm run typecheck             # runtime and SDK TypeScript
npm run check:standalone      # no hosted services, safe defaults, launchable entitlements
npm run check:private-deps    # no private packages or credentials in manifests
npm run check:menu-contract   # SDK menu schema matches the runtime
npm run check:bridge-permissions  # bridge commands are registered and allowed; companion subset is intact
npm run test:bridge           # bridge initialization, IPC argument names and SDK fallback
npm run test:config           # generated configuration and capabilities for each scenario
npm run companion:check       # companion svelte-check
npm run lint:rust             # rustfmt and clippy with warnings as errors
npm run test:rust             # Rust tests, including WASM modules run in the runtime host
```

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md). Please report vulnerabilities privately
as described in [SECURITY.md](SECURITY.md).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

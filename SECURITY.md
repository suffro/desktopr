# Security Policy

## Reporting a vulnerability

Please do not open a public issue for security problems.

Report vulnerabilities privately through GitHub: open the repository's
**Security** tab and choose **Report a vulnerability**. Include:

- the affected component (runtime, bridge/SDK, WASM host, build workflow);
- the platform and version or commit;
- steps to reproduce or a proof of concept;
- the impact you expect.

You should receive an acknowledgement within a few days. Once a fix is ready,
it will be released and credited to you unless you prefer to stay anonymous.

## Supported versions

Security fixes are made on the latest release and the `main` branch.

## Scope and trust model

Desktopr runs web content that the application developer chooses and exposes
native capabilities to it. When reviewing or reporting an issue, keep in mind:

- **Trusted:** the bundled local page and the developer-configured application
  origin (`APP_URL`) including its subdomains, so do not point `APP_URL` at a
  domain whose subdomains third parties control. Desktopr runs the developer's web app, so these can call
  the native bridge by design, including the low-level `Desktopr.tauri` API that
  reaches the Tauri plugins granted by the app's capability.
- **Untrusted:** any other origin, content loaded by the application from third
  parties, and WASM plugin modules. Plugins run in a WASI sandbox without
  network access, with execution time and I/O limits, and can only reach their
  temporary job directory and their own `/storage` directory.
- **Window identity** comes from the runtime, not from the web page. The static
  capability covers only the `main` window; windows opened later receive their
  own capability for their label. Filesystem commands derive their scope from
  the calling window and reject a mismatching `windowLabel`.
- **Companion (cache-only) windows** get a reduced bridge: they cannot open
  windows or companions, install or remove WASM modules, clear plugin storage,
  change autostart, or change and clear diagnostics. Their filesystem access is
  confined to their own scope, without access to plugin storage.
- **Companion mode** (`COMPANION_MODE=true`) intentionally accepts bridge calls
  from any HTTPS origin and from localhost. It is used by the Desktopr Companion
  app (`APP_FRONTEND=companion`), which loads a URL the user enters in the main
  window with the full bridge so every feature can be tried. Only load trusted
  web apps in the companion, and do not ship production apps built in this
  mode.
- **Network diagnostics** (`network.ping`, `network.estimateBandwidth`, the network monitor)
  send native HTTP(S) requests, outside the webview's CORS rules, to URLs the
  app chooses. Local and LAN hosts are reachable on purpose; requests are
  limited to HTTP(S) with bounded timeouts, download size and polling rate.

Examples of in-scope issues: bypassing filesystem scopes or path validation,
escaping the plugin sandbox, a companion window reaching commands or data
outside its scope, granting the bridge to an unconfigured origin, or signing
credentials leaking from the build workflow.

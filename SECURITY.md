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
  origin (`APP_URL`). These can call the native bridge by design.
- **Untrusted:** any other origin, content loaded by the application from third
  parties, and WASM plugin modules. Plugins run in a WASI sandbox without
  network access, with execution time and I/O limits, and can only reach their
  temporary job directory and their own `/storage` directory.
- **Companion mode** (`COMPANION_MODE=true`) intentionally accepts bridge calls
  from any HTTPS origin and is meant for development tooling only. Do not ship
  production apps built in this mode.

Examples of in-scope issues: bypassing filesystem scopes or path validation,
escaping the plugin sandbox, granting the bridge to an unconfigured origin, or
signing credentials leaking from the build workflow.

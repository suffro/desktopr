# In-repository GitHub Actions build

## Decision

`.github/workflows/build.yml` replaces the legacy `Bubbledesk/github-actions`
`build.yml`, `sign.yml` and `dist.yml`. It is a single workflow that supports
`workflow_dispatch` and `workflow_call` and runs:

1. `prepare`: validates inputs, the updater input pair and the platform list.
2. `build`: one matrix job per platform (`ubuntu-22.04`, `windows-latest`,
   `macos-latest`) that builds the monorepo checkout directly, signs when the
   platform's secrets are present and uploads a `desktopr-<platform>` artifact
   with a per-platform SHA-256 list.
3. `release`: only when `release` is true, creates a (draft by default) GitHub
   Release from the artifacts. Only this job has `contents: write`.

The steps of the `build` job were later moved into the composite action
the action at the repository root, which the job now calls; see
`external-build-action.md`. Everything below still describes what that action
does, and the workflow remains the entry point for this repository.

Signing is performed during `tauri build` with Tauri's native mechanisms instead
of a separate unzip/re-sign workflow:

- Windows: the PFX from `WINDOWS_CERTIFICATE` is imported into the runner's
  certificate store and `bundle.windows.certificateThumbprint` is set, so Tauri
  signs the executable and the NSIS installer.
- macOS: Tauri reads `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
  `APPLE_SIGNING_IDENTITY` and, for notarization, `APPLE_ID`, `APPLE_PASSWORD`,
  `APPLE_TEAM_ID`. Without signing secrets the build is ad-hoc signed
  (`signingIdentity: "-"`) and not notarized.
- Updater artifacts are signed with `TAURI_SIGNING_PRIVATE_KEY` only when the
  updater is enabled.

Every credential is a secret of the repository that runs or calls the workflow.
Partial secret sets fail before building. Empty secrets are unset before
`tauri build` because Tauri treats a present-but-empty variable as configured.

The workflow keeps the Cargo package name/version recorded in `Cargo.lock` and
applies the app version only to `tauri.conf.json`, so `--locked` builds accept
any app version. Icons are generated in CI from a repository path with
`cargo tauri icon`. The production template bundles `dmg` so macOS output is a
downloadable image rather than a bare `.app` directory.

The Windows job can also produce an MSIX package (Microsoft Store or
sideloading). It is opt-in: the four `msix_*` identity inputs from Partner
Center must all be provided, otherwise the step is skipped; partial values, a
publisher not starting with `CN=` or a non-numeric version fail in `prepare`.
Unlike the legacy step, a requested MSIX that fails to build fails the job.
The package contains only the main executable (plus any top-level DLLs), since
the runtime bundles no resources, and Store assets are drawn from the generated
icon without distortion. It is signed with the imported Windows certificate
only when the certificate subject equals `msix_identity_publisher`; otherwise it
is left unsigned, as Store submissions are signed by the Store.

After building, each platform launches the app for 15 seconds (the macOS
`.app` bundle via `open`, the Linux binary under `xvfb-run`, the Windows
executable) and fails if it exits. A green build alone missed unsigned macOS
apps being killed at launch because of a restricted entitlement.

## Rationale

- The legacy flow depended on a private wrapper checkout token, R2/CDN uploads,
  a hosted `latest.json`, a hosted logging worker and credentials passed as
  plain workflow inputs. None of these may survive in Desktopr OSS.
- Tauri's native signing signs the inner binary and the installer in one pass;
  re-signing an unzipped bundle afterwards would leave the installer payload
  unsigned and duplicates bundling logic.
- Only first-party actions (`actions/*`) are used; the Rust toolchain comes
  from the runner's `rustup` and the Tauri CLI is pinned and installed with
  `cargo install --locked`.

## Not carried over

- R2/CDN uploads, previous-build copies, pruning and hosted `latest.json`
  merging (`dist.yml` and the distribution half of `sign.yml`).
- Icon download from a hosted URL, `bundle.map.json`, the hosted failure
  logging worker and the private wrapper checkout.
- Generating an updater `latest.json` for GitHub Releases. Developers who enable
  the updater currently host their own manifest.

## Verification

Workflow syntax and embedded shell scripts are checked with `actionlint` plus
`shellcheck`; PowerShell steps are parsed with `pwsh`, and the rendered MSIX
manifest was checked as well-formed XML with escaped values. The unsigned
Linux, Windows (including MSIX) and macOS jobs succeeded on GitHub. Signing with
real certificates, notarization, the updater path and the release job still need
a run with the corresponding secrets or inputs.

# Composite build action for external repositories

## Decision

The per-platform build lives in the composite action `action.yml` at the
repository root. `.github/workflows/build.yml` keeps only input validation, the
platform matrix and the release job, and calls the action once per runner with
`uses: ./`. There is one implementation of the build, used both inside this
repository and by other repositories.

The metadata file sits at the root because GitHub Marketplace only lists an
action whose `action.yml` is there, and because it shortens the reference to
`suffro/desktopr@v1`.

A workflow in any repository can build a desktop app from its own web
application:

```yaml
- uses: actions/checkout@v5
- run: npm ci && npm run build
- uses: suffro/desktopr@v1
  with:
    app_name: Example
    frontend_dist: dist
```

GitHub checks out the whole action repository, so the action carries the
runtime with it. The runtime root is `$GITHUB_ACTION_PATH` itself: the workspace
for a local `uses: ./`, and `_actions/<owner>/<repo>/<ref>` for a
cross-repository checkout. Caller-supplied paths (`frontend_dist`, `icon`) are resolved against
`$GITHUB_WORKSPACE` instead, and the caller's checkout is never modified.

`scripts/prod-conf.sh` gained the `bundled` frontend mode: with
`APP_FRONTEND=bundled` and `APP_FRONTEND_DIST=<relative directory>` it points
`build.frontendDist` at that directory, exactly as the `companion` mode points
at the companion build. The action stages the caller's directory into
`<runtime>/frontend-dist` first, because Tauri resolves `frontendDist` relative
to the directory holding `tauri.conf.json` and the caller's workspace is not
under it.

`bundled` grants no remote origin and keeps the `'self'` CSP: the app serves its
own files through the asset protocol. `bundled` and `app_url` are mutually
exclusive, and the directory must contain a non-empty `index.html`.

## Signing is split in two, along the line the tools impose

The build action signs while it packages, because that is the only correct
order: an NSIS installer embeds the executable, and a DMG must be built from an
already-signed `.app`. Signing is per platform and driven only by which
credentials the caller passes; `sign: false` overrides them, for callers that
hand over every secret with `secrets: inherit`. A macOS build without a
certificate still gets an ad-hoc signature, since an unsigned app is killed at
launch.

`.github/actions/sign/` is a second, standalone action for artifacts that were
built elsewhere. It shares no implementation with the build path: there Tauri
performs the signing from environment variables, while here the action imports
the certificate into a throwaway keychain and drives `codesign`, `hdiutil`,
`notarytool`, `stapler` and `signtool` itself. Given a `.app` it signs the
nested binaries before the bundle, wraps it in a DMG and notarizes that;
given a finished `.dmg`, `.exe`, `.msi` or `.msix` it signs it in place. With
only the three notarization values and no certificate it notarizes an artifact
that is already signed. An MSIX is signed only when the certificate subject
equals the publisher inside the package.

It is a separate action, not a `mode` input on the build action, because the two
paths share no code: one file would hold two disjoint halves plus a condition on
every build step. A composite action can call a sibling action in its own
repository at the same commit with `uses: $/...`, but that does not help here —
build-and-sign cannot be composed, since Tauri signs during bundling.

Only the root action can be listed in GitHub Marketplace; the sign action stays
usable by path.

## Rationale

- A reusable workflow cannot do this. In a `workflow_call` the default
  `actions/checkout` checks out the *caller's* repository, so `build.yml` would
  need a second checkout and a path rewrite of every step, while the caller
  would still need a way to inject its build output. A composite action gets the
  runtime for free and runs inside the caller's job, next to its build steps.
- Copying the build steps into an action and leaving `build.yml` untouched would
  create two implementations that drift. Extracting them keeps signing, caching,
  the smoke test and artifact collection identical for every consumer.
- Secrets are action inputs because composite actions cannot read the `secrets`
  context. The caller passes its own secrets explicitly; the action never sees a
  secret the caller did not hand it.

## Constraints that shaped the implementation

- Composite actions have no top-level `env` and no `defaults.run.shell`, so
  every step declares its own shell, and `NODE_VERSION`/`TAURI_CLI_VERSION`
  became inputs.
- `hashFiles()` only sees the workspace, so the Cargo and npm cache keys are
  hashed in shell from the runtime's lockfiles.
- `actions/*` steps need native paths on Windows while bash steps need POSIX
  ones, so the context step publishes both.
- The Cargo cache key includes a hash of the runtime path: Tauri records
  absolute paths in its build artifacts, and a cache saved from a different
  checkout path breaks the build.

## Rejected

- **Downloading the caller's build from a URL or artifact.** It needs either a
  cross-repository token or a public tarball plus a checksum, and it separates
  the build of the web app from the build of the desktop app in time.
- **Committing the web app build into a fork.** Still supported through
  `frontend: bundled` in `build.yml`, but it requires the output to be in the
  checkout, which does not fit a normal frontend build pipeline.

## Verification

`actionlint` reports no new finding on the workflows (the pre-existing
`workflow_dispatch` input-count warning is unchanged), `shellcheck` is clean on
the action's bash steps and on `prod-conf.sh`, and the action's YAML, input
names and output step references were cross-checked against `build.yml`. The
runtime-root arithmetic was checked against both the local and the simulated
cross-repository action path. `npm run test:config` covers the `bundled` mode,
including that `frontendDist` resolves to the staged `index.html` from
`src-tauri/`, and each new guard was observed failing.

On GitHub, run 35383004743 built through the action on Linux, Windows and macOS
with the launch smoke test green on each, and the macOS app was Developer ID
signed and notarized (`Accepted`), which exercises passing secrets as action
inputs. Run 35383013081 exercised `frontend=bundled`: the caller's directory was
staged, `frontendDist` pointed at it and the generated capability granted no
remote origin.

Not yet verified: a real cross-repository run. Both runs used the local `./`
path; the `_actions/<owner>/<repo>/<ref>` layout was only simulated locally.

`.github/workflows/sign-action-test.yml` verifies the signing work on demand
and passed on runs 35412477195 and 35412958040: a fixture `.app` came out
Developer ID signed, wrapped in a DMG, notarized (`accepted`) and stapled; a
build given every Apple credential but `sign: false` produced an app that is
ad-hoc signed, checked by mounting the DMG; the action reports and skips on
Linux. Windows needs no secret, since the job generates a self-signed
certificate, trusts it on the runner and checks that an executable and a
package whose publisher matches are signed by it while a package with another
publisher is left `NotSigned`. Signing with a real Windows certificate remains
unverified, because the repository has none.

# Composite build action for external repositories

## Decision

The per-platform build lives in the composite action
`.github/actions/build/action.yml`. `.github/workflows/build.yml` keeps only
input validation, the platform matrix and the release job, and calls the action
once per runner with `uses: ./.github/actions/build`. There is one
implementation of the build, used both inside this repository and by other
repositories.

A workflow in any repository can build a desktop app from its own web
application:

```yaml
- uses: actions/checkout@v5
- run: npm ci && npm run build
- uses: suffro/desktopr/.github/actions/build@v1
  with:
    app_name: Example
    frontend_dist: dist
```

GitHub checks out the whole action repository, so the action carries the
runtime with it. It resolves the runtime root from `$GITHUB_ACTION_PATH/../../..`,
which is the repository root both for a local `./.github/actions/build` and for
a cross-repository `_actions/<owner>/<repo>/<ref>/.github/actions/build`
checkout. Caller-supplied paths (`frontend_dist`, `icon`) are resolved against
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

Not yet verified: a real cross-repository run. Both runs used the local
`./.github/actions/build` path; the `_actions/<owner>/<repo>/<ref>` layout was
only simulated locally.

# Documentation site

## Decision

The VitePress documentation lives in `docs/` (npm workspace `@desktopr/docs`),
absorbed from the separate `docs` repository in phase 14 as current sources
only: no Git history and no lockfile.

- The site documents the **open build workflow**: fork/clone, `npm run dev`,
  `npm run build`, and `build.yml` with signing driven by repository secrets.
  Every hosted-service page was deleted (pricing, build credits, Desktopr Edge,
  terms, privacy) together with the dashboard, status-page and hosted download
  links.
- No third-party requests: the ShareThis script and share buttons, the YouTube
  components, the home page presentation video and the Product Hunt /
  PostYourStartup badges were removed.
- The four configuration pages were empty placeholders and are now written
  (project structure, app settings, Tauri config, distribution).
- Kept and de-commercialised: the FAQ (`docs/support.md`, with support pointing at
  GitHub issues), the comparison page and table, branding and the Bubble.io
  plugin page.
- `npm run docs:build` runs in CI; VitePress fails the build on dead links.
- Deployment is not in the repository: the maintainer publishes the static
  output to Cloudflare, which already serves `desktopr.dev`. That host is the
  project's own site, so `scripts/check-standalone.mjs` allows it while still
  rejecting hosted Desktopr services such as `dashboard.desktopr.dev`. The
  bundle identifier default (`app.desktopr.app`) is deliberately unchanged: it
  is an identifier, not a URL, and changing it would move app data for anyone
  building with the defaults.

## Rejected

- A GitHub Pages deploy workflow (the maintainer deploys to Cloudflare).
- Keeping the documentation in a separate repository.
- Importing the source repository history.

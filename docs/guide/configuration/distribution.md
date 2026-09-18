---
title: Distribution
description: How to ship the installers Desktopr builds - artifacts, GitHub Releases, stores and self-hosting.
---

# Distribution

Desktopr builds installers and stops there: where they go is up to you. There is no Desktopr-hosted distribution service, and nothing is ever uploaded to a Desktopr server.

## What a build produces

| Platform | Outputs |
| --- | --- |
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | NSIS installer (`-setup.exe`), optional `.msix` |
| macOS | `.dmg` |

Local builds write them to `src-tauri/target/release/bundle/`.

## GitHub Actions artifacts

The **build** workflow uploads the installers of each platform as a workflow artifact, together with a `SHA256SUMS-<platform>.txt` file. Artifacts are enough for internal testing and for passing builds to your QA.

## GitHub Releases

Set `release: true` and an `app_version` and the workflow also creates a GitHub Release with every artifact attached. Useful inputs:

| Input | Purpose |
| --- | --- |
| `release_draft` | create the release as a draft (default) so you review it before publishing |
| `release_tag_prefix` | the tag is `<prefix><app_version>`, so several apps can release from one repository |
| `release_notes_file` | repository path of the release notes, instead of notes generated from commits |

The release job is the only one with write access to the repository contents.

## Stores

- **Microsoft Store:** provide the four `msix_*` identity inputs from Partner Center and the Windows build also produces an MSIX package, which you upload to Partner Center. The Store signs it for you. See [Microsoft Store](/guide/signing/microsoft-store).
- **Other stores:** any installer Desktopr produces can be submitted like a normal desktop app.

## Self-hosting and updates

You can serve the installers from your own website, S3 bucket, CDN or internal file server. If you also want in-app updates, host an update manifest yourself and enable the updater as described in [App Settings](/guide/configuration/app-settings#updates).

## Before you ship

- **Sign your builds.** Unsigned apps trigger SmartScreen and Gatekeeper warnings. See [Signing](/guide/signing/what).
- **Publish the checksums** next to the downloads so users can verify them.
- **Check the first-launch experience** on each platform you support, starting from a real download rather than from your build directory.

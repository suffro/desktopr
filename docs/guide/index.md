---
title: What is Desktopr?
description: Learn what Desktopr is, how it works, and what makes it different from other desktop app builders.
---

# What is Desktopr?

> Easily build and sign lightweight desktop applications using web stacks, with Desktopr GitHub actions and a straightforward Node API.

**Desktopr** is an open source, cross‑platform desktop builder that wraps any web app into a fully native desktop application for **Windows**, **macOS**, and **Linux** — powered by [**Tauri**](./what-is-tauri.md) and a TypeScript ↔ Rust bridge API.

It allows developers to take any existing web app (built with SvelteKit, React, Vue, Bubble.io or any other framework) and easily transform it into a real native desktop app, with full white-label and total ownership on your builds. The whole toolchain is [on GitHub](https://github.com/suffro/desktopr) under the Apache 2.0 license.

## Why Desktopr Exists

Building native desktop applications is traditionally complex.  
It involves:
- Maintaining separate codebases per OS  
- Handling native icons, installers, signatures, and updates  
- Managing certificates and build pipelines  
- Integrating OS‑level APIs manually (notifications, file system, shortcuts, etc.)

**Desktopr** automates all of this.

Developers keep their existing web app — and Desktopr takes care of everything else.

## How It Works

1. **You provide a public URL** and other details of your web app (or bundle local files).  
2. **Desktopr** builds a custom Tauri wrapper around it, on your machine or in your GitHub Actions.  
3. Inside that wrapper, a **Rust ↔ TypeScript bridge** exposes native APIs for:
   - File system access  
   - Global shortcuts  
   - Notifications and tray icons  
   - Deep links (`app://...`)  
   - Updater and auto‑launch  
   - Network and diagnostics  
   - And more, via `Desktopr.*`  

4. The build generates the native installers (NSIS/MSIX, DMG, AppImage/deb/rpm).

## Key Features

- ⚙️ **Desktop builds** from any web app  
- 🔗 **Full JS/TS bridge** for notifications, files, and OS events  
- 🔒 **Signed installers** with your own certificates, and optional self-hosted updates  
- 🧩 **Sandboxed WebAssembly plugins**  
- 🎨 **Custom branding** (icons, app name, colors)  
- 🧱 No "not-web code languages" knowledge required

## The Developer Experience

Desktopr abstracts away the “boring” setup of Tauri and Rust, while preserving full control over configuration files and API exposure.

Developers can:
- Define app metadata (name, identifiers, schemes)
- Configure native menu and tray icons
- Keep signing certificates as their own repository secrets
- Run cross-platform builds in GitHub Actions and download the installers
- Access native OS capabilities through the `Desktopr` bridge API using the `desktopr` Node module.

Everything is **modular**, **isolated**, and **reproducible**.

## Desktopr vs. Others

| Feature | Desktopr | Others |
|----------|-------------|-----------|
| Full ownership of binaries | ✅ You own them | ❌ Vendor‑locked |
| Native bridge API | ✅ Yes | ⚙️ Limited |
| Plugins | ✅ Sandboxed WASM | ❌ No |
| Source code | ✅ Open source (Apache 2.0) | ❌ Closed |
| Cost | Free | Subscription |

Desktopr is not a centralized service — it's an **independent builder** that you run yourself.
You keep full control over your binaries, storage, and branding.


## Who It’s For

Spoiler... everybody, here a few examples:
- Teams needing internal native tools without cross‑platform complexity  
- Agencies building branded desktop versions for clients  
- Small developers turning web apps into desktop products  
- No-code developers (eg. Bubble.io) who want true native integration  


## Summary

**Desktopr** turns your web app into a fully native desktop app.  
No complex setup. No platform‑specific maintenance. Just your app, everywhere.
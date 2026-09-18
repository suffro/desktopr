---
title: Desktopr Bridge Overview
description: Comprehensive overview of the Desktopr Bridge, its lifecycle, usage, and platform specifics.
prev:
    text: What is Desktopr
    link: guide/what-is-desktopr
next:
    text: App
    link: guide/bridge/api/app
---


# Desktopr Bridge

This is the official API for communicating with the app native wrapper with the `desktopr` Node module.

## Get Started

The `desktopr` Node module allows any web application to access native desktop features exposed by the Desktopr wrapper, using a clean and typed API.

If the app is running in a normal browser environment, `desktopr` provides a safe detection method  `isDesktoprAvailable()`  so you can fallback.


### Installation

Install `desktopr` with `npm` or any node package manager of your choice.

```bash
  npm install desktopr

```

with `yarn`:

```bash
  yarn add desktopr

```

## Usage Example

```ts
  import { Desktopr, isDesktoprAvailable } from "desktopr";

  if (isDesktoprAvailable()) {
    await Desktopr.window.new(); // opens a new window of the desktop app
  } else {
    console.log("Running in browser mode — native features unavailable.");
  }

```


## API Shape

The SDK exposes TypeScript definitions for the entire bridge via  `DesktoprAPI`, ensuring autocomplete and type safety.


## Detecting Native Environment

The SDK includes a lightweight helper:

```ts
  isDesktoprAvailable()
```

It  **never throws**, even in SSR or when running outside Desktopr.

Useful for apps that must run both:

  - as a normal website
  - and as a desktop app wrapped with Desktopr

### When Desktopr Is Not Available

If  `Desktopr`  is missing (e.g. browser mode), trying to call native APIs directly will throw.

Make sure to guard features or provide fallbacks:

```ts
  if (!isDesktoprAvailable()) return;
  await Desktopr.window.new(...);
```


## Modules

| Module           | Description                                                                 |
| ---------------- | --------------------------------------------------------------------------- |
| `app`            | Control application-level behaviors and retrieve app information.           |
| `notifications`  | Manage system notifications, including sending and scheduling notifications.|
| `clipboard`      | Read from and write to the system clipboard.                                |
| `files`          | Access and manipulate files on the local filesystem.                        |
| `window`         | Control and query application window behaviors (resize, focus, etc.).       |
| `events`         | Listen for and emit custom or system events.                                |
| `globalShortcut` | Register and handle global keyboard shortcuts.                              |
| `fs`             | Provides a dadicated filesystem for the app.                                |
| `menu`           | Manage and update application native menus.                                 |
| `diagnostics`    | Access diagnostic tools and logs for troubleshooting.                       |
| `network`        | Perform network requests and monitor connectivity state.                    |
| `autostart`      | Manage application auto-start behavior on system login.                     |
| `badge`          | Set the application badge count (macOS only).                               |
| `plugins`        | Load, manage and run WASM based plugins.                                    |

## Platform Notes

- The `badge` functionality, which allows setting an application badge count, is currently supported **only on macOS**.


:::warning
**Cross-Platform Limitations:** Some modules and features may behave differently or be unavailable depending on the operating system. Always check for availability and handle fallback logic accordingly.
:::
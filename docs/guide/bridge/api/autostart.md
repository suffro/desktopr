---
title: Autostart Module
description: Configure the automatic startup of the Desktopr Bridge app at user login.
prev:
    text: Window
    link: guide/bridge/api/window
next:
    text: Clipboard
    link: guide/bridge/api/clipboard
---


# Autostart Module

The Autostart Module allows you to configure the Desktopr Bridge application to start automatically when the user logs into their system. This feature ensures that the app is ready and running without manual intervention after system startup.

## Overview

Automatic startup is useful for background services or apps that need to maintain persistent connections.

It supports multiple startup modes to control how the app appears on launch:

- **shown**: The app window is displayed normally.
- **minimized**: The app starts minimized to the taskbar or dock.
- **hidden**: The app starts without showing any window.

You can enable or disable autostart and specify the preferred startup mode according to your needs.

## Methods

| Method                   | Description                                             |
|--------------------------|---------------------------------------------------------|
| `enable()`               | Enables autostart for the app.                          |
| `disable()`              | Disables autostart for the app.                         |
| `isEnabled()`            | Returns a boolean indicating if autostart is enabled.  |
| `mode.get()`             | Retrieves the current autostart mode as a string.      |
| `mode.set(mode: string)` | Sets the autostart mode. You can check accepted values [here](/guide/bridge/api/autostart#autostartmode) . |

## AutostartMode

| Mode       | Description                                      |
|------------|-------------------------------------------------|
| `shown`    | The app window is shown on startup.              |
| `minimized`| The app starts minimized to the taskbar or dock. |
| `hidden`   | The app starts without displaying any window.    |

## Example

```js
async function setupAutostart(mode) { // mode must be one of 'shown'|'minimized'|'hidden'
  const enabled = await Desktopr.autostart.isEnabled();
  if (!enabled) await Desktopr.autostart.enable();
  
  await Desktopr.autostart.mode.set(mode);
}

setupAutostart();
```

## Notes

:::tip
Always check the current autostart status with `isEnabled()` before attempting to enable or disable it to avoid unnecessary operations.
:::

:::warning
Autostart behavior and support may vary across operating systems. For example, Windows, macOS, and Linux handle autostart differently, and some modes might not be fully supported on all platforms.
:::

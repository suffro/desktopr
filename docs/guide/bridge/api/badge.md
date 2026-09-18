---
title: Badge Module
description: Manage the Dock badge count on macOS using the Desktopr Bridge.
prev:
    text: Plugins
    link: guide/bridge/api/plugins
next:
    text: Menu
    link: guide/bridge/api/menu
---


# Badge Module

:::danger Warning
This module is currently supported **only on macOS**.
:::

The `badge` module allows you to control the application’s Dock badge — the small red indicator typically used to display unread counts or notifications.  
This feature is **available only on macOS**. On Windows and Linux, this module will be `undefined` and should not be accessed without checking availability.

## Overview

Use the Badge Interface to display or clear numeric badges on the app’s Dock icon.  
Typical use cases include showing the number of unread messages, pending tasks, or notifications.

## Methods

| Method        | Description                                                                 |
|----------------|------------------------------------------------------------------------------|
| `set(count: number)` | Sets the numeric badge on the Dock icon to the specified value. Must be a non-negative integer. |
| `clear()`      | Clears the badge from the Dock icon, restoring it to its default state.     |

## Example

```javascript
// Check the module availability as it's supported on macOS only
if (Desktopr.badge) {

  // Set a badge count of n (eg. 5)
  await Desktopr.badge.set(5);

  // Clear the badge as needed
  await Desktopr.badge.clear();
  
} else {
  console.warn("Badge API is not available on this platform.");
}
```

## Notes

:::danger Warning
This module is currently supported **only on macOS**.
:::
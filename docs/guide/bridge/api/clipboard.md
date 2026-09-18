---
title: Clipboard Module
description: Read from and write to the system clipboard.
prev:
    text: Autostart
    link: guide/bridge/api/autostart
next:
    text: Utilities
    link: guide/bridge/api/utilities
---


# Clipboard Module

The `Clipboard` module lets your app read from and write text to the system clipboard.

## Methods

| Method         | Description                                                        | Return Type    |
|--------------- |--------------------------------------------------------------------|---------------|
| `readText()`       | Reads the current text content from the system clipboard and returns a promise that resolves to a string. | `Promise<string>` |
| `writeText(text: string)`       | Writes the provided text to the system clipboard and returns a promise that resolves when the operation is complete. | `Promise<void>`    |

## ClipboardInterface

```ts
interface ClipboardInterface {
    readText: () => Promise<string>;
    writeText: (text: string) => Promise<void>;
}
```


## Example Usage

```js
await Desktopr.clipboard.writeText("Hello, world!");

const text = await Desktopr.clipboard.readText();

console.log('Text from clipboard:', text);
```
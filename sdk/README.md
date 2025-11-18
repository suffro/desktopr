# Bubbledesk
[Bubbledesk official website ↗](https://bubbledesk.app)

This is the official JavaScript/TypeScript SDK for communicating with the native Bubbledesk bridge.  
It allows any web application to access native desktop features exposed by the Bubbledesk wrapper, using a clean, typed, importable API.

If the app is running in a normal browser environment, the SDK provides a safe detection method `isBubbledeskAvailable()` so you can fallback.

---

## Installation

```bash
npm install bubbledesk
```

or

```bash
yarn add bubbledesk
```

---

## Usage

```ts
import { Bubbledesk, isBubbledeskAvailable } from "bubbledesk";

if (isBubbledeskAvailable()) {
  await Bubbledesk.window.new();
} else {
  console.log("Running in browser mode — native features unavailable.");
}
```

---

## API Shape

The SDK exposes TypeScript definitions for the entire bridge via `BubbledeskAPI`, ensuring autocomplete and type safety.

---

## Detecting Native Environment

The SDK includes a lightweight helper:

```ts
isBubbledeskAvailable(): boolean
```

It **never throws**, even in SSR or when running outside Bubbledesk.

Useful for apps that must run both:
- as a normal website
- and as a desktop app wrapped with Bubbledesk


### When Bubbledesk Is Not Available

If `Bubbledesk` is missing (e.g. browser mode), trying to call native APIs directly will throw.

Make sure to guard features or provide fallbacks:

```ts
if (!isBubbledeskAvailable()) return;
await Bubbledesk.window.new(...);
```

---

[Bubbledesk official website ↗](https://bubbledesk.app)

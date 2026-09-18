---
title: Utilities
description: Desktopr utility functions and low-level runtime helpers.
prev:
    text: Clipboard
    link: guide/bridge/api/clipboard
next:
    text: App
    link: guide/bridge/api/app
---

# Bridge Utilities

This page documents small utility APIs exposed by Desktopr.

These functions are not tied to a single native module, but they are useful when you need to interact with the Desktopr runtime, open external URLs, or access the low-level Tauri proxy.

## openBrowser

`Desktopr.openBrowser()` opens a URL in the user's default external browser.

Use it when you want to send users outside the desktop app, for example to open your website, documentation, support page, billing portal, customer dashboard, or any other external resource.

::: info You may not need openBrowser
In a Desktopr app, `window.open()` and `<a target="_blank">` links already open in the user's default external browser by default — no extra API call needed.

Use `Desktopr.openBrowser()` only when you need to open a URL from code that cannot rely on those standard web mechanisms, for example from a background handler, a native event callback, or any logic path where there is no user-triggered anchor or `window.open` call.
:::


### Example

```ts
if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

await Desktopr.openBrowser('https://desktopr.dev');
```

If you want to open a URL inside a new native desktop window instead, use the [window module](/guide/bridge/api/window.md):

```ts
await Desktopr.window.new({
  url: 'https://desktopr.dev'
});
```

### API

```ts
Desktopr.openBrowser(url: string): Promise<void>
```

| Parameter | Type | Description |
| --- | --- | --- |
| `url` | `string` | The URL to open in the user's default external browser. |

**Returns:** `Promise<void>`

## onReady

`Desktopr.onReady()` registers a callback that runs when the Desktopr bridge is loaded and ready.

This is useful when your web app needs to run bridge-dependent logic as soon as the Desktopr runtime becomes available.

### Example

```ts
if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

Desktopr.onReady(() => {
  console.log('Desktopr is ready!');
});
```

### API

```ts
Desktopr.onReady(callback: Function): void
```

| Parameter | Type | Description |
| --- | --- | --- |
| `callback` | `Function` | Function executed when the Desktopr bridge is ready. |

**Returns:** `void`

## Tauri proxy

```ts
Desktopr.tauri
```

`Desktopr.tauri` exposes a low-level proxy to selected Tauri APIs available inside the Desktopr runtime.

Use `Desktopr.tauri` only when you need direct access to lower-level Tauri primitives such as `core.invoke`, event handling, or the current Tauri window object.

::: warning
`Desktopr.tauri` is a low-level advanced API.

Prefer the standard Desktopr Bridge modules whenever possible. Direct Tauri access may be more sensitive to internal runtime changes and should be used carefully.
:::

### Availability

`Desktopr.tauri` may be `undefined` if the current environment does not expose the underlying Tauri proxy.

Always check that it exists before using it:

```ts
if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

if (!Desktopr.tauri) {
  throw new Error('The Tauri proxy is not available in this environment.');
}

console.log(Desktopr.tauri);
```

### Invoking a Tauri command

You can use `Desktopr.tauri.core.invoke()` to call a Tauri command directly.

```ts
const result = await Desktopr.tauri.core.invoke('some_command', {
  value: 'Hello from Desktopr'
});

return result;
```

### Listening to Tauri events

You can listen to Tauri events through `Desktopr.tauri.event.listen()`.

```ts
const unlisten = await Desktopr.tauri.event.listen('some-event', (event) => {
  console.log('Received event:', event);
});

// Call this when you no longer need the listener.
unlisten();
```

### Emitting a Tauri event

You can emit a Tauri event through `Desktopr.tauri.event.emit()`.

```ts
await Desktopr.tauri.event.emit('some-event', {
  message: 'Hello from Desktopr'
});
```

### Getting the current Tauri window

You can access the current Tauri window through `Desktopr.tauri.window.getCurrent()`.

```ts
const currentWindow = Desktopr.tauri.window.getCurrent();

console.log(currentWindow);
```

---

<div style="color: var(--vp-c-brand-1);">

### TypeScript Interface

</div>

<details>

<summary>The Tauri proxy exposed by Desktopr follows this interface</summary>

```ts
interface EventCallback<T> {
  event: string;
  windowLabel: string;
  id: number;
  payload: T;
}

type TauriCore = {
  invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T>;
};

type TauriGlobal = TauriCore | { core: TauriCore };

type TauriArgs = Record<string, unknown>;

type UnlistenFn = () => void;

type TauriCoreExtended = {
  /**
   * Sends a message to the Rust backend.
   *
   * The command must be registered as a Tauri command.
   */
  invoke<T = unknown>(cmd: string, args?: TauriArgs): Promise<T>;

  /**
   * Converts a local file path into an asset URL that can be used inside the WebView.
   */
  convertFileSrc(filePath: string, protocol?: string): string;
};

type TauriEvent = {
  /**
   * Listens to an event emitted by the backend or another window.
   */
  listen<T>(event: string, handler: (event: EventCallback<T>) => void): Promise<UnlistenFn>;

  /**
   * Listens to an event only once.
   */
  once<T>(event: string, handler: (event: EventCallback<T>) => void): Promise<UnlistenFn>;

  /**
   * Emits an event to the backend and other Tauri windows.
   */
  emit(event: string, payload?: unknown): Promise<void>;
};

type TauriWindow = {
  /**
   * Returns the current Tauri window object.
   */
  getCurrent(): any;

  /**
   * Returns all available Tauri window objects.
   */
  getAll(): any[];
};

type TauriMock = {
  /**
   * Mocks IPC calls during tests, when available.
   */
  mockIPC(handler: (cmd: string, args: TauriArgs) => any): void;
};

interface WindowTauri {
  /**
   * Tauri core APIs.
   *
   * In Tauri v2, invoke is exposed under `core`.
   */
  core: TauriCoreExtended;

  /**
   * Tauri event APIs.
   */
  event: TauriEvent;

  /**
   * Tauri window APIs.
   */
  window: TauriWindow;

  /**
   * Optional testing utilities.
   */
  mocks?: TauriMock;

  /**
   * Additional runtime-specific APIs may be exposed here.
   */
  [key: string]: any;
}
```
</details>

---

:::tip Notes
`Desktopr.tauri` is intentionally low-level. For most use cases, use the Desktopr Bridge modules instead of calling Tauri APIs directly.

Direct Tauri access is useful when you need advanced behavior that is not yet wrapped by a dedicated Desktopr module.
:::

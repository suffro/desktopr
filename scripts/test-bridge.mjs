// Exercises the built SDK fallback and the bundled bridge initialization
// against a fake Tauri global. Build first: `npm run test:bridge`.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import vm from "node:vm";

const root = resolve(import.meta.dirname, "..");
const require = createRequire(import.meta.url);
const constants = JSON.parse(readFileSync(resolve(root, "src-ts/bridge.constants.json"), "utf8"));
let failures = 0;

async function test(name, run) {
  try {
    await run();
    console.log(`ok - ${name}`);
  } catch (error) {
    failures += 1;
    console.error(`not ok - ${name}\n${error.stack}`);
  }
}

// ---------- SDK ----------

const sdk = require(resolve(root, "sdk/dist-sdk/sdk/index.js"));

await test("SDK exports the documented entry points", () => {
  assert.equal(typeof sdk.Desktopr, "object");
  assert.equal(typeof sdk.isDesktoprAvailable, "function");
});

await test("SDK reports unavailability without throwing outside a browser", () => {
  delete globalThis.window;
  assert.equal(sdk.isDesktoprAvailable(), false);
  assert.throws(() => sdk.Desktopr.fs, /window is not defined/u);
});

await test("SDK throws a descriptive error in a browser without the wrapper", () => {
  globalThis.window = {};
  try {
    assert.equal(sdk.isDesktoprAvailable(), false);
    assert.throws(() => sdk.Desktopr.fs, /window\.Desktopr is not available/u);
    assert.throws(() => {
      sdk.Desktopr.isDesktop = true;
    }, /window\.Desktopr is not available/u);
  } finally {
    delete globalThis.window;
  }
});

await test("SDK forwards to the injected bridge with the bridge as receiver", () => {
  const bridge = {
    apiVersion: "test",
    whoAmI() {
      return this;
    },
  };
  globalThis.window = { Desktopr: bridge };
  try {
    assert.equal(sdk.isDesktoprAvailable(), true);
    assert.equal(sdk.Desktopr.apiVersion, "test");
    const whoAmI = sdk.Desktopr.whoAmI;
    assert.equal(whoAmI(), bridge);
  } finally {
    delete globalThis.window;
  }
});

// ---------- Bridge ----------

const bridgeSource = readFileSync(resolve(root, "src-tauri/tsc/bridge.js"), "utf8");

function createWebview() {
  const events = new EventTarget();
  const calls = [];
  const quiet = { log() {}, info() {}, debug() {}, warn() {}, error() {} };
  const context = {
    console: quiet,
    setTimeout,
    clearTimeout,
    setInterval,
    clearInterval,
    queueMicrotask,
    CustomEvent,
    Event,
    crypto: globalThis.crypto,
    requestAnimationFrame: (callback) => setTimeout(callback, 0),
    addEventListener: events.addEventListener.bind(events),
    removeEventListener: events.removeEventListener.bind(events),
    dispatchEvent: events.dispatchEvent.bind(events),
    __TAURI__: {
      core: {
        invoke: async (command, payload) => {
          // Copy out of the VM realm so deepEqual compares plain host objects.
          calls.push({ command, payload: JSON.parse(JSON.stringify(payload ?? null)) });
          return { label: "main" };
        },
      },
    },
  };
  context.window = context;
  vm.createContext(context);
  return { context, calls, evaluate: () => vm.runInContext(bridgeSource, context) };
}

await test("bridge defines a frozen window.Desktopr once", async () => {
  const webview = createWebview();
  let ready = false;
  webview.context.addEventListener("dtrReady", () => {
    ready = true;
  });
  webview.evaluate();

  const api = webview.context.Desktopr;
  assert.ok(api, "window.Desktopr was not defined");
  assert.equal(api.apiVersion, constants.apiVersion);
  assert.equal(api.isAvailable, true);
  assert.equal(api.isDesktop, true);
  for (const module of [
    "notifications", "clipboard", "files", "app", "window", "events", "globalShortcut", "fs",
    "menu", "diagnostics", "network", "autostart", "badge", "plugins", "contextMenu",
    "globalVariables", "tauri",
  ]) {
    assert.ok(api[module], `missing module ${module}`);
  }

  const descriptor = Object.getOwnPropertyDescriptor(webview.context, "Desktopr");
  assert.equal(descriptor.writable, false);
  assert.equal(descriptor.configurable, false);

  webview.evaluate();
  assert.equal(webview.context.Desktopr, api, "a second evaluation replaced the bridge");

  const deadline = Date.now() + 5_000;
  while (!ready && Date.now() < deadline) await new Promise((r) => setTimeout(r, 20));
  assert.ok(ready, "dtrReady was not dispatched");
});

await test("bridge commands reach Tauri with Rust argument names", async () => {
  const webview = createWebview();
  webview.evaluate();
  const api = webview.context.Desktopr;

  await api.invoke("dtr_custom", { value: 1 });
  await api.network.estimateBandwidth("https://example.com/sample", 1024, 500);

  assert.deepEqual(
    webview.calls.find((call) => call.command === "dtr_custom")?.payload,
    { value: 1 },
  );
  assert.deepEqual(
    webview.calls.find((call) => call.command === "dtr_network_bandwidth_estimate")?.payload,
    { url: "https://example.com/sample", sizeHintBytes: 1024, timeoutMs: 500 },
  );
});

await test("bridge companion label prefix matches the Rust runtime", () => {
  const companion = readFileSync(resolve(root, "src-tauri/src/bridge/companion.rs"), "utf8");
  const rustPrefix = companion.match(/COMPANION_LABEL_PREFIX: &str = "([^"]+)"/u)?.[1];
  assert.equal(constants.companionWindowLabelPrefix, rustPrefix);
});

if (failures > 0) {
  console.error(`${failures} bridge/SDK test(s) failed`);
  process.exitCode = 1;
}

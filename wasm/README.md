# Desktopr WASM modules

Desktopr executes WASI Preview 1 modules locally. A module reads one JSON object
from standard input and writes exactly one JSON object to standard output:

```json
{"fn":"add","args":[3,5]}
```

```json
{"ok":true,"value":8}
```

The runtime limits execution time and standard I/O, disables networking, and
preopens the temporary plugin job directory plus dedicated `/storage` for
persistent plugin data. Module diagnostics may use standard error, but standard
output must contain only the response JSON.

## Layout

- `module-template/`: minimal Rust starting point for a new module.
- `modules/math/`: working example module.

## Build

Install the Rust target once:

```sh
rustup target add wasm32-wasip1
```

From the repository root:

```sh
npm run wasm:check
npm run wasm:build:template
npm run wasm:build:math
```

Artifacts are written below each module's ignored `dist/` directory. The build
script also verifies the WebAssembly magic header before publishing the file.

In a running Desktopr app, install the artifact with the plugin picker and call
it through the existing bridge:

```js
await window.Desktopr.plugins.add("math");
const result = await window.Desktopr.plugins.call("math", {
  fn: "add",
  args: [3, 5],
});

console.log(result.value); // { ok: true, value: 8 }
```

Relative filesystem paths use temporary per-call storage. Use paths below
`/storage` when module data must persist across calls.

To add another module, copy `module-template`, give the Cargo package a unique
name, add it to the workspace members, and register its package and artifact
names in `build.mjs`.

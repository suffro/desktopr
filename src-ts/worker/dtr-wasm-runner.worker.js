"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
// All comments in English.
// Node-style globals polyfills for browser Worker
const buffer_1 = require("buffer");
globalThis.Buffer = globalThis.Buffer || buffer_1.Buffer;
globalThis.process = globalThis.process || { env: {} };
globalThis.global = globalThis.global || globalThis;
// Minimal WASI worker using ONLY the public @wasmer/wasi API.
const wasi_1 = require("@wasmer/wasi");
self.onmessage = async (ev) => {
    const { id, wasmBytes, payload, wasi, timeoutMs } = ev.data;
    const port = ev.ports?.[0];
    // Soft timeout (cannot preempt sync Wasm; aborts before run if expired)
    const ctrl = new AbortController();
    let timer;
    if (timeoutMs && timeoutMs > 0) {
        timer = setTimeout(() => ctrl.abort(), timeoutMs);
    }
    const reply = (r) => (port ? port.postMessage(r) : postMessage(r));
    try {
        if (!wasi)
            return reply({ ok: false, error: "This runner supports only WASI (wasm32-wasip1). Set wasi: true." });
        if (ctrl.signal.aborted)
            throw new Error("timeout");
        await (0, wasi_1.init)(); // required by @wasmer/wasi
        // Encode stdin JSON + newline (like `echo ... | wasmtime`)
        const stdinText = JSON.stringify(payload) + "\n";
        const wasiInst = new wasi_1.WASI({
            args: ["module.wasm"],
            env: {}, // add passthrough if you need
        });
        wasiInst.setStdinString(stdinText);
        const bytes = new Uint8Array(wasmBytes);
        const module = await WebAssembly.compile(bytes);
        const instance = await wasiInst.instantiate(module, {});
        const exitCode = wasiInst.start(instance);
        const stdoutRaw = wasiInst.getStdoutString();
        const stdout = typeof stdoutRaw === "string" ? stdoutRaw.trim() : stdoutRaw;
        const stderr = wasiInst.getStderrString();
        const value = typeof stdout === "string" ? tryParseJson(stdout) : undefined;
        reply({ ok: true, stdout, stderr, value });
    }
    catch (e) {
        reply({ ok: false, error: String(e?.message || e) });
    }
    finally {
        if (timer)
            clearTimeout(timer);
    }
};
function tryParseJson(s) { try {
    return JSON.parse(s);
}
catch {
    return undefined;
} }

import { readFileSync, readdirSync } from "node:fs";
import { extname, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");

// Commands companion (cache-only) windows must not reach: they open windows or
// change app-wide state outside the companion's own sandbox.
const COMPANION_DENIED = new Set([
  "dtr_win_open",
  "dtr_launch_companion",
  "dtr_plugin_add_module",
  "dtr_plugin_pick_and_add_module",
  "dtr_plugin_remove_module",
  "dtr_plugin_storage_clear",
  "dtr_plugin_clear_all_jobs",
  "dtr_autostart_enable",
  "dtr_autostart_disable",
  "dtr_set_autostart_mode",
  "dtr_logs_set_privacy",
  "dtr_logs_run_retention",
  "dtr_fs_diagnostics_rm",
  "dtr_fs_diagnostics_clear",
]);

const findings = [];

function permissionCommands(toml, identifier) {
  const block = toml.split("[[permission]]").find((b) => b.includes(`identifier = "${identifier}"`));
  if (!block) {
    findings.push(`permissions/desktopr-bridge.toml: missing permission "${identifier}"`);
    return new Set();
  }
  return new Set([...block.matchAll(/^\s*"(dtr_[a-z_]+)",?\s*$/gmu)].map((m) => m[1]));
}

// Returns release and debug-only (#[cfg(debug_assertions)]) handler commands.
function registeredCommands(mainRs) {
  const handler = mainRs.match(/generate_handler!\[([\s\S]*?)\]\)/u)?.[1] ?? "";
  const release = new Set();
  const debug = new Set();
  let debugOnly = false;
  for (const line of handler.split("\n")) {
    if (line.includes("#[cfg(debug_assertions)]")) {
      debugOnly = true;
      continue;
    }
    const name = line.match(/^\s*(dtr_[a-z_]+),?\s*$/u)?.[1];
    if (name) {
      (debugOnly ? debug : release).add(name);
      debugOnly = false;
    }
  }
  return { release, debug };
}

function invokedCommands(directory, out = new Set()) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) invokedCommands(path, out);
    else if (extname(entry.name) === ".ts") {
      for (const m of readFileSync(path, "utf8").matchAll(/invoke(?:<[^>]*>)?\(\s*["'](dtr_[a-z_]+)["']/gu)) {
        out.add(m[1]);
      }
    }
  }
  return out;
}

const toml = read("src-tauri/permissions/desktopr-bridge.toml");
const bridge = permissionCommands(toml, "desktopr-bridge");
const companion = permissionCommands(toml, "desktopr-bridge-companion");
const debugBridge = permissionCommands(toml, "desktopr-bridge-debug");
const { release: registered, debug: registeredDebug } = registeredCommands(read("src-tauri/src/main.rs"));
const invoked = invokedCommands(resolve(root, "src-ts"));

for (const command of invoked) {
  if (registeredDebug.has(command)) {
    if (!debugBridge.has(command)) findings.push(`${command}: debug-only command not allowed by desktopr-bridge-debug`);
    continue;
  }
  if (!registered.has(command)) findings.push(`${command}: invoked by the bridge but not registered in main.rs`);
  if (!bridge.has(command)) findings.push(`${command}: invoked by the bridge but not allowed by desktopr-bridge`);
}
for (const command of debugBridge) {
  if (!registeredDebug.has(command)) findings.push(`${command}: allowed by desktopr-bridge-debug but not a debug-only command`);
}
for (const command of bridge) {
  if (!registered.has(command)) findings.push(`${command}: allowed by desktopr-bridge but not registered in main.rs`);
}
for (const command of companion) {
  if (!bridge.has(command)) findings.push(`${command}: allowed for companions but not by desktopr-bridge`);
  if (COMPANION_DENIED.has(command)) findings.push(`${command}: must not be allowed for companion windows`);
}
for (const command of bridge) {
  if (!COMPANION_DENIED.has(command) && !companion.has(command)) {
    findings.push(`${command}: missing from desktopr-bridge-companion (or add it to COMPANION_DENIED)`);
  }
}

// Windows opened at runtime receive their own capability; the static one must
// not match companion windows through a wildcard.
for (const path of ["conf-templates/remote.template.json", "src-tauri/capabilities/remote.json"]) {
  const capability = JSON.parse(read(path));
  for (const key of ["windows", "webviews"]) {
    if (JSON.stringify(capability[key]) !== JSON.stringify(["main"])) {
      findings.push(`${path}: "${key}" must be ["main"]`);
    }
  }
}

if (findings.length > 0) {
  console.error("Bridge permission checks failed:");
  for (const finding of findings) console.error(`- ${finding}`);
  process.exitCode = 1;
} else {
  console.log(
    `Bridge permissions consistent: ${invoked.size} invoked, ${bridge.size} allowed, ${companion.size} for companions.`,
  );
}

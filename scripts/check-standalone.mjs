import { readFileSync, readdirSync } from "node:fs";
import { extname, relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const excludedDirectories = new Set([
  ".context",
  ".git",
  ".svelte-kit",
  "dist",
  "docs",
  "node_modules",
  "target",
]);
const textExtensions = new Set([
  ".html",
  ".js",
  ".json",
  ".md",
  ".mjs",
  ".rs",
  ".sh",
  ".toml",
  ".ts",
  ".yaml",
  ".yml",
]);
// The documentation site (desktopr.dev) is the project's own static site and is
// allowed. Hosted Desktopr/Bubbledesk services are not: the runtime must never
// depend on one.
const hostedDomainPattern =
  /https?:\/\/[^\s"'<>]*(?:[a-z0-9-]+\.desktopr\.(?:dev|app)|bubbledesk\.[a-z]+)/iu;
const findings = [];

function inspectTextFile(path) {
  const lines = readFileSync(path, "utf8").split(/\r?\n/u);
  lines.forEach((line, index) => {
    if (hostedDomainPattern.test(line)) {
      findings.push(`${relative(root, path)}:${index + 1} [hosted Desktopr/Bubbledesk URL]`);
    }
  });
}

function walk(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.isSymbolicLink() || excludedDirectories.has(entry.name)) continue;

    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      walk(path);
    } else if (textExtensions.has(extname(entry.name)) || entry.name === "window.env") {
      inspectTextFile(path);
    }
  }
}

walk(root);

const tauriConfig = JSON.parse(readFileSync(resolve(root, "src-tauri/tauri.conf.json"), "utf8"));
const capability = JSON.parse(
  readFileSync(resolve(root, "src-tauri/capabilities/remote.json"), "utf8"),
);
const bridgeConstants = JSON.parse(
  readFileSync(resolve(root, "src-ts/bridge.constants.json"), "utf8"),
);
const windowEnvironment = readFileSync(resolve(root, "src-tauri/window.env"), "utf8");
const cargoManifest = readFileSync(resolve(root, "src-tauri/Cargo.toml"), "utf8");
const defaultFeatures = cargoManifest.match(/default\s*=\s*\[([\s\S]*?)\]/u)?.[1] ?? "";

if (tauriConfig.plugins?.updater) findings.push("src-tauri/tauri.conf.json [updater enabled by default]");
if (tauriConfig.bundle?.createUpdaterArtifacts === true) {
  findings.push("src-tauri/tauri.conf.json [updater artifacts enabled by default]");
}
if (capability.remote?.urls?.length) {
  findings.push("src-tauri/capabilities/remote.json [remote origin enabled by default]");
}
if (capability.permissions?.includes("updater:default")) {
  findings.push("src-tauri/capabilities/remote.json [updater permission enabled by default]");
}
if (bridgeConstants.appUrl !== "") {
  findings.push("src-ts/bridge.constants.json [external application URL enabled by default]");
}
if (!/^MAIN_WINDOW_URL=$/mu.test(windowEnvironment)) {
  findings.push("src-tauri/window.env [external main window URL enabled by default]");
}
if (/tauri-plugin-updater|"updater"/u.test(defaultFeatures)) {
  findings.push("src-tauri/Cargo.toml [updater included in default Cargo features]");
}

// Restricted `com.apple.developer.*` entitlements need a provisioning profile;
// macOS kills unsigned or ad-hoc signed apps that declare them at launch.
const entitlements = readFileSync(resolve(root, "src-tauri/Entitlements.plist"), "utf8");
if (/<key>\s*com\.apple\.developer\./u.test(entitlements)) {
  findings.push("src-tauri/Entitlements.plist [restricted entitlement prevents unsigned macOS builds from launching]");
}

if (findings.length > 0) {
  console.error("Standalone configuration checks failed:");
  for (const finding of findings) console.error(`- ${finding}`);
  process.exitCode = 1;
} else {
  console.log("Standalone defaults contain no Desktopr hosted-service dependency.");
}

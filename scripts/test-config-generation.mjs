// Runs the configuration generators in a temporary copy of their inputs and
// checks the generated Tauri configuration, capability and constants files.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
let failures = 0;

function generate(script, environment = {}, prepare) {
  const workdir = mkdtempSync(join(tmpdir(), "desktopr-conf-"));
  cpSync(join(root, "scripts"), join(workdir, "scripts"), { recursive: true });
  cpSync(join(root, "conf-templates"), join(workdir, "conf-templates"), { recursive: true });
  mkdirSync(join(workdir, "src-tauri/capabilities"), { recursive: true });
  mkdirSync(join(workdir, "src-ts"), { recursive: true });
  prepare?.(workdir);

  // Start from a minimal environment so developer shell variables cannot leak in.
  const result = spawnSync("bash", [`scripts/${script}`], {
    cwd: workdir,
    encoding: "utf8",
    env: { PATH: process.env.PATH, HOME: process.env.HOME, ...environment },
  });
  const read = (path) => readFileSync(join(workdir, path), "utf8");
  const json = (path) => JSON.parse(read(path));
  return {
    status: result.status,
    output: `${result.stdout}${result.stderr}`,
    // Tauri resolves frontendDist from the directory holding tauri.conf.json.
    resolvedFrontend: (entry) => resolve(workdir, "src-tauri", json("src-tauri/tauri.conf.json").build.frontendDist, entry),
    tauri: () => json("src-tauri/tauri.conf.json"),
    capability: () => json("src-tauri/capabilities/remote.json"),
    constants: () => json("src-ts/bridge.constants.json"),
    windowEnv: () => read("src-tauri/window.env"),
    cargo: () => read("src-tauri/Cargo.toml"),
    cleanup: () => rmSync(workdir, { recursive: true, force: true }),
  };
}

function scenario(name, script, environment, check, prepare) {
  const run = generate(script, environment, prepare);
  try {
    check(run);
    console.log(`ok - ${name}`);
  } catch (error) {
    failures += 1;
    console.error(`not ok - ${name}\n${error.message}\n--- generator output ---\n${run.output}`);
  } finally {
    run.cleanup();
  }
}

function assertSucceeded(run) {
  assert.equal(run.status, 0, "generator exited with an error");
}

function assertMainOnly(capability) {
  assert.deepEqual(capability.windows, ["main"]);
  assert.deepEqual(capability.webviews, ["main"]);
}

scenario("production defaults are standalone", "prod-conf.sh", {}, (run) => {
  assertSucceeded(run);
  const tauri = run.tauri();
  const capability = run.capability();
  assertMainOnly(capability);
  assert.equal(capability.remote, undefined);
  assert.ok(!capability.permissions.includes("updater:default"));
  assert.ok(!capability.permissions.includes("desktopr-bridge-debug"));
  assert.ok(tauri.app.security.capabilities.includes("remote"));
  assert.match(tauri.app.security.csp, /^default-src 'self';/u);
  // A nonce or hash in script-src makes WKWebView drop 'unsafe-inline' and with
  // it the initialization scripts that carry the bridge.
  assert.deepEqual(tauri.app.security.dangerousDisableAssetCspModification, ["script-src"]);
  assert.equal(tauri.plugins?.updater, undefined);
  assert.equal(tauri.bundle.createUpdaterArtifacts, false);
  assert.equal(run.constants().appUrl, "");
  assert.match(run.windowEnv(), /^MAIN_WINDOW_URL=$/mu);
  assert.equal(
    run.cargo(),
    readFileSync(join(root, "src-tauri/Cargo.toml"), "utf8"),
    "generated Cargo.toml drifted from the checked-in manifest; update conf-templates/Cargo.template.toml",
  );
});

scenario(
  "production APP_URL grants only its origin and subdomains",
  "prod-conf.sh",
  { APP_URL: "https://app.example.com:8443/some/path?x=1" },
  (run) => {
    assertSucceeded(run);
    const capability = run.capability();
    assertMainOnly(capability);
    assert.deepEqual(capability.remote.urls, [
      "https://app.example.com:8443/*",
      "https://*.app.example.com:8443/*",
    ]);
    assert.equal(run.constants().appUrl, "https://app.example.com:8443");
    assert.match(run.tauri().app.security.csp, /default-src 'self' https:\/\/app\.example\.com:8443;/u);
    // The window never shows a bundled asset, so Tauri keeps rewriting them.
    assert.equal(run.tauri().app.security.dangerousDisableAssetCspModification, undefined);
  },
);

scenario("production rejects non-HTTP APP_URL", "prod-conf.sh", { APP_URL: "file:///etc" }, (run) => {
  assert.notEqual(run.status, 0);
});

scenario("companion mode accepts any HTTPS origin", "prod-conf.sh", { COMPANION_MODE: "true" }, (run) => {
  assertSucceeded(run);
  const capability = run.capability();
  assertMainOnly(capability);
  assert.ok(capability.remote.urls.includes("https://*/*"));
  assert.ok(!capability.remote.urls.some((url) => url.startsWith("http://") && !/localhost|127\.0\.0\.1/u.test(url)));
});

scenario("companion frontend bundles the companion app", "prod-conf.sh", { APP_FRONTEND: "companion" }, (run) => {
  assertSucceeded(run);
  const tauri = run.tauri();
  assert.equal(tauri.build.frontendDist, "../apps/companion/dist");
  const capability = run.capability();
  assertMainOnly(capability);
  assert.ok(capability.remote.urls.includes("https://*/*"), "companion must allow any HTTPS web app");
  assert.equal(run.constants().appUrl, "");
  assert.match(run.windowEnv(), /^MAIN_WINDOW_URL=$/mu);
  // The companion index.html carries an inline script: without this the nonce
  // Tauri adds to script-src blocks its own initialization scripts on macOS and
  // the companion opens without a bridge.
  assert.deepEqual(tauri.app.security.dangerousDisableAssetCspModification, ["script-src"]);
});

scenario(
  "companion frontend rejects APP_URL",
  "prod-conf.sh",
  { APP_FRONTEND: "companion", APP_URL: "https://app.example.com" },
  (run) => {
    assert.notEqual(run.status, 0);
  },
);

// Stages a built web application the way the build action copies it into the
// runtime checkout.
function withFrontendDist(directory, files = { "index.html": "<!doctype html><title>app</title>" }) {
  return (workdir) => {
    mkdirSync(join(workdir, directory), { recursive: true });
    for (const [name, content] of Object.entries(files)) {
      writeFileSync(join(workdir, directory, name), content);
    }
  };
}

scenario(
  "bundled frontend serves the developer's own build",
  "prod-conf.sh",
  { APP_FRONTEND: "bundled", APP_FRONTEND_DIST: "frontend-dist" },
  (run) => {
    assertSucceeded(run);
    const tauri = run.tauri();
    assert.equal(tauri.build.frontendDist, "../frontend-dist");
    assert.ok(
      existsSync(run.resolvedFrontend("index.html")),
      "frontendDist must resolve to the staged directory from src-tauri/",
    );
    // Local content is served by the app itself: no remote origin is granted.
    const capability = run.capability();
    assertMainOnly(capability);
    assert.equal(capability.remote, undefined);
    assert.match(tauri.app.security.csp, /^default-src 'self';/u);
    assert.equal(run.constants().appUrl, "");
    assert.match(run.windowEnv(), /^MAIN_WINDOW_URL=$/mu);
  },
  withFrontendDist("frontend-dist"),
);

scenario(
  "bundled frontend rejects APP_URL",
  "prod-conf.sh",
  { APP_FRONTEND: "bundled", APP_FRONTEND_DIST: "frontend-dist", APP_URL: "https://app.example.com" },
  (run) => {
    assert.notEqual(run.status, 0);
  },
  withFrontendDist("frontend-dist"),
);

scenario("bundled frontend requires a directory", "prod-conf.sh", { APP_FRONTEND: "bundled" }, (run) => {
  assert.notEqual(run.status, 0);
});

scenario(
  "bundled frontend rejects a missing directory",
  "prod-conf.sh",
  { APP_FRONTEND: "bundled", APP_FRONTEND_DIST: "frontend-dist" },
  (run) => {
    assert.notEqual(run.status, 0);
  },
);

scenario(
  "bundled frontend rejects a directory without index.html",
  "prod-conf.sh",
  { APP_FRONTEND: "bundled", APP_FRONTEND_DIST: "frontend-dist" },
  (run) => {
    assert.notEqual(run.status, 0);
  },
  withFrontendDist("frontend-dist", { "app.js": "console.log(1);" }),
);

scenario(
  "bundled frontend rejects a path escaping the checkout",
  "prod-conf.sh",
  { APP_FRONTEND: "bundled", APP_FRONTEND_DIST: "../elsewhere" },
  (run) => {
    assert.notEqual(run.status, 0);
  },
);

scenario("unknown frontend is rejected", "prod-conf.sh", { APP_FRONTEND: "other" }, (run) => {
  assert.notEqual(run.status, 0);
});

scenario("standalone frontend keeps the bundled fallback page", "prod-conf.sh", {}, (run) => {
  assertSucceeded(run);
  assert.deepEqual(run.tauri().build.frontendDist, ["standalone/index.html"]);
});

scenario(
  "updater requires both endpoint and public key",
  "prod-conf.sh",
  { UPDATE_ENDPOINT: "https://updates.example.com/latest.json" },
  (run) => {
    assert.notEqual(run.status, 0);
  },
);

scenario(
  "updater rejects non-HTTPS endpoints",
  "prod-conf.sh",
  { UPDATE_ENDPOINT: "http://updates.example.com/latest.json", ED25519_PUBKEY: "test-key" },
  (run) => {
    assert.notEqual(run.status, 0);
  },
);

scenario(
  "developer updater configuration is applied",
  "prod-conf.sh",
  { UPDATE_ENDPOINT: "https://updates.example.com/latest.json", TAURI_SIGNING_PUBLIC_KEY: "test-key" },
  (run) => {
    assertSucceeded(run);
    const tauri = run.tauri();
    assert.deepEqual(tauri.plugins.updater, {
      endpoints: ["https://updates.example.com/latest.json"],
      pubkey: "test-key",
    });
    assert.equal(tauri.bundle.createUpdaterArtifacts, true);
    assert.ok(run.capability().permissions.includes("updater:default"));
  },
);

for (const script of ["dev-conf.sh"]) {
  scenario(`${script} grants debug commands to main only`, script, {}, (run) => {
    assertSucceeded(run);
    const capability = run.capability();
    assertMainOnly(capability);
    assert.ok(capability.permissions.includes("desktopr-bridge-debug"));
    assert.equal(capability.remote, undefined);
    assert.equal(run.constants().appUrl, "");
  });
}

scenario("dev APP_URL grants only its origin", "dev-conf.sh", { APP_URL: "http://localhost:5173/app" }, (run) => {
  assertSucceeded(run);
  assert.deepEqual(run.capability().remote.urls, ["http://localhost:5173/*"]);
  assert.equal(run.constants().appUrl, "http://localhost:5173");
});

if (failures > 0) {
  console.error(`${failures} configuration generation scenario(s) failed`);
  process.exitCode = 1;
}

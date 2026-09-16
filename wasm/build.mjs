import { copyFile, mkdir, readFile, rm } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const wasmRoot = dirname(fileURLToPath(import.meta.url));
const target = "wasm32-wasip1";
const modules = {
  "module-template": {
    packageName: "desktopr_module_template",
    outputName: "module-template.wasm",
  },
  "modules/math": {
    packageName: "desktopr_math",
    outputName: "math.wasm",
  },
};

const requested = process.argv.slice(2);
const selected = requested.length === 0 ? Object.keys(modules) : requested;

for (const modulePath of selected) {
  if (!Object.hasOwn(modules, modulePath)) {
    console.error(`Unknown WASM module: ${modulePath}`);
    process.exit(2);
  }
}

for (const modulePath of selected) {
  const { packageName, outputName } = modules[modulePath];
  const build = spawnSync(
    "cargo",
    ["build", "--release", "--locked", "--target", target, "--package", packageName],
    { cwd: wasmRoot, stdio: "inherit" },
  );

  if (build.error) throw build.error;
  if (build.status !== 0) process.exit(build.status ?? 1);

  const source = join(wasmRoot, "target", target, "release", `${packageName}.wasm`);
  const destinationDirectory = join(wasmRoot, modulePath, "dist");
  const destination = join(destinationDirectory, outputName);

  await rm(destinationDirectory, { recursive: true, force: true });
  await mkdir(destinationDirectory, { recursive: true });
  await copyFile(source, destination);

  const bytes = await readFile(destination);
  if (!bytes.subarray(0, 4).equals(Buffer.from([0x00, 0x61, 0x73, 0x6d]))) {
    throw new Error(`Invalid WebAssembly artifact: ${destination}`);
  }

  console.log(`Built ${modulePath}/dist/${outputName}`);
}

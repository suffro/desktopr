import { readFileSync, readdirSync } from "node:fs";
import { relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const excludedDirectories = new Set([".git", "node_modules", "target"]);
const dependencyFiles = new Set([
  ".npmrc",
  "Cargo.lock",
  "Cargo.toml",
  "package-lock.json",
  "package.json",
]);

const findings = [];

function inspectFile(path) {
  const lines = readFileSync(path, "utf8").split(/\r?\n/u);

  lines.forEach((line, index) => {
    const categories = new Set();

    if (/suffro-lib|Suffro\/suffro-lib/iu.test(line)) {
      categories.add("private dependency");
    }

    if (/(?:github_pat_|gh[pousr]_)[A-Za-z0-9_]+/u.test(line)) {
      categories.add("GitHub token");
    }

    if (/(?:_authToken|_password)\s*=/iu.test(line)) {
      categories.add("package registry credential");
    }

    for (const match of line.matchAll(/(?:git\+)?https?:\/\/[^\s"'<>]+/giu)) {
      try {
        const url = new URL(match[0].replace(/^git\+/u, ""));
        if (url.username || url.password) {
          categories.add("credentialed URL");
        }
      } catch {
        // Invalid URLs are handled by their owning package manager.
      }
    }

    for (const category of categories) {
      findings.push(`${relative(root, path)}:${index + 1} [${category}]`);
    }
  });
}

function walk(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.isSymbolicLink() || excludedDirectories.has(entry.name)) {
      continue;
    }

    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      walk(path);
    } else if (dependencyFiles.has(entry.name)) {
      inspectFile(path);
    }
  }
}

walk(root);

if (findings.length > 0) {
  console.error("Private dependency or credential markers found:");
  for (const finding of findings) {
    console.error(`- ${finding}`);
  }
  process.exitCode = 1;
} else {
  console.log("No private dependency or credential markers found.");
}

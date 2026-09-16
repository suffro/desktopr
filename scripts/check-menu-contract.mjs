import assert from "node:assert/strict";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const Ajv = require("ajv");
const { DESKTOPR_MENU_CONFIG_JSON_SCHEMA } = require(
  "../sdk/dist-sdk/sdk/index.js",
);

const validate = new Ajv({ allErrors: true, strict: false }).compile(
  DESKTOPR_MENU_CONFIG_JSON_SCHEMA,
);

const validConfig = {
  enabled: true,
  platforms: ["linux", "windows", "macos"],
  file: {
    section: "file",
    items: [
      {
        type: "submenu",
        id: "tools",
        label: "Tools",
        items: [
          { type: "separator" },
          {
            type: "custom",
            id: "tools.run",
            label: "Run",
            enabled: true,
            interaction: "click",
          },
        ],
      },
    ],
  },
};

const missingRequiredRuntimeField = {
  enabled: true,
  platforms: ["linux"],
  file: {
    section: "file",
    items: [
      {
        type: "custom",
        id: "file.open",
        label: "Open",
        interaction: "click",
      },
    ],
  },
};

assert.equal(validate(validConfig), true, "valid runtime menu was rejected");
assert.equal(
  validate(missingRequiredRuntimeField),
  false,
  "menu missing a Rust-required field was accepted",
);

console.log("Desktopr menu contract checks passed.");

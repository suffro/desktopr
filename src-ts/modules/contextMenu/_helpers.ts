import { CmNode, CmType } from "@types";
import { validate } from "suffro-lib";

// Keep CM_TYPES as a readonly tuple and assert it matches CmType[]
export const CM_TYPES = [
  "item",
  "check",
  "separator",
  "submenu",
  "predefined",
]!;

// Narrow to CmType by inclusion
export function isCmType(value: unknown): value is CmType {
  return typeof value === "string" && (CM_TYPES as readonly string[]).includes(value);
}

/**
 * Normalize any unknown value into a CmType if possible:
 * - must be a non-empty string
 * - lowercased
 * - whitespace removed
 * - must be one of CM_TYPES
 */
export function parseCmType(value: unknown): CmType | null {
  if (!validate.nonEmptyString(value)) return null;
  const normalized = String(value).toLowerCase().trim().replace(/\s+/g, "");
  return isCmType(normalized) ? normalized : null;
}

/**
 * Return only entries with a valid, normalized type.
 * Does not mutate input entries.
 */
export function normalizeEntries(entries: ReadonlyArray<CmNode>): CmNode[] {
  const out: CmNode[] = [];
  for (const entry of entries) {
    const t = parseCmType((entry as any)?.type);
    if (!t) continue;
    out.push({ ...entry, type: t } as CmNode);
  }
  console.log(out);
  
  return out;
}
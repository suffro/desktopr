import { BubbledeskAPI } from "@types";
import { OpenResult } from "@types";

export function buildFiles(core: { invoke: BubbledeskAPI["invoke"] }) {
  return {
    open: (option?: { multi?: boolean }) =>
      core.invoke<OpenResult>("bd_file_open", { multi: option?.multi ?? false }),
    save: (default_name?: string | null) =>
      core.invoke<string>("bd_file_save", { default_name: default_name ?? null }),
  };
}

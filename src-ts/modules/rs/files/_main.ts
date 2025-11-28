import { BubbledeskAPI, FilesInterface, OpenResultWithBytes } from "../../../_types";
import { OpenResult } from "../../../_types";
import { U64 } from "suffro-lib";

export function buildFiles(core: { invoke: BubbledeskAPI["invoke"] }): FilesInterface {
  return {
    open: (options?: { multi?: boolean, allowed?: string[], maxBytes?: U64 }) =>
      core.invoke<OpenResult>("bd_file_open", { multi: options?.multi ?? false, allowedExtensions: options?.allowed, maxBytes: options?.maxBytes }),
    openWithBytes: (options?: { multi?: boolean, allowed?: string[], maxBytes?: U64 }) =>
      core.invoke<OpenResultWithBytes>("bd_file_open_with_bytes", { multi: options?.multi ?? false, allowedExtensions: options?.allowed, maxBytes: options?.maxBytes }),
    save: (defaultName?: string | null) =>
      core.invoke<string>("bd_file_save", { defaultName: defaultName ?? null }),
  };
}

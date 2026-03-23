import { DesktoprAPI, FilesInterface, OpenResultWithBytes } from "../../../_types";
import { OpenResult } from "../../../_types";
import { U64 } from "suffro-lib/utils";

export function buildFiles(core: { invoke: DesktoprAPI["invoke"] }): FilesInterface {
  return {
    open: (options?: { multi?: boolean, allowed?: string[], maxBytes?: U64 }) =>
      core.invoke<OpenResult>("dtr_file_open", { multi: options?.multi ?? false, allowedExtensions: options?.allowed, maxBytes: options?.maxBytes }),
    openWithBytes: (options?: { multi?: boolean, allowed?: string[], maxBytes?: U64 }) =>
      core.invoke<OpenResultWithBytes>("dtr_file_open_with_bytes", { multi: options?.multi ?? false, allowedExtensions: options?.allowed, maxBytes: options?.maxBytes }),
    save: (defaultName?: string | null) =>
      core.invoke<string>("dtr_file_save", { defaultName: defaultName ?? null }),
  };
}

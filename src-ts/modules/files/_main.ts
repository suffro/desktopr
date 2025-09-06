import { BubbledeskAPI, FilesInterface } from "@types";
import { OpenResult } from "@types";

export function buildFiles(core: { invoke: BubbledeskAPI["invoke"] }): FilesInterface {
  return {
    open: (option?: { multi?: boolean }) =>
      core.invoke<OpenResult>("bd_file_open", { multi: option?.multi ?? false }),
    save: (defaultName?: string | null) =>
      core.invoke<string>("bd_file_save", { defaultName: defaultName ?? null }),
  };
}

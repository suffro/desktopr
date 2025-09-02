import { BubbledeskAPI } from "@types";
import type { FsEntry, FsPaths, FsScopeMethods } from "@types";

function scope(core: { invoke: BubbledeskAPI["invoke"] }, permanent: boolean): FsScopeMethods {
  return {
    listDir: (rel: string = "") => core.invoke<FsEntry[]>("db_fs_list_dir", { rel, permanent }),
    mkdir: (rel: string) => core.invoke<void>("db_fs_mkdir", { rel, permanent }),
    rm: (rel: string, recursive = false) => core.invoke<void>("db_fs_rm", { rel, recursive, permanent }),
    stat: (rel: string = "") => core.invoke<FsEntry>("db_fs_stat", { rel, permanent }),
    writeText: (rel, contents, opts) =>
      core.invoke<void>("db_fs_write_text", {
        rel, permanent, contents,
        createDirs: opts?.createDirs, append: opts?.append,
      }),
    readText: (rel) => core.invoke<string>("db_fs_read_text", { rel, permanent }),
    writeBytes: (rel, base64, opts) =>
      core.invoke<void>("db_fs_write_bytes", {
        rel, permanent, dataBase64: base64, createDirs: opts?.createDirs,
      }),
    readBytes: (rel) => core.invoke<string>("db_fs_read_bytes", { rel, permanent }),
    exists: (rel) => core.invoke<boolean>("db_fs_exists", { rel, permanent }),
    move: (src, dest, opts) =>
      core.invoke<void>("db_fs_move", {
        src, dest, permanent,
        createDirs: opts?.createDirs, overwrite: opts?.overwrite,
      }),
    copy: (src, dest, opts) =>
      core.invoke<void>("db_fs_copy", {
        src, dest, permanent,
        recursive: opts?.recursive, createDirs: opts?.createDirs, overwrite: opts?.overwrite,
      }),
    path: async () => {
      const p = await core.invoke<FsPaths>("db_fs_paths");
      return permanent ? p.data : p.cache;
    },
    base: permanent ? ".data" : ".cache",
  };
}

export function buildFs(core: { invoke: BubbledeskAPI["invoke"] }) {
  const cache = scope(core, false);
  const data  = scope(core, true);
  return {
    cache: { ...cache, clear: () => core.invoke<void>("db_fs_clear_cache") },
    data,
    paths: () => core.invoke<FsPaths>("db_fs_paths"),
    base: { cache: ".cache", data: ".data" },
  };
}
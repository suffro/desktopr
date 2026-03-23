import { CompanionState } from "../../../_companion_context";
import { DesktoprAPI } from "../../../_types";
import type {
  FsEntry,
  FsInterface,
  FsPaths,
  FsScopeMethods,
} from "../../../_types";

const getWindowLabelIfCompanion = (): string | undefined => {
  const compState: CompanionState = window?.Desktopr?.window
    ?.state as CompanionState;
  let label: string | undefined = undefined;
  if (compState && compState?.windowLabel?.trim() && compState?.isCacheOnly) label = compState.windowLabel.trim();
  return label;
};

// Comments are in English
function scope(
  core: { invoke: DesktoprAPI["invoke"] },
  permanent: boolean
): FsScopeMethods {
  // Guard that prevents persistent data operations from isolated (companion) windows.
  const ensureDataNotIsolated = () => {
    if (!permanent) return;

    const label = getWindowLabelIfCompanion();
    if (label && label.trim().length > 0) {
      throw new Error(
        "Persistent data operations are not available in isolated companion windows. Use the cache scope (Desktopr.fs.cache) for per-session storage."
      );
    }
  };

  return {
    listContent: (rel: string = "") => {
      ensureDataNotIsolated();
      return core.invoke<FsEntry[]>("dtr_fs_list_dir", {
        rel,
        permanent,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    newDirectory: (rel: string) => {
      ensureDataNotIsolated();
      return core.invoke<void>("dtr_fs_mkdir", {
        rel,
        permanent,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    remove: (rel: string, recursive = false) => {
      ensureDataNotIsolated();
      return core.invoke<void>("dtr_fs_rm", {
        rel,
        recursive,
        permanent,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    stat: (rel: string = "") => {
      ensureDataNotIsolated();
      return core.invoke<FsEntry>("dtr_fs_stat", {
        rel,
        permanent,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    writeText: (rel, contents, opts) => {
      ensureDataNotIsolated();
      return core.invoke<void>("dtr_fs_write_text", {
        rel,
        permanent,
        contents,
        createDirs: opts?.createDirs,
        append: opts?.append,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    readText: (rel) => {
      ensureDataNotIsolated();
      return core.invoke<string>("dtr_fs_read_text", {
        rel,
        permanent,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    writeBytes: (rel, base64, opts) => {
      ensureDataNotIsolated();
      return core.invoke<void>("dtr_fs_write_bytes", {
        rel,
        permanent,
        dataBase64: base64,
        createDirs: opts?.createDirs,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    readBytes: (rel) => {
      ensureDataNotIsolated();
      return core.invoke<string>("dtr_fs_read_bytes", {
        rel,
        permanent,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    exists: (rel) => {
      ensureDataNotIsolated();
      return core.invoke<boolean>("dtr_fs_exists", {
        rel,
        permanent,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    move: (src, dest, opts) => {
      ensureDataNotIsolated();
      return core.invoke<void>("dtr_fs_move", {
        src,
        dest,
        permanent,
        createDirs: opts?.createDirs,
        overwrite: opts?.overwrite,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    copy: (src, dest, opts) => {
      ensureDataNotIsolated();
      return core.invoke<void>("dtr_fs_copy", {
        src,
        dest,
        permanent,
        recursive: opts?.recursive,
        createDirs: opts?.createDirs,
        overwrite: opts?.overwrite,
        windowLabel: getWindowLabelIfCompanion(),
      });
    },

    path: async () => {
      ensureDataNotIsolated();
      const p = await core.invoke<FsPaths>("dtr_fs_paths");
      return permanent ? p.data : p.cache;
    },

    clear: async () => {
      ensureDataNotIsolated();
      if (permanent) {
        return core.invoke<void>("dtr_fs_clear_data");
      }
      return core.invoke<void>("dtr_fs_clear_cache");
    },

    base: permanent ? ".data" : ".cache",
  };
}

export function buildFs(core: {
  invoke: DesktoprAPI["invoke"];
}): FsInterface {
  const cache = scope(core, false);
  const data = scope(core, true);
  return {
    cache,
    data,
    paths: async () => core.invoke<FsPaths>("dtr_fs_paths"),
    base: { cache: ".cache", data: ".data" },
    trash: {
      clear: async () => core.invoke<void>("dtr_fs_data_clear_trash"),
      recover: async () => core.invoke<void>("dtr_fs_data_recover_trash"),
      listContent: async (rel: string = "") =>
        core.invoke<FsEntry[]>("dtr_fs_trash_list_dir", { rel }),
      stat: async (rel: string = "") =>
        core.invoke<FsEntry>("dtr_fs_trash_stat", { rel }),
      exists: async (rel: string = "") =>
        core.invoke<boolean>("dtr_fs_trash_exists", { rel }),
      readText: async (rel: string) =>
        core.invoke<string>("dtr_fs_trash_read_text", { rel }),
      readBytes: async (rel: string) =>
        core.invoke<string>("dtr_fs_trash_read_bytes", { rel }),
    },
    diagnostics: {
      clear: async () => core.invoke<void>("dtr_fs_diagnostics_clear"),
      remove: async (rel: string, recursive = false) =>
        core.invoke<void>("dtr_fs_diagnostics_rm", { rel, recursive }),
      listContent: async (rel: string = "") =>
        core.invoke<FsEntry[]>("dtr_fs_diagnostics_list_dir", { rel }),
      stat: async (rel: string = "") =>
        core.invoke<FsEntry>("dtr_fs_diagnostics_stat", { rel }),
      exists: async (rel: string = "") =>
        core.invoke<boolean>("dtr_fs_diagnostics_exists", { rel }),
      readText: async (rel: string) =>
        core.invoke<string>("dtr_fs_diagnostics_read_text", { rel }),
      readBytes: async (rel: string) =>
        core.invoke<string>("dtr_fs_diagnostics_read_bytes", { rel }),
    },
  };
}

"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildFs = buildFs;
function scope(core, permanent) {
    return {
        listContent: (rel = "") => core.invoke("bd_fs_list_dir", { rel, permanent }),
        newDirectory: (rel) => core.invoke("bd_fs_mkdir", { rel, permanent }),
        remove: (rel, recursive = false) => core.invoke("bd_fs_rm", { rel, recursive, permanent }),
        stat: (rel = "") => core.invoke("bd_fs_stat", { rel, permanent }),
        writeText: (rel, contents, opts) => core.invoke("bd_fs_write_text", {
            rel, permanent, contents,
            createDirs: opts?.createDirs, append: opts?.append,
        }),
        readText: (rel) => core.invoke("bd_fs_read_text", { rel, permanent }),
        writeBytes: (rel, base64, opts) => core.invoke("bd_fs_write_bytes", {
            rel, permanent, dataBase64: base64, createDirs: opts?.createDirs,
        }),
        readBytes: (rel) => core.invoke("bd_fs_read_bytes", { rel, permanent }),
        exists: (rel) => core.invoke("bd_fs_exists", { rel, permanent }),
        move: (src, dest, opts) => core.invoke("bd_fs_move", {
            src, dest, permanent,
            createDirs: opts?.createDirs, overwrite: opts?.overwrite,
        }),
        copy: (src, dest, opts) => core.invoke("bd_fs_copy", {
            src, dest, permanent,
            recursive: opts?.recursive, createDirs: opts?.createDirs, overwrite: opts?.overwrite,
        }),
        path: async () => {
            const p = await core.invoke("bd_fs_paths");
            return permanent ? p.data : p.cache;
        },
        clear: async () => {
            if (permanent)
                core.invoke("bd_fs_clear_data");
            else
                core.invoke("bd_fs_clear_cache");
        },
        base: permanent ? ".data" : ".cache",
    };
}
function buildFs(core) {
    const cache = scope(core, false);
    const data = scope(core, true);
    return {
        cache,
        data,
        paths: async () => core.invoke("bd_fs_paths"),
        base: { cache: ".cache", data: ".data" },
        trash: {
            clear: async () => core.invoke("bd_fs_data_clear_trash"),
            recover: async () => core.invoke("bd_fs_data_recover_trash"),
            listContent: async (rel = "") => core.invoke("bd_fs_trash_list_dir", { rel }),
            stat: async (rel = "") => core.invoke("bd_fs_trash_stat", { rel }),
            exists: async (rel = "") => core.invoke("bd_fs_trash_exists", { rel }),
            readText: async (rel) => core.invoke("bd_fs_trash_read_text", { rel }),
            readBytes: async (rel) => core.invoke("bd_fs_trash_read_bytes", { rel }),
        },
        diagnostics: {
            clear: async () => core.invoke("bd_fs_diagnostics_clear"),
            remove: async (rel, recursive = false) => core.invoke("bd_fs_diagnostics_rm", { rel, recursive }),
            listContent: async (rel = "") => core.invoke("bd_fs_diagnostics_list_dir", { rel }),
            stat: async (rel = "") => core.invoke("bd_fs_diagnostics_stat", { rel }),
            exists: async (rel = "") => core.invoke("bd_fs_diagnostics_exists", { rel }),
            readText: async (rel) => core.invoke("bd_fs_diagnostics_read_text", { rel }),
            readBytes: async (rel) => core.invoke("bd_fs_diagnostics_read_bytes", { rel }),
        }
    };
}

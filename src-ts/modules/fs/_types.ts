export type FsEntry = {
    name: string;
    path: string;
    is_dir: boolean;
    size?: number | null;
  };
  
  export type FsPaths = { cache: string; data: string };
  
  export type FsScopeMethods = {
    listDir: (rel: string) => Promise<FsEntry[]>;
    mkdir: (rel: string) => Promise<void>;
    rm: (rel: string, recursive?: boolean) => Promise<void>;
    stat: (rel: string) => Promise<FsEntry>;
    writeText: (
      rel: string,
      contents: string,
      opts?: { createDirs?: boolean; append?: boolean }
    ) => Promise<void>;
    readText: (rel: string) => Promise<string>;
    writeBytes: (
      rel: string,
      base64: string,
      opts?: { createDirs?: boolean }
    ) => Promise<void>;
    readBytes: (rel: string) => Promise<string>;
    exists: (rel: string) => Promise<boolean>;
    move: (
      src: string,
      dest: string,
      opts?: { createDirs?: boolean; overwrite?: boolean }
    ) => Promise<void>;
    copy: (
      src: string,
      dest: string,
      opts?: { recursive?: boolean; createDirs?: boolean; overwrite?: boolean }
    ) => Promise<void>;
    path: () => Promise<string>;
    base: string;
  };
  

  export interface FsInterface {
      cache: FsScopeMethods & { clear: () => Promise<void> };
      data: FsScopeMethods;
      paths: () => Promise<FsPaths>;
      base: { cache: string; data: string };
  };
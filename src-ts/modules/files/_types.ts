export type OpenResult = { paths: string[] };

export interface FilesInterface {
      open: (option?: { multi?: boolean }) => Promise<OpenResult>;
      /** empty string if canceled */
      save: (defaultName?: string | null) => Promise<string>;
  };
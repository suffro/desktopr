// reserved for future window-related typed payloads
export type _WindowTypesPlaceholder = unknown;

export interface WindowInterface {
  devTools: {
    toggle: (label?: string) => Promise<void>;
    open: (label?: string) => Promise<void>;
    close: (label?: string) => Promise<void>;
  };
  new: (
    options?:
      | {
          label?: string | undefined;
          fullscreen?: boolean | undefined;
          url?: string | undefined;
        }
      | undefined
  ) => Promise<void>;
  close: (label: string) => Promise<void>;
  minimize: () => Promise<void>;
  maximizeToggle: () => Promise<void>;
  fullscreen: (enable: boolean) => Promise<void>;
}

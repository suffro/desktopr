// reserved for future window-related typed payloads
export type _WindowTypesPlaceholder = unknown;

export interface WindowInterface {
      devTools: {
          toogle: (label?: string) => Promise<void>;
          open: (label?: string) => Promise<void>;
          clse: (label?: string) => Promise<void>;
      };
      minimize: () => Promise<void>;
      maximizeToggle: () => Promise<void>;
      fullscreen: (enable: boolean) => Promise<void>;
  };
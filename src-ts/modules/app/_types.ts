export type AppInfo = unknown; // refine when you define a concrete shape

export interface AppInterface {
      info: () => Promise<AppInfo>;
  };
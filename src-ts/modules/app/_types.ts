import { I32 } from "suffro-lib";

export type AppInfo = unknown; // refine when you define a concrete shape

export interface AppInterface {
      info: () => Promise<AppInfo>;
      exit: (code?: I32 | undefined) => Promise<AppInfo>
  };
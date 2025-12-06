
export interface CompanionInterface {
    launch: (appConfig?: CompanionConfig) => Promise<void>
  };

export type CompanionConfig = {
  title: string;
  url: string;
  backgroundColor: string;
  fullscreen: boolean;
}
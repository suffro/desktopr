
export interface CompanionInterface {
    launch: (appConfig: object, menuConfig?: object | undefined) => Promise<void>
  };
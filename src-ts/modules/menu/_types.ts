export type MenuEventId =
  | "file.new-window"
  | "file.close-window"
  | "file.quit"
  | "view.reload"
  | "view.reload-hard"
  | "view.fullscreen"
  | "view.devtools"
  | "window.minimize"
  | "window.maximize"
  | "window.close-window"
  | "help.about"
  | "help.services"
  | "help.hide"
  | "help.hide-others"
  | "help.show-all";


export interface MenuInterface {
    setEnabled: (id: string, enabled: boolean) => Promise<void>;
    setChecked: (id: string, checked: boolean) => Promise<void>;
  };


export type MenuCustomItemEventPlayload = { id: string, [key: string]: any };

export interface MenuInterface {
    setEnabled: (id: string, enabled: boolean) => Promise<void>;
    setChecked: (id: string, checked: boolean) => Promise<void>;
  };

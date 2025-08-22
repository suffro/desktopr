export type ShortcutEventPayload = {
    accelerator: string;
    shortcut: string;
    id: number;
    state: "Pressed" | "Released" | string;
    [key: string]: any;
  };
  
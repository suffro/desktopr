import { CM_TYPES } from "@helpers";

export type CmType = typeof CM_TYPES[number];

export type CmItem = {
  type: Extract<CmType,"item">;
  id: string;
  text: string;
  enabled?: boolean;
  shortcut?: string; // e.g. "CmdOrCtrl+C"
};

export type CmCheck = {
  type: Extract<CmType,"check">;
  id: string;
  text: string;
  checked?: boolean;
  enabled?: boolean;
};

export type CmSeparator = {
  type: Extract<CmType,"separator">;
};

export type CmSubmenu = {
  type: Extract<CmType,"submenu">;
  text: string;
  items: CmNode[];
};

export type CmPredefined = {
  type: Extract<CmType,"predefined">;
  kind:
    | 'undo'
    | 'redo'
    | 'cut'
    | 'copy'
    | 'paste'
    | 'selectAll'
    | 'delete'
    | 'minimize'
    | 'closeWindow'
    | 'zoom'
    | 'hide'
    | 'hideOthers'
    | 'showAll'
    | 'quit'
    | 'services';
};

export type CmNode = CmItem | CmCheck | CmSeparator | CmSubmenu | CmPredefined;

export type CmPopupOptions = {
  window?: string;            // default: "main"
  atLogical?: [number, number];
  atPhysical?: [number, number];
  dismissOutside?: boolean;   // default: true
  timeoutMs?: number;         // default: 15000
};

export interface ContextMenuInterface {
    show: (entries: CmNode[], options: CmPopupOptions) => Promise<string>;
  };
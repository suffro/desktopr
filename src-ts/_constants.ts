

import CONSTANTS from "./bridge.constants.json";
import SDK_PACKAGE_JSON from "../sdk/package.json";

export const APP_URL: string = CONSTANTS.appUrl!;
export const API_VERSION: string = SDK_PACKAGE_JSON.version!;

export const COMAPNION_WINDOW_LABEL_PREFIX: string = CONSTANTS.companionWindowLabelPrefix;

export const READY_EVENT_NAME: string = "dtrReady";

export const WINDOWS_LABELS_TRACKER_VARIABLE_NAME: string = "open-windows-labels-tracker-rpkw6kjzxn8bfhj5u74q";

export const BROWSER_STORAGE_NAMESPACE: string = "dtr_xam8wknpz1vf";
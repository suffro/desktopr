import { BubbledeskAPI } from "../../../bubbledesk/_types";
export declare const tauriReadyCheck: () => boolean;
export declare const waitTauri: () => Promise<void>;
export declare const newWindow: (core: {
    invoke: BubbledeskAPI["invoke"];
}, options?: {
    label?: string;
    fullscreen?: boolean;
    url?: string;
}) => Promise<void>;

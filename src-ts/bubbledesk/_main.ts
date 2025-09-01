import { listenForEvent } from "@helpers";
import { BubbledeskAPI, BubbledeskInstanceInterface } from "@types";

export const BubbledeskInstance: BubbledeskInstanceInterface = {
    ready: (): boolean => {
        if(!window?.Bubbledesk) return false;
        return true;
    },
    get: (): BubbledeskAPI => {
        if(!BubbledeskInstance.ready()) throw("'window.Bubbledesk' not found");
        return window?.Bubbledesk as BubbledeskAPI;
    }
}

export const bdInitiators = async () => {
    try {
        if(!BubbledeskInstance.ready()) throw("'window.Bubbledesk' not found");

        const Bubbledesk = BubbledeskInstance.get()!; 
        
        Bubbledesk.events.onMenuEvent((id: string) => {
            if (id === "view.devtools") {
                console.log("DevTools toggled");
                Bubbledesk.window.devTools.toggle("main");
            }
        });

        console.log("## READY ##");
    } catch (error) {
        console.error(error);
    }
}
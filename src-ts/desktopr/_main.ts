// import { listenForEvent } from "../_helpers";
import { READY_EVENT_NAME } from "@constants";
import { DesktoprAPI, DesktoprInstanceInterface } from "../_types";

export const DesktoprInstance: DesktoprInstanceInterface = {
    ready: (): boolean => {
        if(!window?.Desktopr) return false;
        return true;
    },
    get: (): DesktoprAPI => {
        if(!DesktoprInstance.ready()) throw("'window.Desktopr' not found");
        return window?.Desktopr as DesktoprAPI;
    }
}

export const dtrInitiators = async () => {
    try {
        if(!DesktoprInstance.ready()) throw("'window.Desktopr' not found");

        const Desktopr = DesktoprInstance.get()!; 
        
        Desktopr.events.onMenuEvent((id: string) => {
            if (id === "view.devtools") {
                console.log("DevTools toggled");
                Desktopr.window.devTools.toggle("main");
            }
        });

        console.log("## READY ##");

        const eventReady = new CustomEvent(READY_EVENT_NAME);

        window?.dispatchEvent(eventReady);
    } catch (error) {
        console.error(error);
    }
}

export const dtrReadyEventListener = (callback: Function) => window.addEventListener(READY_EVENT_NAME, () => callback);
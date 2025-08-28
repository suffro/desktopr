import { MenuEventId, BubbledeskAPI, BubbledeskInstanceInterface } from "@types";

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

export const bdInitiators = () => {
    if(!BubbledeskInstance.ready()) return console.error("'window.Bubbledesk' not found");

    const Bubbledesk = BubbledeskInstance.get()!; 

    Bubbledesk.events.on("menu:event", (id: MenuEventId) => {
        if (id === "view.devtools") {
            console.log("DevTools toggled");
            Bubbledesk.window.devTools.toogle("main");
        }
    });
}
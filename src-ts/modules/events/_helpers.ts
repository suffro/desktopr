export const listenForEvent = async (event: string, handler: (payload: any) => void) => {
    const tauri = (window as any).__TAURI__;
    const eventApi = tauri?.event;
    if (!eventApi?.listen) throw new Error("Tauri event API not available");
    const unlisten = await eventApi.listen(event, (e: any) => handler(e?.payload));
    return () => unlisten();
  };
  
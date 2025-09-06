import type { BubbledeskAPI, NotificationsInterface } from "@types";

export function buildNotifications(core: { invoke: BubbledeskAPI["invoke"] }): NotificationsInterface {
  return {
    state: (): Promise<string> => core.invoke("bd_notification_state"),
    request: (): Promise<string> => core.invoke("bd_request_permission"),
    show: (title: string, body?: string): Promise<void> => core.invoke("bd_notify", { title, body }),
  };
}

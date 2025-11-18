"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildNotifications = buildNotifications;
function buildNotifications(core) {
    return {
        state: () => core.invoke("bd_notification_state"),
        request: () => core.invoke("bd_request_permission"),
        show: (title, body) => core.invoke("bd_notify", { title, body }),
    };
}

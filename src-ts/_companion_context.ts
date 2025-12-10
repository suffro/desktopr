// Comments are in English

import { COMAPNION_WINDOW_LABEL_PREFIX } from "./_constants";
import { WindowInfo } from "./_types";

export type CompanionState = {
  isCacheOnly?: boolean;
  sessionId?: string;
  windowLabel?: string;
};

let state: CompanionState = {
  isCacheOnly: false,
};

function parseCompanionWindowLabel(label: string): CompanionState {
        const prefix = COMAPNION_WINDOW_LABEL_PREFIX.trim();
        if (!label.startsWith(prefix)) {
            return { isCacheOnly: false };
        }
        const sessionId = label.slice(prefix.length);
        if (!sessionId) {
            return { isCacheOnly: false, windowLabel: label };
        }
        return { isCacheOnly: true, sessionId, windowLabel: label };
    }

let initialized = false;

export async function getCompanionContext(): Promise<void> {
  if (initialized) return;
  initialized = true;

  if (typeof window === 'undefined' || !window?.Bubbledesk) {
    console.error("Error executing setupCompanionContext():\nwindow.Bubbledesk undefined or not yet initialized.")
    return;
  }
    const windowInfo: WindowInfo = await window.Bubbledesk.window.getInfo();
    const compState = parseCompanionWindowLabel(windowInfo.label);

    if(window?.Bubbledesk?.window?.state) window.Bubbledesk.window.state = compState;
    // if(window?.Bubbledesk?.companion?.state) window.Bubbledesk.companion.state = compState;
}

// Simple getters

export function isCacheOnlyWindow(): boolean {
  return (state?.isCacheOnly) ?? false;
}

export function getCacheOnlyWindowLabel(): string | undefined {
  return state?.windowLabel;
}

export function getCacheOnlySessionId(): string | undefined {
  return state?.sessionId;
}
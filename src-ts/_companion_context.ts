// Comments are in English

import { COMAPNION_WINDOW_LABEL_PREFIX } from "./_constants";
import { WindowInfo } from "./_types";

export type CompanionState = {
  isCompanion?: boolean;
  sessionId?: string;
  windowLabel?: string;
};

let state: CompanionState = {
  isCompanion: false,
};

function parseCompanionWindowLabel(label: string): CompanionState {
        const prefix = COMAPNION_WINDOW_LABEL_PREFIX.trim();
        if (!label.startsWith(prefix)) {
            return { isCompanion: false };
        }
        const sessionId = label.slice(prefix.length);
        if (!sessionId) {
            return { isCompanion: false, windowLabel: label };
        }
        return { isCompanion: true, sessionId, windowLabel: label };
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

    if(window?.Bubbledesk?.window?.companionState) window.Bubbledesk.window.companionState = compState;
    if(window?.Bubbledesk?.companion?.state) window.Bubbledesk.companion.state = compState;
}

// Simple getters

export function isCompanionWindow(): boolean {
  return (state?.isCompanion) ?? false;
}

export function getCompanionWindowLabel(): string | undefined {
  return state?.windowLabel;
}

export function getCompanionSessionId(): string | undefined {
  return state?.sessionId;
}
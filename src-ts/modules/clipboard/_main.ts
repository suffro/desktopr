import type { BubbledeskAPI } from "@types";

export function buildClipboard(core: { invoke: BubbledeskAPI["invoke"] }) {
  return {
    readText: (): Promise<string> => core.invoke("bd_clipboard_read"),
    writeText: (text: string): Promise<void> => core.invoke("bd_clipboard_write", { text }),
  };
}

// src-ts/sdk/index.ts

// Main SDK entry: typed proxy over window.Bubbledesk
export { Bubbledesk } from "./_proxy";

// Re-export the API type so app devs can type their code against it if they want
export { type BubbledeskAPI } from "../_types";
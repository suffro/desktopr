import { SandboxCallInput } from "@types";

export const normalizeModuleName = (name: string): string => {
    const sanitizeWasmExtensions: string = name.replaceAll(".wasm","");
    const addWasmExtensions: string = `${sanitizeWasmExtensions}.wasm`;
    return addWasmExtensions;
}


export const normalizeModuleInput = (input: SandboxCallInput): SandboxCallInput => {
    let finalInput = input;
    const modulePath = input.module_path;
    finalInput.module_path = normalizeModuleName(modulePath);
    return finalInput;
}
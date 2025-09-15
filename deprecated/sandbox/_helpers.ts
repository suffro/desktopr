import { SandboxCallInput } from "@types";
import { validate } from "suffro-lib";

export const normalizeModuleName = (name: string): string => {
    const sanitizeWasmExtensions: string = name.replaceAll(".wasm","");
    const addWasmExtensions: string = `${sanitizeWasmExtensions}.wasm`;
    return addWasmExtensions;
}


export const normalizeModuleInput = (input: SandboxCallInput): SandboxCallInput => {
    if(validate.emptyObject(input)) throw new Error(`Invalid input\n${input}`);
    let finalInput = input;
    const modulePath = input.module_path;
    finalInput.module_path = normalizeModuleName(modulePath);
    // console.log("[input]",finalInput);
    return finalInput;
}

export const joinRelativePathToJob = (jobId: string, rel?: string): string => {
    const cleanJobId = jobId.replaceAll(" ","");
    const cleanRel = (rel??"").replaceAll(" ","");

    if(!validate.nonEmptyString(cleanJobId)) throw("Invalid job Id");
    
    if(validate.nonEmptyString(cleanRel)) return `${cleanJobId}/${cleanRel}`;
    else return cleanJobId;
}
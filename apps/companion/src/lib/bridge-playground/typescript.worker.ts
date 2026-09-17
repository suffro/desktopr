import * as Comlink from 'comlink';
import {
	createDefaultMapFromCDN,
	createSystem,
	createVirtualTypeScriptEnvironment
} from '@typescript/vfs';
import ts from 'typescript';
import { desktoprSdkTypes } from './generated/desktoprSdkTypes';

export type TsCompletionItem = {
	label: string;
	type: string;
	detail?: string;
	info?: string;
	apply?: string;
};

type IndexedMember = TsCompletionItem & {
	valueType?: string;
};

const MAIN_FILE = 'index.ts';

const compilerOptions: ts.CompilerOptions = {
	target: ts.ScriptTarget.ES2022,
	module: ts.ModuleKind.ESNext,
	moduleResolution: ts.ModuleResolutionKind.NodeJs,
	strict: false,
	noEmit: true,
	allowJs: true,
	checkJs: true,
	allowNonTsExtensions: true,
	skipLibCheck: true,
	lib: ['lib.es2022.d.ts', 'lib.dom.d.ts', 'lib.dom.iterable.d.ts']
};

const desktoprSupportTypes = `
type I8 = number;
type I16 = number;
type I32 = number;
type I64 = number;

type U8 = number;
type U16 = number;
type U32 = number;
type U64 = number;

type F32 = number;
type F64 = number;

declare const Desktopr: import("desktopr").DesktoprAPI;
declare function isDesktoprAvailable(): boolean;
`;

let env: ReturnType<typeof createVirtualTypeScriptEnvironment> | null = null;
let indexedTypes: Map<string, IndexedMember[]> | null = null;

function getMemberName(member: ts.TypeElement): string | null {
	const name = member.name;

	if (!name) return null;

	if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) {
		return name.text;
	}

	return null;
}

function getJsDoc(member: ts.Node, sourceFile: ts.SourceFile): string | undefined {
	const comments = ts.getJSDocCommentsAndTags(member);

	if (!comments.length) return undefined;

	return comments
		.map((comment) => comment.getText(sourceFile))
		.join('\n')
		.replace(/\/\*\*|\*\/|\n\s*\*/g, '')
		.trim();
}

function cleanTypeName(typeText: string | undefined): string | undefined {
	if (!typeText) return undefined;

	let cleaned = typeText.trim();

	cleaned = cleaned.replace(/^readonly\s+/, '');
	cleaned = cleaned.replace(/\s*\|\s*undefined/g, '');
	cleaned = cleaned.replace(/\s*\|\s*null/g, '');
	cleaned = cleaned.replace(/\s*&\s*\{[\s\S]*$/, '');
	cleaned = cleaned.replace(/^Promise<(.+)>$/, '$1');
	cleaned = cleaned.replace(/^Readonly<(.+)>$/, '$1');
	cleaned = cleaned.replace(/\[\]$/, '');

	return cleaned.trim();
}

function convertKindFromMember(member: ts.TypeElement): string {
	if (ts.isMethodSignature(member)) return 'function';

	if (ts.isPropertySignature(member)) {
		const typeText = member.type?.getText();

		if (!typeText) return 'property';

		if (
			typeText.includes('Interface') ||
			typeText.includes('Methods') ||
			typeText.includes('{')
		) {
			return 'namespace';
		}

		return 'property';
	}

	return 'property';
}

function addMembers(typeMap: Map<string, IndexedMember[]>, typeName: string, members: IndexedMember[]) {
	const existing = typeMap.get(typeName) ?? [];
	const merged = [...existing];

	for (const member of members) {
		if (!merged.some((item) => item.label === member.label)) {
			merged.push(member);
		}
	}

	typeMap.set(typeName, merged);
}

function indexTypeLiteral(
	typeMap: Map<string, IndexedMember[]>,
	sourceFile: ts.SourceFile,
	typeName: string,
	members: ts.NodeArray<ts.TypeElement>
) {
	const indexedMembers: IndexedMember[] = [];

	for (const member of members) {
		const name = getMemberName(member);

		if (!name) continue;

		let detail: string | undefined;
		let valueType: string | undefined;

		if (ts.isMethodSignature(member)) {
			const parameters = member.parameters
				.map((parameter) => parameter.getText(sourceFile))
				.join(', ');

			const returnType = member.type?.getText(sourceFile) ?? 'unknown';

			detail = `${name}(${parameters}): ${returnType}`;
			valueType = cleanTypeName(returnType);
		}

		if (ts.isPropertySignature(member)) {
			const optional = member.questionToken ? '?' : '';
			const typeText = member.type?.getText(sourceFile) ?? 'unknown';

			detail = `${name}${optional}: ${typeText}`;

			if (member.type && ts.isTypeLiteralNode(member.type)) {
				const syntheticTypeName = `${typeName}.${name}`;
				indexTypeLiteral(typeMap, sourceFile, syntheticTypeName, member.type.members);
				valueType = syntheticTypeName;
			} else {
				valueType = cleanTypeName(typeText);
			}
		}

		indexedMembers.push({
			label: name,
			type: convertKindFromMember(member),
			detail,
			info: getJsDoc(member, sourceFile),
			apply: name,
			valueType
		});
	}

	addMembers(typeMap, typeName, indexedMembers);
}

function visitNode(typeMap: Map<string, IndexedMember[]>, sourceFile: ts.SourceFile, node: ts.Node) {
	if (ts.isInterfaceDeclaration(node)) {
		indexTypeLiteral(typeMap, sourceFile, node.name.text, node.members);
		return;
	}

	if (ts.isTypeAliasDeclaration(node)) {
		if (ts.isTypeLiteralNode(node.type)) {
			indexTypeLiteral(typeMap, sourceFile, node.name.text, node.type.members);
		}

		return;
	}

	if (ts.isModuleDeclaration(node) && node.body && ts.isModuleBlock(node.body)) {
		for (const statement of node.body.statements) {
			visitNode(typeMap, sourceFile, statement);
		}

		return;
	}

	ts.forEachChild(node, (child) => visitNode(typeMap, sourceFile, child));
}

function getIndexedTypes() {
	if (indexedTypes) return indexedTypes;

	const typeMap = new Map<string, IndexedMember[]>();

	const sourceFile = ts.createSourceFile(
		'desktopr-sdk.d.ts',
		desktoprSdkTypes,
		ts.ScriptTarget.Latest,
		true,
		ts.ScriptKind.TS
	);

	visitNode(typeMap, sourceFile, sourceFile);

	indexedTypes = typeMap;

	return indexedTypes;
}

function resolveMembersForDesktoprChain(chain: string): IndexedMember[] {
	const typeMap = getIndexedTypes();
	const parts = chain.split('.');

	if (parts[0] !== 'Desktopr') return [];

	let currentType = 'DesktoprAPI';

	for (const part of parts.slice(1)) {
		const members = typeMap.get(currentType) ?? [];
		const member = members.find((item) => item.label === part);

		if (!member?.valueType) return [];

		currentType = member.valueType;
	}

	return typeMap.get(currentType) ?? [];
}

function getDesktoprChainCompletion(code: string, position: number): TsCompletionItem[] | null {
	const beforeCursor = code.slice(0, position);

	const match = beforeCursor.match(
		/(Desktopr(?:\.[A-Za-z_$][\w$]*)*)\.([A-Za-z_$][\w$]*)?$/
	);

	if (!match) return null;

	const chain = match[1];
	const partial = match[2] ?? '';

	const members = resolveMembersForDesktoprChain(chain);

	return members
		.filter((member) => member.label.startsWith(partial))
		.map((member) => ({
			label: member.label,
			type: member.type,
			detail: member.detail,
			info: member.info,
			apply: member.apply ?? member.label
		}));
}

async function getEnv() {
	if (env) return env;

	const fsMap = await createDefaultMapFromCDN(
		compilerOptions,
		ts.version,
		false,
		ts
	);

	fsMap.set(MAIN_FILE, 'Desktopr;\nisDesktoprAvailable;\n');

	fsMap.set('desktopr-sdk.d.ts', desktoprSdkTypes);
	fsMap.set('desktopr-playground-globals.d.ts', desktoprSupportTypes);

	fsMap.set('node_modules/desktopr/index.d.ts', desktoprSdkTypes);
	fsMap.set('node_modules/desktopr/dist-sdk/sdk/_proxy.d.ts', desktoprSdkTypes);
	fsMap.set('node_modules/desktopr/dist-sdk/sdk/index.d.ts', desktoprSdkTypes);

	const system = createSystem(fsMap);

	env = createVirtualTypeScriptEnvironment(
		system,
		[
			MAIN_FILE,
			'desktopr-sdk.d.ts',
			'desktopr-playground-globals.d.ts',
			'node_modules/desktopr/index.d.ts',
			'node_modules/desktopr/dist-sdk/sdk/_proxy.d.ts',
			'node_modules/desktopr/dist-sdk/sdk/index.d.ts'
		],
		ts,
		compilerOptions
	);

	return env;
}

function convertKind(kind: string | undefined) {
	switch (kind) {
		case ts.ScriptElementKind.memberFunctionElement:
		case ts.ScriptElementKind.functionElement:
		case ts.ScriptElementKind.localFunctionElement:
			return 'function';

		case ts.ScriptElementKind.memberVariableElement:
		case ts.ScriptElementKind.variableElement:
		case ts.ScriptElementKind.constElement:
		case ts.ScriptElementKind.letElement:
			return 'variable';

		case ts.ScriptElementKind.classElement:
			return 'class';

		case ts.ScriptElementKind.interfaceElement:
			return 'interface';

		case ts.ScriptElementKind.moduleElement:
			return 'namespace';

		case ts.ScriptElementKind.keyword:
			return 'keyword';

		default:
			return 'property';
	}
}

const api = {
	async initialize() {
		getIndexedTypes();
		await getEnv();
		return true;
	},

	async getCompletions(code: string, position: number): Promise<TsCompletionItem[]> {
		const desktoprChainCompletions = getDesktoprChainCompletion(code, position);

		if (desktoprChainCompletions) {
			return desktoprChainCompletions;
		}

		const currentEnv = await getEnv();

		currentEnv.updateFile(MAIN_FILE, code);

		const completions = currentEnv.languageService.getCompletionsAtPosition(
			MAIN_FILE,
			position,
			{
				includeCompletionsForModuleExports: true,
				includeCompletionsWithInsertText: true,
				includeAutomaticOptionalChainCompletions: true
			}
		);

		if (!completions?.entries?.length) {
			return [];
		}

		return completions.entries.map((entry) => {
			const details = currentEnv.languageService.getCompletionEntryDetails(
				MAIN_FILE,
				position,
				entry.name,
				{},
				entry.source,
				{},
				entry.data
			);

			const detail = details
				? ts.displayPartsToString(details.displayParts)
				: undefined;

			const info = details
				? ts.displayPartsToString(details.documentation)
				: undefined;

			return {
				label: entry.name,
				type: convertKind(entry.kind),
				detail,
				info,
				apply: entry.insertText || entry.name
			};
		});
	}
};

Comlink.expose(api);
import fs from 'node:fs';
import path from 'node:path';

// The SDK workspace (built with `npm run sdk:build`) is the package the playground types come from.
const DESKTOPR_PACKAGE_DIR = path.resolve(import.meta.dirname, '../../../sdk');
const OUTPUT_FILE = path.resolve(import.meta.dirname, '../src/lib/bridge-playground/generated/desktoprSdkTypes.ts');

const ENTRY_CANDIDATES = [
	'dist-sdk/sdk/_proxy.d.ts',
	'dist-sdk/sdk/index.d.ts',
	'dist-sdk/index.d.ts',
	'dist/index.d.ts',
	'index.d.ts'
];

function fileExists(filePath) {
	try {
		return fs.existsSync(filePath) && fs.statSync(filePath).isFile();
	} catch {
		return false;
	}
}

function readPackageJson(packageDir) {
	const packageJsonPath = path.join(packageDir, 'package.json');

	if (!fileExists(packageJsonPath)) {
		throw new Error(`Could not find package.json in ${packageDir}`);
	}

	return JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
}

function walkFiles(dir, predicate, collected = []) {
	if (!fs.existsSync(dir)) return collected;

	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		const fullPath = path.join(dir, entry.name);

		if (entry.isDirectory()) {
			if (entry.name === 'node_modules') continue;
			walkFiles(fullPath, predicate, collected);
			continue;
		}

		if (entry.isFile() && predicate(fullPath)) {
			collected.push(fullPath);
		}
	}

	return collected;
}

function normalizeToPosix(filePath) {
	return filePath.split(path.sep).join('/');
}

function indent(text, prefix) {
	return text
		.split('\n')
		.map((line) => (line.trim().length > 0 ? `${prefix}${line}` : line))
		.join('\n');
}

function stripDeclarationImports(source) {
	return source
		.replace(/^import\s+type\s+.*?;$/gm, '')
		.replace(/^import\s+.*?;$/gm, '')
		.replace(/^export\s+\*\s+from\s+['"].*?['"];$/gm, '');
}

function normalizeDeclarationBody(source) {
	return stripDeclarationImports(source)
		.replace(/\bexport\s+declare\s+/g, 'export ')
		.replace(/\bdeclare\s+/g, '')
		.trim();
}

function getModuleNamesForDeclaration(relativePath) {
	const normalizedPath = normalizeToPosix(relativePath).replace(/\.d\.ts$/, '');

	const moduleNames = new Set([
		'desktopr',
		`desktopr/${normalizedPath}`
	]);

	if (normalizedPath === 'dist-sdk/sdk/_proxy') {
		moduleNames.add('desktopr/dist-sdk/sdk/_proxy');
	}

	if (normalizedPath === 'dist-sdk/sdk/index') {
		moduleNames.add('desktopr/dist-sdk/sdk/index');
	}

	return Array.from(moduleNames);
}

function convertModuleDeclarations(source, relativePath) {
	const body = normalizeDeclarationBody(source);

	if (!body) {
		return getModuleNamesForDeclaration(relativePath)
			.map((moduleName) => `declare module "${moduleName}" {\n}`)
			.join('\n\n');
	}

	return getModuleNamesForDeclaration(relativePath)
		.map((moduleName) => {
			return `declare module "${moduleName}" {\n${indent(body, '\t')}\n}`;
		})
		.join('\n\n');
}

function findEntryDeclaration(packageDir, packageJson) {
	const packageTypesCandidates = [
		packageJson.types,
		packageJson.typings,
		packageJson.exports?.types,
		packageJson.exports?.['.']?.types,
		packageJson.exports?.['.']?.import?.types,
		packageJson.exports?.['.']?.default?.types
	].filter(Boolean);

	const candidates = [
		...packageTypesCandidates,
		...ENTRY_CANDIDATES
	];

	for (const candidate of candidates) {
		if (typeof candidate !== 'string') continue;

		const candidatePath = path.resolve(packageDir, candidate);

		if (fileExists(candidatePath)) {
			return candidatePath;
		}
	}

	const allDeclarations = walkFiles(packageDir, (filePath) => filePath.endsWith('.d.ts'));

	const proxyDeclaration = allDeclarations.find((filePath) => {
		const normalized = normalizeToPosix(filePath);
		return normalized.endsWith('/dist-sdk/sdk/_proxy.d.ts');
	});

	if (proxyDeclaration) return proxyDeclaration;

	const desktoprApiDeclaration = allDeclarations.find((filePath) => {
		const source = fs.readFileSync(filePath, 'utf8');
		return source.includes('Desktopr') || source.includes('DesktoprAPI');
	});

	if (desktoprApiDeclaration) return desktoprApiDeclaration;

	return allDeclarations[0] ?? null;
}

function collectRelevantDeclarations(packageDir, entryDeclaration) {
	const allDeclarations = walkFiles(packageDir, (filePath) => filePath.endsWith('.d.ts'));

	const relevant = allDeclarations.filter((filePath) => {
		const normalized = normalizeToPosix(path.relative(packageDir, filePath));

		return (
			filePath === entryDeclaration ||
			normalized.startsWith('dist-sdk/') ||
			normalized.startsWith('dist/')
		);
	});

	return Array.from(new Set(relevant)).sort();
}

function getGlobalDeclarationModule(packageDir, entryDeclaration) {
	const proxyDeclarationPath = path.join(packageDir, 'dist-sdk/sdk/_proxy.d.ts');

	if (fileExists(proxyDeclarationPath)) {
		return 'desktopr/dist-sdk/sdk/_proxy';
	}

	const entryRelativePath = normalizeToPosix(path.relative(packageDir, entryDeclaration));
	return `desktopr/${entryRelativePath.replace(/\.d\.ts$/, '')}`;
}

function buildGeneratedFile(packageDir, declarationFiles, entryDeclaration) {
	const packageJson = readPackageJson(packageDir);
	const entryRelativePath = normalizeToPosix(path.relative(packageDir, entryDeclaration));
	const globalDeclarationModule = getGlobalDeclarationModule(packageDir, entryDeclaration);

	const declarationBlocks = declarationFiles
		.map((filePath) => {
			const relativePath = normalizeToPosix(path.relative(packageDir, filePath));
			const source = fs.readFileSync(filePath, 'utf8');

			return [
				`// Source: desktopr/${relativePath}`,
				convertModuleDeclarations(source, relativePath)
			].join('\n');
		})
		.join('\n\n');

	const playgroundGlobals = `
declare global {
	const Desktopr: typeof import("${globalDeclarationModule}").Desktopr;
	const isDesktoprAvailable: typeof import("${globalDeclarationModule}").isDesktoprAvailable;
}

export {};
`.trim();

	const fullTypes = `
${declarationBlocks}

${playgroundGlobals}
`.trim();

	return `// This file is generated automatically.
// Do not edit it manually.
// Source package: desktopr@${packageJson.version ?? 'unknown'}
// Entry declaration: desktopr/${entryRelativePath}
// Global declaration module: ${globalDeclarationModule}

export const desktoprSdkTypes = ${JSON.stringify(fullTypes)};
`;
}

function main() {
	if (!fs.existsSync(DESKTOPR_PACKAGE_DIR)) {
		throw new Error(
			`Could not find the desktopr package at ${DESKTOPR_PACKAGE_DIR}. Did you run npm install?`
		);
	}

	const packageJson = readPackageJson(DESKTOPR_PACKAGE_DIR);
	const entryDeclaration = findEntryDeclaration(DESKTOPR_PACKAGE_DIR, packageJson);

	if (!entryDeclaration) {
		throw new Error('Could not find any .d.ts file inside node_modules/desktopr.');
	}

	const declarationFiles = collectRelevantDeclarations(DESKTOPR_PACKAGE_DIR, entryDeclaration);

	if (declarationFiles.length === 0) {
		throw new Error('No relevant Desktopr declaration files were found.');
	}

	const output = buildGeneratedFile(DESKTOPR_PACKAGE_DIR, declarationFiles, entryDeclaration);

	fs.mkdirSync(path.dirname(OUTPUT_FILE), { recursive: true });
	fs.writeFileSync(OUTPUT_FILE, output, 'utf8');

	console.log(
		`Generated Desktopr playground types: ${normalizeToPosix(
			path.relative(process.cwd(), OUTPUT_FILE)
		)}`
	);
	console.log(`Desktopr package version: ${packageJson.version ?? 'unknown'}`);
	console.log(
		`Entry declaration: ${normalizeToPosix(path.relative(process.cwd(), entryDeclaration))}`
	);
	console.log(`Global declaration module: ${getGlobalDeclarationModule(DESKTOPR_PACKAGE_DIR, entryDeclaration)}`);
	console.log(`Included declaration files: ${declarationFiles.length}`);
}

main();
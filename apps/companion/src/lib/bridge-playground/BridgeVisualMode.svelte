<script lang="ts">
	import { Desktopr, isDesktoprAvailable } from 'desktopr';
	import { bridgeSnippets, snippetCategories, type BridgeSnippet } from './snippets';

	type ConsoleEntry = {
		type: 'log' | 'warn' | 'error';
		values: unknown[];
	};

	type ExecutionState = {
		status: 'idle' | 'running' | 'success' | 'error';
		result: unknown;
		error: string | null;
		console: ConsoleEntry[];
		durationMs: number | null;
	};

	const categoryIds = Object.keys(snippetCategories);

	let openCategory = $state(categoryIds[0] ?? '');
	let selectedSnippet = $state<BridgeSnippet | null>(null);
	let paramValues = $state<Record<string, string | number | boolean>>({});

	let execution = $state<ExecutionState>({
		status: 'idle',
		result: null,
		error: null,
		console: [],
		durationMs: null
	});

	const snippetsByCategory = $derived(() => {
		const map: Record<string, BridgeSnippet[]> = {};
		for (const snippet of bridgeSnippets) {
			if (snippet.id === 'empty-snippet') continue;
			if (!map[snippet.category]) map[snippet.category] = [];
			map[snippet.category].push(snippet);
		}
		return map;
	});

	function toggleCategory(categoryId: string) {
		openCategory = openCategory === categoryId ? '' : categoryId;
	}

	function selectSnippet(snippet: BridgeSnippet) {
		selectedSnippet = snippet;
		paramValues = {};
		for (const param of snippet.params ?? []) {
			paramValues[param.name] = param.default;
		}
		execution = { status: 'idle', result: null, error: null, console: [], durationMs: null };
	}

	function serialize(value: unknown): string {
		if (typeof value === 'string') return value;
		try {
			return JSON.stringify(value, null, 2);
		} catch {
			return String(value);
		}
	}

	async function transpileTypeScript(source: string) {
		const ts = await import('typescript');
		const output = ts.transpileModule(source, {
			compilerOptions: {
				target: ts.ScriptTarget.ES2020,
				module: ts.ModuleKind.ESNext,
				strict: false,
				esModuleInterop: true
			}
		});
		return output.outputText;
	}

	async function runSnippet() {
		if (!selectedSnippet) return;

		const startedAt = performance.now();
		execution = { status: 'running', result: null, error: null, console: [], durationMs: null };

		const consoleEntries: ConsoleEntry[] = [];
		const playgroundConsole = {
			log: (...values: unknown[]) => consoleEntries.push({ type: 'log', values }),
			warn: (...values: unknown[]) => consoleEntries.push({ type: 'warn', values }),
			error: (...values: unknown[]) => consoleEntries.push({ type: 'error', values })
		};

		try {
			const code = selectedSnippet.buildCode
				? selectedSnippet.buildCode(paramValues)
				: selectedSnippet.code;

			const jsCode = await transpileTypeScript(code);
			const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
			const runner = new AsyncFunction(
				'Desktopr',
				'isDesktoprAvailable',
				'console',
				`"use strict";\n${jsCode}`
			);

			const result = await runner(Desktopr, isDesktoprAvailable, playgroundConsole);
			const durationMs = Math.round(performance.now() - startedAt);
			execution = { status: 'success', result, error: null, console: consoleEntries, durationMs };
		} catch (error) {
			const durationMs = Math.round(performance.now() - startedAt);
			execution = {
				status: 'error',
				result: null,
				error: error instanceof Error ? error.message : String(error),
				console: consoleEntries,
				durationMs
			};
		}
	}

	function friendlyValue(value: unknown): {
		kind: 'primitive' | 'boolean' | 'object' | 'array' | 'null';
		display: string;
		pairs?: [string, string][];
	} {
		if (value === null || value === undefined) return { kind: 'null', display: 'Done' };
		if (typeof value === 'boolean') return { kind: 'boolean', display: String(value) };
		if (typeof value === 'string' || typeof value === 'number')
			return { kind: 'primitive', display: String(value) };
		if (Array.isArray(value)) return { kind: 'array', display: serialize(value) };
		if (typeof value === 'object') {
			const pairs = Object.entries(value as Record<string, unknown>).map(
				([k, v]): [string, string] => [k, serialize(v)]
			);
			return { kind: 'object', display: serialize(value), pairs };
		}
		return { kind: 'primitive', display: serialize(value) };
	}
</script>

<div class="h-full grid gap-4 lg:grid-cols-[minmax(220px,0.4fr)_minmax(0,1fr)]">

	<!-- Left: accordion snippet browser -->
	<nav class="overflow-y-auto rounded-2xl border border-neutral-800 bg-neutral-900/70 p-3">
		<p class="mb-3 px-1 text-xs font-medium uppercase tracking-widest text-neutral-500">Modules</p>
		<div class="flex flex-col gap-1">
			{#each categoryIds as categoryId}
				{@const category = snippetCategories[categoryId]}
				{@const snippets = snippetsByCategory()[categoryId] ?? []}
				{#if snippets.length > 0}
					<!-- Category header -->
					<button
						type="button"
						onclick={() => toggleCategory(categoryId)}
						class="hover:cursor-pointer flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-sm font-semibold transition hover:bg-neutral-800/60"
						class:text-amber-400={openCategory === categoryId}
						class:text-neutral-300={openCategory !== categoryId}
					>
						<span>{category?.label ?? categoryId}</span>
						<svg
							class="h-3.5 w-3.5 shrink-0 text-neutral-500 transition-transform duration-200"
							class:rotate-180={openCategory === categoryId}
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<path d="m6 9 6 6 6-6" />
						</svg>
					</button>

					<!-- Snippet list (visible only when category is open) -->
					{#if openCategory === categoryId}
						<div class="mb-1 flex flex-col gap-0.5 pl-1">
							{#each snippets as snippet}
								<button
									type="button"
									onclick={() => selectSnippet(snippet)}
									class={[
										'hover:cursor-pointer w-full rounded-lg border px-3 py-2 text-left text-sm transition',
										selectedSnippet?.id === snippet.id
											? 'border-[#FD9A00]/40 bg-[#FD9A00]/10 text-amber-300'
											: 'border-transparent text-neutral-300 hover:bg-neutral-800'
									].join(' ')}
								>
									<span class="block font-medium leading-tight">{snippet.label}</span>
									<span class="mt-0.5 block text-xs leading-snug text-neutral-500 line-clamp-2"
										>{snippet.description}</span
									>
								</button>
							{/each}
						</div>
					{/if}
				{/if}
			{/each}
		</div>
	</nav>

	<!-- Right: detail + form + result -->
	<div class="overflow-y-auto flex flex-col gap-4 pr-0.5">
		{#if selectedSnippet}
			<!-- Snippet header + params form -->
			<div class="rounded-2xl border border-neutral-800 bg-neutral-900/70 p-6">
				<div class="mb-5 flex items-start justify-between gap-4">
					<div>
						<h2 class="text-xl font-semibold text-neutral-100">{selectedSnippet.label}</h2>
						<p class="mt-1 text-sm text-neutral-400">{selectedSnippet.description}</p>
					</div>
					<button
						type="button"
						onclick={runSnippet}
						disabled={execution.status === 'running'}
						class="hover:cursor-pointer inline-flex shrink-0 items-center justify-center gap-2 rounded-xl bg-[#FD9A00] px-5 py-2.5 text-sm font-semibold text-black transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
					>
						{#if execution.status === 'running'}
							<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24">
								<path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3c4.97 0 9 4.03 9 9">
									<animateTransform attributeName="transform" dur="1.5s" repeatCount="indefinite" type="rotate" values="0 12 12;360 12 12" />
								</path>
							</svg>
							Running…
						{:else}
							<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24">
								<path fill="currentColor" d="M6 5.912c0-.155.037-.307.107-.443c.23-.44.75-.599 1.163-.354l10.29 6.088c.14.083.255.206.332.355c.23.44.08.995-.332 1.239L7.27 18.885a.8.8 0 0 1-.415.115C6.383 19 6 18.592 6 18.089z" />
							</svg>
							Run
						{/if}
					</button>
				</div>

				{#if selectedSnippet.params && selectedSnippet.params.length > 0}
					<div class="flex flex-col gap-3">
						{#each selectedSnippet.params as param (param.name)}
							<div>
								<label
									class="mb-1.5 block text-sm font-medium text-neutral-300"
									for="param-{param.name}"
								>
									{param.label}
								</label>
								{#if param.description}
									<p class="mb-1 text-xs text-neutral-500">{param.description}</p>
								{/if}
								{#if param.type === 'textarea'}
									<textarea
										id="param-{param.name}"
										rows="3"
										placeholder={param.placeholder}
										value={String(paramValues[param.name] ?? param.default)}
										oninput={(e) => {
											paramValues[param.name] = (e.target as HTMLTextAreaElement).value;
										}}
										class="w-full resize-none rounded-xl border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm text-neutral-100 outline-none transition placeholder:text-neutral-600 focus:border-[#FD9A00] focus:ring-2 focus:ring-[#FD9A00]/20"
									></textarea>
								{:else if param.type === 'boolean'}
									<button
										id="param-{param.name}"
										type="button"
										role="switch"
										aria-checked={Boolean(paramValues[param.name] ?? param.default)}
										aria-label={param.label}
										onclick={() => {
											paramValues[param.name] = !Boolean(paramValues[param.name] ?? param.default);
										}}
										class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-[#FD9A00]/30 focus:ring-offset-2 focus:ring-offset-neutral-900"
										class:bg-[#FD9A00]={Boolean(paramValues[param.name] ?? param.default)}
										class:bg-neutral-700={!Boolean(paramValues[param.name] ?? param.default)}
									>
										<span
											class="pointer-events-none inline-block h-4 w-4 rounded-full bg-white shadow-sm transition-transform duration-200"
											class:translate-x-5={Boolean(paramValues[param.name] ?? param.default)}
											class:translate-x-0={!Boolean(paramValues[param.name] ?? param.default)}
										></span>
									</button>
								{:else}
									<input
										id="param-{param.name}"
										type={param.type === 'number' ? 'number' : 'text'}
										placeholder={param.placeholder}
										value={paramValues[param.name] ?? param.default}
										oninput={(e) => {
											const raw = (e.target as HTMLInputElement).value;
											paramValues[param.name] = param.type === 'number' ? Number(raw) : raw;
										}}
										class="w-full rounded-xl border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm text-neutral-100 outline-none transition placeholder:text-neutral-600 focus:border-[#FD9A00] focus:ring-2 focus:ring-[#FD9A00]/20"
									/>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Result panel -->
			<div class="rounded-2xl border border-neutral-800 bg-neutral-900/70 p-6">
				<div class="mb-4 flex items-center justify-between">
					<h3 class="text-sm font-semibold text-neutral-300">Result</h3>
					<div class="flex items-center gap-3">
						{#if execution.durationMs !== null}
							<span class="text-xs text-neutral-500">{execution.durationMs}ms</span>
						{/if}
						<span
							class="rounded-full px-2.5 py-0.5 text-xs font-medium"
							class:bg-neutral-800={execution.status === 'idle'}
							class:text-neutral-400={execution.status === 'idle'}
							class:bg-yellow-500={execution.status === 'running'}
							class:text-black={execution.status === 'running' || execution.status === 'success'}
							class:bg-emerald-500={execution.status === 'success'}
							class:bg-red-500={execution.status === 'error'}
							class:text-white={execution.status === 'error'}
						>{execution.status}</span>
					</div>
				</div>

				{#if execution.status === 'idle'}
					<p class="text-sm text-neutral-500">Press Run to execute this action.</p>
				{:else if execution.status === 'running'}
					<p class="text-sm text-neutral-500">Executing…</p>
				{:else if execution.status === 'error'}
					<div class="rounded-xl border border-red-900/70 bg-red-950/40 p-4">
						<p class="mb-1 text-xs font-semibold uppercase text-red-400">Error</p>
						<pre class="whitespace-pre-wrap text-sm text-red-100">{execution.error}</pre>
					</div>
				{:else}
					{@const friendly = friendlyValue(execution.result)}
					{#if friendly.kind === 'null'}
						<div class="flex items-center gap-2 text-emerald-400">
							<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
							<span class="font-medium">Completed successfully</span>
						</div>
					{:else if friendly.kind === 'boolean'}
						<div
							class="flex items-center gap-2"
							class:text-emerald-400={execution.result === true}
							class:text-red-400={execution.result === false}
						>
							{#if execution.result === true}
								<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
								<span class="font-medium">true</span>
							{:else}
								<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18M6 6l12 12"/></svg>
								<span class="font-medium">false</span>
							{/if}
						</div>
					{:else if friendly.kind === 'primitive'}
						<div class="rounded-xl border border-neutral-800 bg-neutral-950 p-4">
							<p class="wrap-break-word text-sm text-neutral-100">{friendly.display}</p>
						</div>
					{:else if friendly.kind === 'object' && friendly.pairs}
						<div class="rounded-xl border border-neutral-800 bg-neutral-950 overflow-hidden">
							{#each friendly.pairs as [key, val], i}
								<div
									class="flex gap-3 px-4 py-2.5 text-sm"
									class:border-t={i > 0}
									class:border-neutral-800={i > 0}
								>
									<span class="min-w-24 shrink-0 font-medium text-amber-400">{key}</span>
									<pre class="whitespace-pre-wrap wrap-break-word text-neutral-200 min-w-0">{val}</pre>
								</div>
							{/each}
						</div>
					{:else}
						<div class="rounded-xl border border-neutral-800 bg-neutral-950 p-4">
							<pre class="whitespace-pre-wrap wrap-break-word text-sm text-neutral-100">{friendly.display}</pre>
						</div>
					{/if}
				{/if}

				{#if execution.console.length > 0}
					<div class="mt-4">
						<p class="mb-2 text-xs font-semibold uppercase tracking-wide text-neutral-500">Console</p>
						<div class="flex flex-col gap-2">
							{#each execution.console as entry}
								<div
									class={[
										'rounded-lg border p-3 text-sm',
										entry.type === 'log' && 'border-neutral-800 bg-neutral-900',
										entry.type === 'warn' && 'border-yellow-900/60 bg-yellow-950/30',
										entry.type === 'error' && 'border-red-900/60 bg-red-950/30'
									].filter(Boolean).join(' ')}
								>
									<p
										class={[
											'mb-1 text-xs uppercase',
											entry.type === 'log' && 'text-neutral-500',
											entry.type === 'warn' && 'text-yellow-400',
											entry.type === 'error' && 'text-red-400'
										].filter(Boolean).join(' ')}
									>{entry.type}</p>
									<pre class="whitespace-pre-wrap wrap-break-word text-neutral-100">{entry.values.map(serialize).join(' ')}</pre>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		{:else}
			<div class="flex flex-1 flex-col items-center justify-center rounded-2xl border border-dashed border-neutral-800 bg-neutral-900/40 py-24 text-center">
				<div class="mb-3 text-neutral-600">
					<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z"/></svg>
				</div>
				<p class="text-sm font-medium text-neutral-400">Select a action to get started</p>
				<p class="mt-1 text-xs text-neutral-600">Expand a module on the left and select an action</p>
			</div>
		{/if}
	</div>
</div>

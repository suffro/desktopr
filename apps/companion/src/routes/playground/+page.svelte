<script lang="ts">
	import { dev } from '$app/environment';
	import { goto } from '$app/navigation';
	import { fly } from 'svelte/transition';
	import BridgeCodeEditor from '../../lib/bridge-playground/BridgeCodeEditor.svelte';
	import BridgeVisualMode from '../../lib/bridge-playground/BridgeVisualMode.svelte';
	import { bridgeSnippets } from '../../lib/bridge-playground/snippets';
	import { Desktopr, isDesktoprAvailable } from 'desktopr';
	import { wait } from '$lib';
	import { onMount } from 'svelte';

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

	let loading: boolean = $state(true);
	let isBridgeAvailable: boolean = $state(false);
	let mode: 'code' | 'visual' = $state('visual');

	let selectedSnippetId = $state(bridgeSnippets[0]?.id ?? '');
	let editorCode = $state(bridgeSnippets[0]?.code ?? '');

	let isSnippetDropdownOpen = $state(false);

	let selectedSnippet = $derived(
		bridgeSnippets.find((item) => item.id === selectedSnippetId) ?? bridgeSnippets[0]
	);

	let execution = $state<ExecutionState>({
		status: 'idle',
		result: null,
		error: null,
		console: [],
		durationMs: null
	});

	function selectSnippet(snippetId: string) {
		const snippet = bridgeSnippets.find((item) => item.id === snippetId);
		if (!snippet) return;
		selectedSnippetId = snippet.id;
		editorCode = snippet.code;
		execution = { status: 'idle', result: null, error: null, console: [], durationMs: null };
	}

	function toggleSnippetDropdown() {
		isSnippetDropdownOpen = !isSnippetDropdownOpen;
	}

	function closeSnippetDropdown() {
		isSnippetDropdownOpen = false;
	}

	function chooseSnippet(snippetId: string) {
		selectSnippet(snippetId);
		closeSnippetDropdown();
	}

	function serialize(value: unknown) {
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

	async function runCode() {
		const startedAt = performance.now();
		execution = { status: 'running', result: null, error: null, console: [], durationMs: null };

		const consoleEntries: ConsoleEntry[] = [];
		const playgroundConsole = {
			log: (...values: unknown[]) => {
				consoleEntries.push({ type: 'log', values });
			},
			warn: (...values: unknown[]) => {
				consoleEntries.push({ type: 'warn', values });
			},
			error: (...values: unknown[]) => {
				consoleEntries.push({ type: 'error', values });
			}
		};

		try {
			const jsCode = await transpileTypeScript(editorCode);
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

	onMount(async () => {
		try {
			loading = true;
			await wait(300);
			isBridgeAvailable = isDesktoprAvailable();
			if (!isBridgeAvailable) {
				if (!dev) await goto('/');
			}
		} catch (error) {
			console.error(error);
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Bridge Playground | Desktopr Companion</title>
	<meta
		name="description"
		content="Test Desktopr Bridge API snippets directly inside the Companion."
	/>
</svelte:head>

<main class="h-[calc(100vh-80px)] overflow-hidden flex flex-col bg-neutral-950 text-neutral-100">
	{#if loading}
		<div class="flex h-full w-full items-center justify-center">
			<div
				class="h-8 w-8 animate-spin rounded-full border-2 border-slate-700 border-t-[#FD9A00]"
			></div>
		</div>
	{:else if isBridgeAvailable || dev}
		<div class="flex-1 min-h-0 flex flex-col gap-4 px-4 py-4 md:px-8 mx-auto w-full max-w-7xl">
			<!-- Header -->
			<header class="shrink-0 flex flex-col gap-3">
				<div class="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
					<div>
						<div class="flex gap-4 items-center">
							<h1 class="mt-1 text-3xl font-semibold tracking-tight">Bridge Playground</h1>
							{#if mode==="code"}
								<div transition:fly={{ duration: 200 }} class="mt-2 flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
									<div
										class="px-3 py-1 flex justify-center items-center gap-2 font-semibold uppercase bg-[#FD9A00]/18 text-[#FD9A00] rounded-lg max-w-fit text-xs"
									>
										Run trusted code only
									</div>
								</div>
							{/if}
						</div>
						<p class="mt-2 max-w-2xl text-sm text-neutral-400">
							Test Desktopr Bridge API methods and inspect responses in real time.
						</p>
					</div>

					<!-- Mode toggle -->
					<div
						class="flex shrink-0 items-center gap-1 rounded-xl border border-neutral-800 bg-neutral-900 p-1 self-start"
					>
						<button
							type="button"
							onclick={() => (mode = 'visual')}
							class="flex hover:cursor-pointer items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition"
							class:bg-[#FD9A00]={mode === 'visual'}
							class:text-neutral-900={mode === 'visual'}
							class:text-neutral-500={mode !== 'visual'}
							class:hover:text-neutral-300={mode !== 'visual'}
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								width="14"
								height="14"
								viewBox="0 0 24 24"
								fill="black"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								><rect width="7" height="7" x="3" y="3" rx="1" /><rect
									width="7"
									height="7"
									x="14"
									y="3"
									rx="1"
								/><rect width="7" height="7" x="14" y="14" rx="1" /><rect
									width="7"
									height="7"
									x="3"
									y="14"
									rx="1"
								/></svg
							>
							Visual
						</button>
						<button
							type="button"
							onclick={() => (mode = 'code')}
							class="flex hover:cursor-pointer items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition"
							class:bg-[#FD9A00]={mode === 'code'}
							class:text-neutral-900={mode === 'code'}
							class:text-neutral-500={mode !== 'code'}
							class:hover:text-neutral-300={mode !== 'code'}
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								width="14"
								height="14"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								><polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" /></svg
							>
							Code
						</button>
					</div>
				</div>
			</header>

			<!-- Visual mode -->
			{#if mode === 'visual'}
				<div class="flex-1 min-h-0">
					<BridgeVisualMode />
				</div>
			{/if}

			<!-- Code mode (kept mounted to preserve editor state) -->
			<div
				class="flex-1 min-h-0 grid gap-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(360px,0.85fr)]"
				class:hidden={mode !== 'code'}
			>
				<!-- Editor panel -->
				<div
					class="flex flex-col rounded-2xl border border-neutral-800 bg-neutral-900/70 p-4 overflow-hidden"
				>
					<div
						class="mb-4 shrink-0 flex flex-col gap-3 md:flex-row md:items-center md:justify-between"
					>
						<div class="flex flex-col w-full justify-start gap-3 items-start">
							<!-- <div>
								<p id="snippet-label" class="text-sm font-medium text-neutral-200">Editor</p>
								<p class="mt-1 text-xs text-neutral-500">
									Choose a prebuilt example or edit the code manually.
								</p>
							</div> -->

							<div
								class="relative w-full"
								onfocusout={(event) => {
									const nextTarget = event.relatedTarget as Node | null;
									if (!nextTarget || !event.currentTarget.contains(nextTarget))
										closeSnippetDropdown();
								}}
							>
								<button
									id="snippet"
									type="button"
									aria-haspopup="listbox"
									aria-expanded={isSnippetDropdownOpen}
									aria-labelledby="snippet-label snippet"
									onclick={toggleSnippetDropdown}
									onkeydown={(event) => {
										if (event.key === 'Escape') {
											closeSnippetDropdown();
											return;
										}
										if (event.key === 'ArrowDown' || event.key === 'Enter' || event.key === ' ') {
											event.preventDefault();
											isSnippetDropdownOpen = true;
										}
									}}
									class="flex w-full h-10 min-w-65 items-center justify-between gap-3 rounded-xl border border-neutral-800 bg-neutral-950 px-3 text-left text-sm font-medium text-neutral-100 outline-none transition hover:border-neutral-600 focus:border-[#FD9A00] focus:ring-2 focus:ring-[#FD9A00]/20"
								>
									<span class="flex flex-col gap-0.5 w-full truncate">
										<!-- <span class="text-sm mr-2 font-light text-neutral-500"
											>Snippet:
										</span> -->
										{(selectedSnippet?.label) ?? 'Choose snippet'}
										
										<!-- <span class="font-light text-neutral-500 text-xs"
											>{(selectedSnippet?.description) ?? ''}
										</span> -->
									</span>
									<svg
										class="h-4 w-4 shrink-0 text-neutral-400 transition-transform"
										class:rotate-180={isSnippetDropdownOpen}
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"
										stroke-linecap="round"
										stroke-linejoin="round"
										aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg
									>
								</button>

								{#if isSnippetDropdownOpen}
									<div
										role="listbox"
										aria-labelledby="snippet-label"
										tabindex="-1"
										class="absolute right-0 z-50 mt-2 max-h-72 w-full min-w-65 overflow-y-auto rounded-xl border border-neutral-800 bg-neutral-950 p-1 shadow-2xl shadow-black/40"
									>
										{#each bridgeSnippets as snippet}
											<button
												type="button"
												role="option"
												aria-selected={snippet.id === selectedSnippetId}
												onclick={() => chooseSnippet(snippet.id)}
												class="flex w-full items-start justify-between gap-3 rounded-lg px-3 py-2 text-left text-sm transition hover:bg-neutral-800/80 focus:bg-neutral-800/80 focus:outline-none"
												class:bg-neutral-800={snippet.id === selectedSnippetId}
												class:text-amber-400={snippet.id === selectedSnippetId}
												class:text-neutral-200={snippet.id !== selectedSnippetId}
											>
												<span class="min-w-0">
													<span class="block truncate font-medium">{snippet.label}</span>
													{#if snippet.description}
														<span class="mt-0.5 block truncate text-xs text-neutral-500"
															>{snippet.description}</span
														>
													{/if}
												</span>
												{#if snippet.id === selectedSnippetId}
													<svg
														class="mt-0.5 h-4 w-4 shrink-0 text-[#FD9A00]"
														xmlns="http://www.w3.org/2000/svg"
														viewBox="0 0 24 24"
														fill="none"
														stroke="currentColor"
														stroke-width="2"
														stroke-linecap="round"
														stroke-linejoin="round"
														aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg
													>
												{/if}
											</button>
										{/each}
									</div>
								{/if}
							</div>
						</div>
						<div class="h-full flex items-end">
							<button
								type="button"
								onclick={runCode}
								disabled={execution.status === 'running'}
								class="inline-flex items-center hover:cursor-pointer justify-center rounded-xl bg-[#FD9A00] px-5 h-10 text-sm font-semibold text-black transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
							>
								{#if execution.status != 'running'}
									<svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
										><path d="M0 0h24v24H0z" fill="none" /><path
											fill="currentColor"
											d="M6 5.912c0-.155.037-.307.107-.443c.23-.44.75-.599 1.163-.354l10.29 6.088c.14.083.255.206.332.355c.23.44.08.995-.332 1.239L7.27 18.885a.8.8 0 0 1-.415.115C6.383 19 6 18.592 6 18.089z"
										/></svg
									>
								{:else}
									<svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24"
										><path d="M0 0h24v24H0z" fill="none" /><path
											fill="none"
											stroke="currentColor"
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M12 3c4.97 0 9 4.03 9 9"
											><animateTransform
												attributeName="transform"
												dur="1.5s"
												repeatCount="indefinite"
												type="rotate"
												values="0 12 12;360 12 12"
											/></path
										></svg
									>
								{/if}
								<span class="ml-2">{execution.status === 'running' ? 'Running...' : 'Run'}</span>
							</button>
						</div>
					</div>

					<!-- Editor fills remaining height -->
					<div class="flex-1 min-h-0">
						<BridgeCodeEditor
							value={editorCode}
							language="typescript"
							onChange={(value: any) => {
								editorCode = value;
							}}
						/>
					</div>
				</div>

				<!-- Results panel: scrolls internally -->
				<aside
					class="flex flex-col gap-4 rounded-2xl border border-neutral-800 bg-neutral-900/70 p-4 overflow-y-auto"
				>
					<div class="flex items-center justify-between border-b border-neutral-800 pb-3 shrink-0">
						<div>
							<h2 class="text-lg font-semibold">Execution result</h2>
							<p class="mt-1 text-xs text-neutral-500">
								Result, errors and captured console output.
							</p>
						</div>
						<span
							class="rounded-full px-3 py-1 text-xs font-medium"
							class:bg-neutral-800={execution.status === 'idle'}
							class:bg-yellow-500={execution.status === 'running'}
							class:bg-emerald-500={execution.status === 'success'}
							class:bg-red-500={execution.status === 'error'}
							class:text-black={execution.status === 'running' || execution.status === 'success'}
							class:text-white={execution.status === 'idle' || execution.status === 'error'}
							>{execution.status}</span
						>
					</div>

					{#if execution.durationMs !== null}
						<p class="text-xs text-neutral-500 shrink-0">Completed in {execution.durationMs}ms</p>
					{/if}

					{#if execution.error}
						<div class="rounded-xl border border-red-900/70 bg-red-950/40 p-4 shrink-0">
							<h3 class="text-sm font-semibold text-red-300">Error</h3>
							<pre class="mt-2 whitespace-pre-wrap text-sm text-red-100">{execution.error}</pre>
						</div>
					{/if}

					<div class="rounded-xl border border-neutral-800 bg-neutral-950 p-4 shrink-0">
						<h3 class="mb-3 text-sm font-semibold text-neutral-300">Return value</h3>
						{#if execution.status === 'idle'}
							<p class="text-sm text-neutral-500">Run a snippet to see the response here.</p>
						{:else if execution.status === 'running'}
							<p class="text-sm text-neutral-500">Executing...</p>
						{:else}
							<pre class="whitespace-pre-wrap wrap-break-word text-sm text-neutral-100">{serialize(
									execution.result
								)}</pre>
						{/if}
					</div>

					<div class="rounded-xl border border-neutral-800 bg-neutral-950 p-4 shrink-0">
						<h3 class="mb-3 text-sm font-semibold text-neutral-300">Console</h3>
						{#if execution.console.length === 0}
							<p class="text-sm text-neutral-500">No console output.</p>
						{:else}
							<div class="flex flex-col gap-3">
								{#each execution.console as entry}
									<div class="rounded-lg border border-neutral-800 bg-neutral-900 p-3">
										<p class="mb-1 text-xs uppercase text-neutral-500">{entry.type}</p>
										<pre
											class="whitespace-pre-wrap wrap-break-word text-sm text-neutral-100">{entry.values
												.map(serialize)
												.join(' ')}</pre>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				</aside>
			</div>
		</div>
	{:else}
		<div></div>
	{/if}
</main>

<script lang="ts">
	import { goto } from '$app/navigation';
	import { COMPANION_URL_GLOBAL_VAR_KEY, isHttpUrl, normalizeUrl, wait } from '$lib';
	import { Desktopr, isDesktoprAvailable } from 'desktopr';
	import { onMount } from 'svelte';

	let loading: boolean = $state(true);
	let isBridgeAvailable: boolean = $state(false);
	let url: string = $state('');
	let fullUrl: string = $derived(normalizeUrl(url));

	$effect(() => {
		if (url && fullUrl && url != fullUrl) url = fullUrl;
	});

	// Loads the web app in the main window so it can use the whole bridge.
	const loadWebApp = async (event: SubmitEvent) => {
		event.preventDefault();
		loading = true;
		try {
			const target = fullUrl.trim();
			if (!isHttpUrl(target)) return alert('Invalid URL');
			await Desktopr.globalVariables.set(COMPANION_URL_GLOBAL_VAR_KEY, target);
			location.href = target;
		} catch (error) {
			console.error(error);
			alert('Something went wrong');
		} finally {
			loading = false;
		}
	};

	onMount(async () => {
		try {
			loading = true;
			await wait(300);
			isBridgeAvailable = isDesktoprAvailable();
			if (isBridgeAvailable) {
				const savedUrl = await Desktopr.globalVariables.get(COMPANION_URL_GLOBAL_VAR_KEY);
				if (isHttpUrl(savedUrl)) url = savedUrl;
			}
		} catch (error) {
			console.error(error);
		} finally {
			loading = false;
		}
	});
</script>

<main class="flex-1 flex items-center justify-center px-4 bg-neutral-800/10">
	{#if loading}
		<div class="flex items-center justify-center h-full w-full">
			<div
				class="h-8 w-8 animate-spin rounded-full border-2 border-slate-700 border-t-[#FD9A00]"
			></div>
		</div>
	{:else if isBridgeAvailable}
	<div class="flex flex-col space-y-2 justify-center items-center max-md:w-70 max-lg:w-80 lg:w-95">
		<form class="w-full max-w-md flex flex-col overflow-hidden" onsubmit={loadWebApp}>
			<!-- <label for="url" class="text-neutral-300/80 text-center mb-2">
						Enter your app url to launch companion:
					</label> -->
			<input
				bind:value={url}
				id="url"
				type="url"
				required
				placeholder="eg. https://your-app.com or localhost:3000"
				class="w-full rounded-t-xl text-center border placeholder:text-neutral-500 border-neutral-700 border-b-neutral-800 bg-neutral-900 px-4 py-4 text-sm text-neutral-100
						transition focus:border-[#FD9A00] ring-none outline-none overflow-hidden"
			/>
			<button
				type="submit"
				class="px-4 py-2 border w-full {url?.trim()
					? 'bg-[#FD9A00] border-[#FD9A00] hover:cursor-pointer hover:bg-[#FD9A00]/90 text-neutral-800'
					: ' bg-neutral-800 border-neutral-800 text-neutral-600'} border-t-0 text-sm font-medium rounded-b-xl"
			>
				Load web app
			</button>
		</form>
		
		<p class="text-xs opacity-30 font-light my-6">or test the Desktopr bridge API directly</p>

			<button
				onclick={()=>goto("playground")}
				class="px-4 py-2 border w-full bg-[#FD9A00] border-[#FD9A00] hover:cursor-pointer hover:bg-[#FD9A00]/90 text-neutral-800 text-sm font-medium rounded-xl"
			>
				Bridge playground
			</button>
		</div>
	{:else}
		<p class="text-sm text-neutral-400 text-center max-w-sm">
			The Desktopr bridge is not available. Open this page inside the Desktopr Companion app.
		</p>
	{/if}
</main>


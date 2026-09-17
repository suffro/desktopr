<script lang="ts">
	// Import global styles
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { superLoading } from '$lib/_stores';
	import { onMount } from 'svelte';
	import { isDesktoprAvailable } from 'desktopr';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { dev } from '$app/environment';

	let { children } = $props();
	let isBridgeAvailable: boolean = $state(false);

	onMount(() => {
		isBridgeAvailable = isDesktoprAvailable();
		console.log(
			isBridgeAvailable ? '[Desktopr bridge detected]' : '[Desktopr bridge not available]'
		);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if $superLoading}
	<div class="flex items-center justify-center h-full w-full">
		<div
			class="h-8 w-8 animate-spin rounded-full border-2 border-slate-700 border-t-[#FD9A00]"
		></div>
	</div>
{:else}
	<header
		class="h-20 px-8 flex items-center justify-between border-b border-neutral-800 text-lg font-semibold bg-neutral-800/10"
	>
		<div class="flex items-center justify-start">
			<span class="relative mr-4 inline-flex h-[30px] w-[30px] items-center justify-center">
				<img src="/icons/logo.png" alt="" />
			</span>
			<span class="flex flex-col items-start justify-center space-y-0 gap-0 -mb-1">
				<span class="text-xs text-[#FD9A00] w-full -mb-0.5 font-light text-left">Desktopr</span>
				<span class="-mt-0.5">Companion {dev?"[dev]":""}</span>
			</span>
		</div>
		<div>
			{#if isBridgeAvailable || dev}
				{#if page?.route.id != '/playground'}
					<button
						onclick={() => goto('playground')}
						class="px-4 py-2 mr-2 text-xs bg-neutral-800 hover:cursor-pointer hover:bg-[#FD9A00] hover:text-neutral-800 text-neutral-400 font-medium rounded-md"
					>
						Playground
					</button>
				{:else}
					<button
						onclick={() => goto('/')}
						class="px-4 py-2 mr-2 text-xs bg-neutral-800 hover:cursor-pointer hover:bg-[#FD9A00] hover:text-neutral-800 text-neutral-400 font-medium rounded-md"
					>
						Back
					</button>
				{/if}
			{/if}
		</div>
	</header>
	{@render children()}

	<footer
		class="h-20 w-full border-t border-neutral-800 flex items-center justify-center text-xs text-neutral-400"
	>
		<div class="opacity-60 w-full flex items-center justify-center px-8">
			Desktopr Companion · Apache-2.0
		</div>
	</footer>
{/if}

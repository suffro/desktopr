<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import type { CompletionContext, CompletionResult } from '@codemirror/autocomplete';

	type Props = {
		value: string;
		language?: 'javascript' | 'typescript';
		onChange?: (value: string) => void;
	};

	type TsCompletionItem = {
		label: string;
		type: string;
		detail?: string;
		info?: string;
		apply?: string;
	};

	type TsWorkerApi = {
		initialize: () => Promise<boolean>;
		getCompletions: (code: string, position: number) => Promise<TsCompletionItem[]>;
	};

	let { value, language = 'typescript', onChange }: Props = $props();

	let editorHost: HTMLDivElement;
	let view = $state<import('@codemirror/view').EditorView | null>(null);

	let isApplyingExternalValue = false;
	let lastEditorValue = $state(value);
	let innerWorker: Worker | null = null;
	let tsWorker: import('comlink').Remote<TsWorkerApi> | null = null;

	function createDesktoprCompletionSource() {
	return async (context: CompletionContext): Promise<CompletionResult | null> => {
		if (!tsWorker) return null;

		const beforeCursor = context.state.doc.sliceString(0, context.pos);
		const match = beforeCursor.match(/(?:Desktopr(?:\.[A-Za-z_$][\w$]*)*)\.([A-Za-z_$][\w$]*)?$/);
		const word = context.matchBefore(/[\w$]+$/);

		if (!match && !word && !context.explicit) {
			return null;
		}

		const partial = match ? (match[1] ?? '') : (word?.text ?? '');
		const from = context.pos - partial.length;

		try {
			const code = context.state.doc.toString();
			const options = await tsWorker.getCompletions(code, context.pos);

			if (!options.length) return null;

			return {
				from,
				options,
				validFor: /^[\w$]*$/
			};
		} catch (error) {
			console.error('Desktopr playground autocomplete failed.', error);
			return null;
		}
	};
}

	onMount(async () => {
		const { EditorView, basicSetup } = await import('codemirror');
		const { EditorState } = await import('@codemirror/state');
		const { javascript } = await import('@codemirror/lang-javascript');
		const { HighlightStyle, syntaxHighlighting } = await import('@codemirror/language');
		const { tags } = await import('@lezer/highlight');
		const { autocompletion } = await import('@codemirror/autocomplete');

		const desktoprDarkTheme = EditorView.theme(
			{
				'&': {
					height: '100%',
					backgroundColor: '#0a0a0a',
					color: '#e5e5e5',
					fontSize: '14px'
				},
				'.cm-editor': {
					height: '100%'
				},
				'.cm-scroller': {
					fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
					backgroundColor: '#0a0a0a'
				},
				'.cm-content': {
					minHeight: '100%',
					caretColor: '#fd9a00'
				},
				'.cm-focused': {
					outline: 'none'
				},
				'.cm-cursor': {
					borderLeftColor: '#fd9a00'
				},
				'.cm-selectionBackground, &.cm-focused .cm-selectionBackground': {
					backgroundColor: 'rgba(253, 154, 0, 0.22)'
				},
				'.cm-gutters': {
					backgroundColor: '#111111',
					color: '#737373',
					borderRight: '1px solid #262626'
				},
				'.cm-activeLine': {
					backgroundColor: 'rgba(255, 255, 255, 0.035)'
				},
				'.cm-activeLineGutter': {
					backgroundColor: '#171717',
					color: '#d4d4d4'
				},
				'.cm-lineNumbers .cm-gutterElement': {
					padding: '0 12px 0 8px'
				},
				'.cm-foldGutter': {
					backgroundColor: '#111111'
				},
				'.cm-foldGutter .cm-gutterElement': {
					color: '#737373'
				},
				'.cm-tooltip': {
					backgroundColor: '#171717',
					border: '1px solid #404040',
					color: '#e5e5e5'
				},
				'.cm-tooltip.cm-tooltip-autocomplete': {
					backgroundColor: '#171717',
					border: '1px solid #404040',
					color: '#e5e5e5'
				},
				'.cm-tooltip-autocomplete ul': {
					backgroundColor: '#171717'
				},
				'.cm-tooltip-autocomplete ul li': {
					color: '#d4d4d4',
					padding: '6px 10px'
				},
				'.cm-tooltip-autocomplete ul li[aria-selected]': {
					backgroundColor: 'rgba(253, 154, 0, 0.18)',
					color: '#fbbf24'
				},
				'.cm-completionLabel': {
					color: '#e5e5e5'
				},
				'.cm-completionDetail': {
					color: '#a3a3a3'
				},
				'.cm-completionInfo': {
					backgroundColor: '#171717',
					border: '1px solid #404040',
					color: '#d4d4d4'
				}
			},
			{
				dark: true
			}
		);

		const desktoprHighlightStyle = HighlightStyle.define([
			{
				tag: tags.keyword,
				color: '#fbbf24'
			},
			{
				tag: [tags.name, tags.deleted, tags.character, tags.propertyName, tags.macroName],
				color: '#93c5fd'
			},
			{
				tag: [tags.function(tags.variableName), tags.labelName],
				color: '#fde68a'
			},
			{
				tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)],
				color: '#fca5a5'
			},
			{
				tag: [tags.definition(tags.name), tags.separator],
				color: '#e5e5e5'
			},
			{
				tag: [tags.typeName, tags.className],
				color: '#86efac'
			},
			{
				tag: [tags.number, tags.changed, tags.annotation, tags.modifier, tags.self, tags.namespace],
				color: '#fdba74'
			},
			{
				tag: [tags.string, tags.special(tags.brace)],
				color: '#bef264'
			},
			{
				tag: [tags.regexp, tags.escape, tags.link],
				color: '#67e8f9'
			},
			{
				tag: [tags.operator, tags.operatorKeyword],
				color: '#f9a8d4'
			},
			{
				tag: [tags.meta, tags.comment],
				color: '#a3a3a3'
			},
			{
				tag: tags.strong,
				fontWeight: 'bold'
			},
			{
				tag: tags.emphasis,
				fontStyle: 'italic'
			},
			{
				tag: tags.strikethrough,
				textDecoration: 'line-through'
			},
			{
				tag: tags.link,
				textDecoration: 'underline'
			},
			{
				tag: tags.invalid,
				color: '#fecaca'
			}
		]);

		try {
			const Comlink = await import('comlink');

			innerWorker = new Worker(new URL('./typescript.worker.ts', import.meta.url), {
				type: 'module'
			});

			tsWorker = Comlink.wrap<TsWorkerApi>(innerWorker);
			await tsWorker.initialize();
		} catch (error) {
			console.error('Desktopr playground TypeScript worker failed to start.', error);

			innerWorker?.terminate();
			innerWorker = null;
			tsWorker = null;
		}

		view = new EditorView({
			parent: editorHost,
			state: EditorState.create({
				doc: value,
				extensions: [
					basicSetup,
					javascript({
						typescript: language === 'typescript'
					}),
					desktoprDarkTheme,
					syntaxHighlighting(desktoprHighlightStyle),
					autocompletion({
						override: [createDesktoprCompletionSource()],
						activateOnTyping: true
					}),
					EditorView.updateListener.of((update) => {
						if (!update.docChanged) return;

						const nextValue = update.state.doc.toString();
						lastEditorValue = nextValue;

						if (!isApplyingExternalValue) {
							onChange?.(nextValue);
						}
					})
				]
			})
		});

		lastEditorValue = value;
	});

	$effect(() => {
		if (!view) return;

		const nextValue = value ?? '';

		if (nextValue === lastEditorValue) return;

		isApplyingExternalValue = true;

		view.dispatch({
			changes: {
				from: 0,
				to: view.state.doc.length,
				insert: nextValue
			}
		});

		lastEditorValue = nextValue;
		isApplyingExternalValue = false;
	});

	onDestroy(() => {
		view?.destroy();
		view = null;

		innerWorker?.terminate();
		innerWorker = null;
		tsWorker = null;
	});
</script>

<div
	bind:this={editorHost}
	class="h-full w-full overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950"
></div>
import type { Completion, CompletionContext, CompletionResult } from '@codemirror/autocomplete';

const desktoprCompletions: Completion[] = [
	{
		label: 'Desktopr',
		type: 'variable',
		detail: 'Desktopr Bridge API',
		info: 'Main Desktopr Bridge API object.'
	},
	{
		label: 'isDesktoprAvailable',
		type: 'function',
		detail: '() => boolean',
		apply: 'isDesktoprAvailable()',
		info: 'Checks whether the Desktopr Bridge API is available.'
	},

	// Notifications
	{
		label: 'Desktopr.notifications',
		type: 'namespace',
		detail: 'Notifications module'
	},
	{
		label: 'Desktopr.notifications.send',
		type: 'function',
		detail: '(options) => Promise<unknown>',
		apply: `Desktopr.notifications.send({
  title: 'Hello from Desktopr',
  body: 'Notification body'
})`,
		info: 'Sends a native system notification.'
	},

	// Clipboard
	{
		label: 'Desktopr.clipboard',
		type: 'namespace',
		detail: 'Clipboard module'
	},
	{
		label: 'Desktopr.clipboard.writeText',
		type: 'function',
		detail: '(text: string) => Promise<unknown>',
		apply: `Desktopr.clipboard.writeText('Hello from Desktopr')`,
		info: 'Writes text to the system clipboard.'
	},
	{
		label: 'Desktopr.clipboard.readText',
		type: 'function',
		detail: '() => Promise<string>',
		apply: 'Desktopr.clipboard.readText()',
		info: 'Reads text from the system clipboard.'
	},

	// Window
	{
		label: 'Desktopr.window',
		type: 'namespace',
		detail: 'Window module'
	},
	{
		label: 'Desktopr.window.new',
		type: 'function',
		detail: '(options?) => Promise<unknown>',
		apply: 'Desktopr.window.new()',
		info: 'Creates or opens a Desktopr native window.'
	}
];

export function desktoprCompletionSource(context: CompletionContext): CompletionResult | null {
	const word = context.matchBefore(/[\w.$]+/);

	if (!word) return null;

	if (word.from === word.to && !context.explicit) {
		return null;
	}

	return {
		from: word.from,
		options: desktoprCompletions,
		validFor: /^[\w.$]*$/
	};
}
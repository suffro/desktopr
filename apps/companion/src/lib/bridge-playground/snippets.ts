export type SnippetParam = {
	name: string;
	label: string;
	type: 'text' | 'number' | 'textarea' | 'boolean';
	default: string | number | boolean;
	placeholder?: string;
	description?: string;
};

export type BridgeSnippet = {
	id: string;
	label: string;
	description: string;
	category: string;
	code: string;
	params?: SnippetParam[];
buildCode?: (params: Record<string, string | number | boolean>) => string;
};

export const snippetCategories: Record<string, { label: string }> = {
	app: { label: 'App & System' },
	window: { label: 'Window' },
	notifications: { label: 'Notifications' },
	clipboard: { label: 'Clipboard' },
	filesystem: { label: 'File System' },
	files: { label: 'File Dialogs' },
	events: { label: 'Events' },
	network: { label: 'Network' },
	shortcuts: { label: 'Shortcuts' },
	badge: { label: 'Badge' },
	autostart: { label: 'Autostart' },
	diagnostics: { label: 'Diagnostics' },
	menu: { label: 'Menu' },
	plugins: { label: 'Plugins' }
};

export const bridgeSnippets: BridgeSnippet[] = [
	{
		id: 'empty-snippet',
		label: 'Custom code',
		description: 'Write your custom code.',
		category: 'app',
		code: `if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');

// your code here (eg. return await Desktopr.app.info());
`
	},
	{
		id: 'check-ready',
		label: 'Check Desktopr availability',
		description: 'Checks if the Desktopr bridge is available in the current environment.',
		category: 'app',
		code: `return isDesktoprAvailable();`
	},
	{
		id: 'app-info',
		label: 'Get app info',
		description: 'Returns general information about the current Desktopr app instance.',
		category: 'app',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.app.info();`
	},
	{
		id: 'open-window',
		label: 'Open new window',
		description: 'Opens a new native Desktopr window.',
		category: 'window',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.window.new();`
	},
	{
		id: 'send-notification',
		label: 'Send notification',
		description: 'Sends a native system notification.',
		category: 'notifications',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

const permissionState = await Desktopr.notifications.state();
console.log(permissionState);

if (permissionState !== 'granted') {
  await Desktopr.notifications.request();
}

return await Desktopr.notifications.show(
  'Hello from Desktopr',
  'This notification was triggered from the Companion playground.'
);`,
		params: [
			{
				name: 'title',
				label: 'Title',
				type: 'text',
				default: 'Hello from Desktopr',
				placeholder: 'Notification title'
			},
			{
				name: 'body',
				label: 'Body',
				type: 'text',
				default: 'This notification was triggered from the Companion playground.',
				placeholder: 'Notification body'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
const permissionState = await Desktopr.notifications.state();
if (permissionState !== 'granted') await Desktopr.notifications.request();
return await Desktopr.notifications.show(${JSON.stringify(p.title)}, ${JSON.stringify(p.body)});`
	},
	{
		id: 'clipboard-write',
		label: 'Write clipboard',
		description: 'Writes text to the system clipboard.',
		category: 'clipboard',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.clipboard.writeText('Hello from Desktopr Companion');`,
		params: [
			{
				name: 'text',
				label: 'Text to copy',
				type: 'text',
				default: 'Hello from Desktopr Companion',
				placeholder: 'Text to write to clipboard'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.clipboard.writeText(${JSON.stringify(p.text)});`
	},
	{
		id: 'clipboard-read',
		label: 'Read clipboard',
		description: 'Reads text from the system clipboard.',
		category: 'clipboard',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.clipboard.readText();`
	},
	{
		id: 'fs-cache-write-read',
		label: 'Write/read cache file',
		description: 'Writes a temporary file in the isolated cache scope and reads it back.',
		category: 'filesystem',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

const filePath = 'playground/hello.txt';

await Desktopr.fs.cache.writeText(filePath, 'Hello from Desktopr cache!', {
  createDirs: true
});

const content = await Desktopr.fs.cache.readText(filePath);

return {
  filePath,
  content
};`,
		params: [
			{
				name: 'filePath',
				label: 'File path',
				type: 'text',
				default: 'playground/hello.txt',
				placeholder: 'relative/path/to/file.txt'
			},
			{
				name: 'content',
				label: 'File content',
				type: 'textarea',
				default: 'Hello from Desktopr cache!',
				placeholder: 'Text content to write'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
const filePath = ${JSON.stringify(p.filePath)};
await Desktopr.fs.cache.writeText(filePath, ${JSON.stringify(p.content)}, { createDirs: true });
const content = await Desktopr.fs.cache.readText(filePath);
return { filePath, content };`
	},
	{
		id: 'fs-cache-list',
		label: 'List cache files',
		description: 'Lists files and folders in the isolated cache scope.',
		category: 'filesystem',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.fs.cache.listContent('');`
	},
	{
		id: 'fs-data-paths',
		label: 'Get FS paths',
		description: 'Returns the base paths used by the isolated Desktopr file system.',
		category: 'filesystem',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.fs.paths();`
	},
	{
		id: 'files-open-paths',
		label: 'Open file dialog',
		description: 'Opens a native file picker and returns selected file paths.',
		category: 'files',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.files.open({
  multi: true,
  allowed: ['.txt', '.md', '.json'],
  maxBytes: 5 * 1024 * 1024
});`
	},
	{
		id: 'files-open-with-bytes',
		label: 'Open file with bytes',
		description: 'Opens a native file picker and returns selected file bytes.',
		category: 'files',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

const result = await Desktopr.files.openWithBytes({
  multi: false,
  allowed: ['.txt', '.md', '.json'],
  maxBytes: 2 * 1024 * 1024
});

return {
  files: result.files.map((file) => ({
    path: file.path,
    bytesLength: file.bytes.length
  }))
};`
	},
	{
		id: 'files-save-dialog',
		label: 'Open save dialog',
		description: 'Opens a native save dialog with a suggested file name.',
		category: 'files',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.files.save('desktopr-playground.txt');`,
		params: [
			{
				name: 'filename',
				label: 'Suggested file name',
				type: 'text',
				default: 'desktopr-playground.txt',
				placeholder: 'filename.txt'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.files.save(${JSON.stringify(p.filename)});`
	},
	{
		id: 'events-emit-listen',
		label: 'Emit and listen event',
		description: 'Registers a temporary event listener and emits a custom event.',
		category: 'events',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

const eventName = 'playground:test-event';

const unsubscribe = await Desktopr.events.on(eventName, (payload) => {
  console.log('Received event payload:', payload);
});

await Desktopr.events.emit(eventName, {
  message: 'Hello from the Bridge Playground',
  createdAt: new Date().toISOString()
});

unsubscribe();

return 'Event emitted and listener removed.';`,
		params: [
			{
				name: 'eventName',
				label: 'Event name',
				type: 'text',
				default: 'playground:test-event',
				placeholder: 'namespace:event-name'
			},
			{
				name: 'message',
				label: 'Message payload',
				type: 'text',
				default: 'Hello from the Bridge Playground',
				placeholder: 'Message to include in the event payload'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
const eventName = ${JSON.stringify(p.eventName)};
const unsubscribe = await Desktopr.events.on(eventName, (payload) => {
  console.log('Received event payload:', payload);
});
await Desktopr.events.emit(eventName, {
  message: ${JSON.stringify(p.message)},
  createdAt: new Date().toISOString()
});
unsubscribe();
return 'Event emitted and listener removed.';`
	},
	{
		id: 'events-listen-network',
		label: 'Listen network status',
		description: 'Registers a network status listener and starts network monitoring.',
		category: 'network',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

const unsubscribe = await Desktopr.events.onNetworkStatus((status) => {
  console.log('Network status updated:', status);
});

await Desktopr.network.setMonitor(10000, ['https://example.com']);

return {
  message: 'Network monitor started. Check console output for updates.',
  cleanup: 'Run Desktopr.network.stopMonitor() and call the unsubscribe function when no longer needed.'
};`
	},
	{
		id: 'network-status',
		label: 'Get network status',
		description: 'Returns the current network status from the Desktopr network module.',
		category: 'network',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.network.status();`
	},
	{
		id: 'network-ping',
		label: 'Ping URL',
		description: 'Runs a network ping/check against a URL.',
		category: 'network',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.network.ping('https://example.com', 5000);`,
		params: [
			{
				name: 'url',
				label: 'URL to ping',
				type: 'text',
				default: 'https://example.com',
				placeholder: 'https://example.com'
			},
			{
				name: 'timeout',
				label: 'Timeout (ms)',
				type: 'number',
				default: 5000,
				placeholder: '5000'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.network.ping(${JSON.stringify(p.url)}, ${Number(p.timeout)});`
	},
	{
		id: 'network-resolve',
		label: 'Resolve host',
		description: 'Resolves a hostname through the Desktopr network module.',
		category: 'network',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.network.resolve('example.com');`,
		params: [
			{
				name: 'hostname',
				label: 'Hostname',
				type: 'text',
				default: 'example.com',
				placeholder: 'example.com'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.network.resolve(${JSON.stringify(p.hostname)});`
	},
	{
		id: 'network-monitor-stop',
		label: 'Stop network monitor',
		description: 'Stops the ongoing Desktopr network monitor, if one is running.',
		category: 'network',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.network.stopMonitor();`
	},
	{
		id: 'shortcut-is-registered',
		label: 'Check shortcut',
		description: 'Checks if a global shortcut is currently registered.',
		category: 'shortcuts',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.globalShortcut.isRegistered('Ctrl+Shift+X');`,
		params: [
			{
				name: 'accelerator',
				label: 'Shortcut',
				type: 'text',
				default: 'Ctrl+Shift+X',
				placeholder: 'Ctrl+Shift+X',
				description: 'e.g. Ctrl+Shift+X or Cmd+Alt+Y'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.globalShortcut.isRegistered(${JSON.stringify(p.accelerator)});`
	},
	{
		id: 'shortcut-register-unregister',
		label: 'Register shortcut briefly',
		description: 'Registers a global shortcut and unregisters it after a short delay.',
		category: 'shortcuts',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

const accelerator = 'Ctrl+Shift+X';

await Desktopr.globalShortcut.register(
  accelerator,
  () => {
    console.log('Shortcut triggered:', accelerator);
  },
  { emitEvent: true }
);

console.log('Shortcut registered. Press Ctrl+Shift+X to test it.');

setTimeout(async () => {
  await Desktopr.globalShortcut.unregister(accelerator);
  console.log('Shortcut unregistered:', accelerator);
}, 5000);

return {
  registered: accelerator,
  note: 'The shortcut will be unregistered automatically after 5 seconds.'
};`,
		params: [
			{
				name: 'accelerator',
				label: 'Shortcut',
				type: 'text',
				default: 'Ctrl+Shift+X',
				placeholder: 'Ctrl+Shift+X',
				description: 'e.g. Ctrl+Shift+X or Cmd+Alt+Y'
			},
			{
				name: 'delay',
				label: 'Auto-unregister delay (ms)',
				type: 'number',
				default: 5000,
				placeholder: '5000'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
const accelerator = ${JSON.stringify(p.accelerator)};
await Desktopr.globalShortcut.register(accelerator, () => {
  console.log('Shortcut triggered:', accelerator);
}, { emitEvent: true });
setTimeout(async () => {
  await Desktopr.globalShortcut.unregister(accelerator);
  console.log('Shortcut unregistered:', accelerator);
}, ${Number(p.delay)});
return { registered: accelerator, note: \`The shortcut will be unregistered after \${${Number(p.delay)}}ms.\` };`
	},
	{
		id: 'drag-drop-listen',
		label: 'Listen drag & drop',
		description: 'Registers a drag-and-drop listener for the current window.',
		category: 'events',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

const unsubscribe = await Desktopr.events.onDragDrop((name, payload) => {
  console.log('Drag/drop event:', name, payload);
}, {
  includeHover: false
});

return {
  message: 'Drag and drop listener registered. Drop a file into the window and check the console.',
  cleanup: 'Call the returned unsubscribe function when no longer needed.'
};`
	},
	{
		id: 'badge-set-clear',
		label: 'Set badge then clear',
		description: 'Sets a Dock badge count on macOS, then clears it after a short delay.',
		category: 'badge',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

if (!Desktopr.badge) {
  return 'Badge API is not available on this platform.';
}

await Desktopr.badge.set(5);

setTimeout(async () => {
  await Desktopr.badge.clear();
  console.log('Badge cleared.');
}, 3000);

return 'Badge set to 5. It will be cleared automatically after 3 seconds.';`,
		params: [
			{
				name: 'count',
				label: 'Badge count',
				type: 'number',
				default: 5,
				placeholder: '1'
			},
			{
				name: 'clearAfter',
				label: 'Clear after (ms)',
				type: 'number',
				default: 3000,
				placeholder: '3000'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
if (!Desktopr.badge) return 'Badge API is not available on this platform.';
await Desktopr.badge.set(${Number(p.count)});
setTimeout(async () => { await Desktopr.badge.clear(); }, ${Number(p.clearAfter)});
return 'Badge set to ${p.count}. It will be cleared after ${p.clearAfter}ms.';`
	},
	{
		id: 'autostart-status',
		label: 'Check autostart',
		description: 'Checks whether autostart is enabled and reads the current autostart mode.',
		category: 'autostart',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

const enabled = await Desktopr.autostart.isEnabled();
const mode = await Desktopr.autostart.mode.get();

return {
  enabled,
  mode
};`
	},
	{
		id: 'autostart-set-hidden',
		label: 'Set autostart mode',
		description: 'Sets the autostart mode to hidden without enabling autostart.',
		category: 'autostart',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

await Desktopr.autostart.mode.set('hidden');

return await Desktopr.autostart.mode.get();`
	},
	{
		id: 'diagnostics-settings',
		label: 'Get diagnostics settings',
		description: 'Returns the current diagnostics privacy/settings configuration.',
		category: 'diagnostics',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.diagnostics.settings.get();`
	},
	{
		id: 'diagnostics-settings-set',
		label: 'Edit diagnostics settings',
		description: 'Edit diagnostics settings parameters.',
		category: 'diagnostics',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

await Desktopr.diagnostics.settings.set({
	analytics_enabled: true,
	crash_reports_enabled: true,
	retention_days_logs: 180,
	retention_days_analytics: 180,
	retention_days_crashes: 365
});

return await Desktopr.diagnostics.settings.get();`,
		params: [
			{
				name: 'analytics_enabled',
				label: 'Analytics enabled',
				type: 'boolean',
				default: false
			},
			{
				name: 'crash_reports_enabled',
				label: 'Crash reports enabled',
				type: 'boolean',
				default: true
			},
			{
				name: 'retention_days_logs',
				label: 'Retention days logs',
				type: 'number',
				default: 180,
				placeholder: '180'
			},
			{
				name: 'retention_days_analytics',
				label: 'Retention days analytics',
				type: 'number',
				default: 180,
				placeholder: '180'
			},
			{
				name: 'retention_days_crashes',
				label: 'Retention days crashes',
				type: 'number',
				default: 365,
				placeholder: '365'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
await Desktopr.diagnostics.settings.set({
	analytics_enabled: ${Boolean(p.analytics_enabled)},
	crash_reports_enabled: ${Boolean(p.crash_reports_enabled)},
	retention_days_logs: ${Number(p.retention_days_logs)},
	retention_days_analytics: ${Number(p.retention_days_analytics)},
	retention_days_crashes: ${Number(p.retention_days_crashes)}
});
return await Desktopr.diagnostics.settings.get();`
	},
	{
		id: 'diagnostics-new-record',
		label: 'Create diagnostics record',
		description: 'Creates a simple diagnostics/analytics record.',
		category: 'diagnostics',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.diagnostics.newRecord(
  'playground-event',
  {
    name: 'snippet_run',
    props: {
      source: 'bridge-playground',
      timestamp: new Date().toISOString()
    }
  },
  'js'
);`,
		params: [
			{
				name: 'recordName',
				label: 'Record name',
				type: 'text',
				default: 'playground-event',
				placeholder: 'event-name'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.diagnostics.newRecord(
  ${JSON.stringify(p.recordName)},
  { name: 'snippet_run', props: { source: 'bridge-playground', timestamp: new Date().toISOString() } },
  'js'
);`
	},
	{
		id: 'menu-init-simple',
		label: 'Initialize simple menu',
		description: 'Initializes a simple native View menu with one custom item.',
		category: 'menu',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

await Desktopr.menu.init.fromConfig({
  enabled: true,
  platforms: ['macos', 'windows', 'linux'],
  view: {
    section: 'view',
    items: [
      {
        type: 'custom',
        id: 'playground.hello',
        label: 'Hello from Playground',
        enabled: true,
        interaction: 'click'
      },
      {
        type: 'separator'
      },
      {
        type: 'predefined',
        item: 'copy'
      },
      {
        type: 'predefined',
        item: 'paste'
      }
    ]
  }
});

return 'Simple menu initialized.';`
	},
	{
		id: 'menu-toggle-item',
		label: 'Toggle menu item',
		description: 'Enables/checks a menu item by id after a menu has been initialized.',
		category: 'menu',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

await Desktopr.menu.setEnabled('playground.hello', true);

return 'Menu item enabled.';`
	},
	{
		id: 'plugins-status',
		label: 'Plugin runtime status',
		description: 'Returns the current Desktopr WASM plugin runtime status.',
		category: 'plugins',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.plugins.status();`
	},
	{
		id: 'plugins-list',
		label: 'List WASM plugins',
		description: 'Lists installed WASM modules available to the Desktopr plugin runtime.',
		category: 'plugins',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.plugins.list();`
	},
	{
		id: 'plugins-add',
		label: 'Add WASM plugin',
		description: 'Opens a native file picker and registers the selected WASM module under the given name.',
		category: 'plugins',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.plugins.add('my-plugin.wasm', 10 * 1024 * 1024);`,
		params: [
			{
				name: 'name',
				label: 'Plugin name',
				type: 'text',
				default: '',
				placeholder: 'eg. my-plugin.wasm',
				description: 'The name under which the plugin will be registered in the app.'
			},
			{
				name: 'maxBytes',
				label: 'Max file size (bytes)',
				type: 'number',
				default: 10 * 1024 * 1024,
				placeholder: 'eg. 10485760'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.plugins.add(${JSON.stringify(p.name)}, ${Number(p.maxBytes)});`
	},
	{
		id: 'plugins-call-example',
		label: 'Call WASM plugin',
		description: 'Calls a named function inside an installed WASM plugin module, passing optional arguments.',
		category: 'plugins',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.plugins.call('math.wasm', {
  fn: 'add',
  args: [12, 2]
}, 5000);`,
		params: [
			{
				name: 'pluginName',
				label: 'Plugin name',
				type: 'text',
				default: '',
				placeholder: 'eg. my-plugin.wasm',
				description: 'Must match the name used when the plugin was added.'
			},
			{
				name: 'fn',
				label: 'Function name',
				type: 'text',
				default: '',
				placeholder: 'eg. my_function'
			},
			{
				name: 'args',
				label: 'Arguments (JSON array or object)',
				type: 'text',
				default: '',
				placeholder: 'eg. [1, "hello", true] or {n1: 3, n2: 5}',
				description: 'Must be a valid JSON array or object.'
			},
			{
				name: 'timeout',
				label: 'Timeout (ms)',
				type: 'number',
				default: 5000,
				placeholder: '5000'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
let _args;
try { _args = JSON.parse(${JSON.stringify(p.args)}); } catch { throw new Error('Arguments must be a valid JSON array.'); }
return await Desktopr.plugins.call(${JSON.stringify(p.pluginName)}, { fn: ${JSON.stringify(p.fn)}, args: _args }, ${Number(p.timeout)});`
	},
	{
		id: 'plugins-remove',
		label: 'Remove WASM plugin',
		description: 'Removes an installed WASM plugin module by name.',
		category: 'plugins',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.plugins.remove('my-plugin.wasm');`,
		params: [
			{
				name: 'name',
				label: 'Plugin name',
				type: 'text',
				default: '',
				placeholder: 'eg. my-plugin.wasm',
				description: 'Must match the name used when the plugin was added.'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.plugins.remove(${JSON.stringify(p.name)});`
	},
	{
		id: 'plugins-kill-jobs',
		label: 'Kill plugin jobs',
		description: 'Stops currently running plugin jobs.',
		category: 'plugins',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.plugins.killJobs();`
	},
	{
		id: 'open-browser',
		label: 'Open browser URL',
		description: 'Opens a URL in the default external browser.',
		category: 'app',
		code: `if (!isDesktoprAvailable()) {
  throw new Error('Desktopr is not available in this environment.');
}

return await Desktopr.openBrowser("https://example.com")`,
		params: [
			{
				name: 'url',
				label: 'URL',
				type: 'text',
				default: 'https://example.com',
				placeholder: 'https://example.com'
			}
		],
		buildCode: (p) => `
if (!isDesktoprAvailable()) throw new Error('Desktopr is not available in this environment.');
return await Desktopr.openBrowser(${JSON.stringify(p.url)});`
	}
];

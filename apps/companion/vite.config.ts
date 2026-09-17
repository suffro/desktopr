import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	// The `desktopr` SDK is linked from the workspace and compiled to CommonJS, so it
	// needs the same CommonJS interop Vite applies to packages installed in node_modules.
	optimizeDeps: {
		include: ['desktopr']
	},
	build: {
		commonjsOptions: {
			include: [/node_modules/, /sdk[\\/]dist-sdk/]
		}
	}
});

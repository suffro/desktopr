import { BubbledeskAPI, CompanionConfig } from "../../../_types"

export const launchCompanion = async (core: { invoke: BubbledeskAPI["invoke"] }, config?: CompanionConfig) => {

    const appConfig = {
			$schema: 'https://schema.tauri.app/config/2',
			productName: (config?.title) ?? "Companion",
			build: {
				removeUnusedCommands: true
			},
			app: {
				withGlobalTauri: true,
				security: {
					capabilities: ['remote'],
					csp: null
				},
				windows: [
					{
						label: 'main',
						title: (config?.title) ?? "Companion",
						visible: false,
						width: 1200,
						height: 800,
						backgroundColor: (config?.backgroundColor) ?? '#171717',
						resizable: true,
						fullscreen: false,
						url: (config?.url) ?? './blank.html'
					}
				]
			},
			bundle: {
				active: true,
				icon: ['icons/icon-companion.png']
			}
		};

    core.invoke("bd_launch_companion", { appConfig });
}
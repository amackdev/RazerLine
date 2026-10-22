// Button mapping state — reactive stores for the active device's button remappings.
// Listens for `button-mappings-updated` events from the worker and provides
// CRUD actions that wrap the Tauri commands.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { ButtonAction, MouseButtonId } from "$lib/types/buttons";
import { MouseButtonId as MB } from "$lib/types/buttons";

export interface ButtonMapping {
	button: MouseButtonId;
	action: ButtonAction;
}

let mappings = $state<Record<string, ButtonAction>>({});
let loading = $state(false);
let productId = $state<number | null>(null);

async function loadMappings(pid: number) {
	productId = pid;
	loading = true;
	try {
		// Get the active profile's button mappings
		const profile = await invoke<any>("get_active_profile", { product_id: pid });
		if (profile.button_mappings) {
			mappings = profile.button_mappings;
		}
	} catch (e) {
		console.error("Failed to load button mappings:", e);
	} finally {
		loading = false;
	}
}

async function updateMapping(button: MouseButtonId, action: ButtonAction) {
	if (productId === null) return;
	
	// Optimistic update
	mappings = { ...mappings, [button]: action };
	
	try {
		await invoke<void>("update_profile", { 
			product_id: productId, 
			profile: {
				...await invoke<any>("get_active_profile", { product_id: productId }),
				button_mappings: { ...mappings }
			}
		});
	} catch (e) {
		console.error("Failed to update button mapping:", e);
		// Revert on error
		await loadMappings(productId);
	}
}

async function resetToDefaults() {
	if (productId === null) return;
	
	try {
		await invoke<void>("update_profile", { 
			product_id: productId, 
			profile: {
				...await invoke<any>("get_active_profile", { product_id: productId }),
				button_mappings: {}
			}
		});
		await loadMappings(productId);
	} catch (e) {
		console.error("Failed to reset mappings:", e);
	}
}

export const buttonMapping = {
	get mappings() {
		return mappings;
	},
	get loading() {
		return loading;
	},
	get productId() {
		return productId;
	},
	loadMappings,
	updateMapping,
	resetToDefaults,
	getAction: (button: MouseButtonId) => mappings[button] ?? MB.defaultAction(button),
	subscribe: (callback: (value: any) => void) => {
		callback(buttonMapping);
		return () => {};
	},
};

let listenersStarted = false;

// Start listening for button-mappings-updated events
export function startButtonMappingListeners() {
	if (listenersStarted) return;
	listenersStarted = true;

	listen("button-mappings-updated", (event: any) => {
		const pid = event.payload?.product_id;
		if (pid !== undefined && pid !== null) {
			loadMappings(pid);
		}
	});
}
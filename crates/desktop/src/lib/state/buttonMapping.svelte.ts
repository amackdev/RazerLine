// Button mapping state — reactive stores for the active device's button remappings.
// Listens for `button-mappings-updated` events from the worker and provides
// CRUD actions that wrap the Tauri commands.

import { onMount } from "svelte";
import { invoke } from "@tauri-apps/api/core";

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

// Create a proper Svelte 5 store with subscribe method
function createButtonMappingStore() {
	type Store = {
		mappings: Record<string, ButtonAction>;
		loading: boolean;
		productId: number | null;
		loadMappings: (pid: number) => Promise<void>;
		updateMapping: (button: MouseButtonId, action: ButtonAction) => Promise<void>;
		resetToDefaults: () => Promise<void>;
		getAction: (button: MouseButtonId) => ButtonAction;
		subscribe: (callback: (value: Store) => void) => () => void;
	};

	const store: Store = {
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
		
		// Svelte store interface
		subscribe: (callback: (value: Store) => void) => {
			// Call immediately with current value
			callback(store);
			
			// Use $effect to track changes
			let prevMappings = JSON.stringify(mappings);
			let prevLoading = loading;
			let prevProductId = productId;
			
			$effect(() => {
				const currMappings = JSON.stringify(mappings);
				const currLoading = loading;
				const currProductId = productId;
				
				if (currMappings !== prevMappings || currLoading !== prevLoading || currProductId !== prevProductId) {
					prevMappings = currMappings;
					prevLoading = currLoading;
					prevProductId = currProductId;
					callback(store);
				}
			});
			
			return () => {};
		},
	};
	
	return store;
}

export const buttonMapping = createButtonMappingStore();

// Start listening for button-mappings-updated events
onMount(() => {
	let unlisten: (() => void) | null = null;
	import("@tauri-apps/api/event").then(({ listen }) => {
		listen("button-mappings-updated", (event: any) => {
			const pid = event.payload?.product_id;
			if (pid !== undefined && pid !== null) {
				loadMappings(pid);
			}
		}).then((fn) => {
			unlisten = fn;
		});
	});
	return () => unlisten?.();
});
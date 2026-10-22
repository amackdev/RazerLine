// Profile state — reactive stores for the active device's profiles.
// Listens for `profile-switched` events from the worker and provides
// CRUD actions that wrap the Tauri commands.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ProfileSummary {
	id: string;
	name: string;
	is_active: boolean;
}

export interface UserProfile {
	id: string;
	name: string;
	dpi: number;
	dpi_stages: number[];
	current_stage_idx: number;
	poll_rate_hz: number | null;
	brightness: number;
	led_color: [number, number, number];
	sync_zones: boolean;
	effect: string;
	matrix_color: [number, number, number] | null;
	button_mappings: Record<string, any>;
}

export interface AppConfigInfo {
	start_with_windows: boolean;
	close_to_tray: boolean;
	start_minimized: boolean;
}

let profiles = $state<ProfileSummary[]>([]);
let activeProfile = $state<UserProfile | null>(null);
let config = $state<AppConfigInfo>({
	start_with_windows: true,
	close_to_tray: true,
	start_minimized: false,
});
let loading = $state(false);
let productId = $state<number | null>(null);

function getProductId() {
	return productId;
}

async function loadConfig() {
	try {
		const cfg = await invoke<AppConfigInfo>("get_config");
		config = cfg;
	} catch (e) {
		console.error("Failed to load config:", e);
	}
}

async function loadProfiles(pid: number) {
	productId = pid;
	loading = true;
	try {
		const list = await invoke<ProfileSummary[]>("list_profiles", { product_id: pid });
		profiles = list;
		const active = await invoke<UserProfile>("get_active_profile", { product_id: pid });
		activeProfile = active;
	} catch (e) {
		console.error("Failed to load profiles:", e);
	} finally {
		loading = false;
	}
}

async function refreshProfiles() {
	if (productId !== null) {
		await loadProfiles(productId);
	}
}

async function setStartWithWindows(enable: boolean) {
	await invoke<void>("set_start_with_windows", { enable });
	config = { ...config, start_with_windows: enable };
}

async function setCloseToTray(enable: boolean) {
	await invoke<void>("set_close_to_tray", { enable });
	config = { ...config, close_to_tray: enable };
}

async function saveConfig() {
	await invoke<void>("save_config");
}

async function switchProfile(profileId: string) {
	if (productId === null) return;
	await invoke<UserProfile>("set_active_profile", { product_id: productId, profile_id: profileId });
	await refreshProfiles();
}

async function createProfile(name: string) {
	if (productId === null) return;
	await invoke<UserProfile>("create_profile", { product_id: productId, name });
	await refreshProfiles();
}

async function duplicateProfile() {
	if (productId === null) return;
	await invoke<UserProfile>("duplicate_profile", { product_id: productId });
	await refreshProfiles();
}

async function deleteProfile(profileId: string) {
	if (productId === null) return;
	const newList = await invoke<ProfileSummary[]>("delete_profile", { product_id: productId, profile_id: profileId });
	profiles = newList;
	if (activeProfile?.id === profileId && newList.length > 0) {
		const active = await invoke<UserProfile>("get_active_profile", { product_id: productId });
		activeProfile = active;
	}
}

async function renameProfile(profileId: string, newName: string) {
	if (productId === null) return;
	await invoke<UserProfile>("rename_profile", { product_id: productId, profile_id: profileId, new_name: newName });
	await refreshProfiles();
}

async function updateProfile(profile: UserProfile) {
	if (productId === null) return;
	await invoke<UserProfile>("update_profile", { product_id: productId, profile });
	await refreshProfiles();
}

export const profile = {
	get profiles() {
		return profiles;
	},
	get activeProfile() {
		return activeProfile;
	},
	get config() {
		return config;
	},
	get loading() {
		return loading;
	},
	get productId() {
		return productId;
	},
	loadConfig,
	loadProfiles,
	refreshProfiles,
	setStartWithWindows,
	setCloseToTray,
	saveConfig,
	switchProfile,
	createProfile,
	duplicateProfile,
	deleteProfile,
	renameProfile,
	updateProfile,
};

let listenersStarted = false;

// Start listening for profile-switched events
export function startProfileListeners() {
	if (listenersStarted) return;
	listenersStarted = true;

	listen("profile-switched", (event: any) => {
		if (event.payload?.product_id === productId) {
			refreshProfiles();
		}
	});
}
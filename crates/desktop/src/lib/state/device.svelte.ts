// Shared device-connection state (Svelte 5 runes — reactive across every
// importer, same role the old egui GUI's `connected`/`active_pid` fields on
// `GuiApp` played). Populated by the Rust background worker's
// `device-connected`/`device-disconnected` events (see
// `crates/desktop/src-tauri/src/worker.rs`), not by polling from here.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface LedZoneInfo {
	id: number;
	name: string;
	has_brightness: boolean;
}

export interface DeviceInfo {
	name: string;
	vendor_id: number;
	product_id: number;
	dpi_min: number | null;
	dpi_max: number | null;
	supported_poll_hz: number[];
	has_identity: boolean;
	has_battery: boolean;
	idle_min_seconds: number | null;
	idle_max_seconds: number | null;
	has_low_battery_threshold: boolean;
	led_zones: LedZoneInfo[];
	matrix_rows: number | null;
	matrix_cols: number | null;
	unsupported_led_effects: string[];
}

export const device: { info: DeviceInfo | null; checked: boolean } = $state({
	info: null,
	checked: false,
});

export async function refreshActiveDevice() {
	device.info = await invoke<DeviceInfo | null>("active_device");
	device.checked = true;
}

let listenersStarted = false;

/** Call once, at app startup. */
export function startDeviceListeners() {
	if (listenersStarted) return;
	listenersStarted = true;

	listen<DeviceInfo>("device-connected", (event) => {
		device.info = event.payload;
		device.checked = true;
	});
	listen("device-disconnected", () => {
		device.info = null;
	});
}

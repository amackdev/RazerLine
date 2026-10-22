// Thin wrappers over the `#[tauri::command]`s in
// `crates/desktop/src-tauri/src/commands.rs`. Device/DPI/battery have their
// own reactive state modules (`lib/state/*.svelte.ts`) since those need to
// react to push events too — this covers the remaining request/response
// commands that don't.

import { invoke } from "@tauri-apps/api/core";

export const api = {
	getPollRate: () => invoke<number>("get_poll_rate"),
	setPollRate: (hz: number) => invoke<void>("set_poll_rate", { hz }),

	getIdleTime: () => invoke<number>("get_idle_time"),
	setIdleTime: (seconds: number) => invoke<void>("set_idle_time", { seconds }),

	getLowBatteryThreshold: () => invoke<number>("get_low_battery_threshold"),
	setLowBatteryThreshold: (percent: number) =>
		invoke<void>("set_low_battery_threshold", { percent }),

	getFirmwareVersion: () => invoke<string>("get_firmware_version"),
	getSerial: () => invoke<string>("get_serial"),
};

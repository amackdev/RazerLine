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

	setLedOff: (zoneId: number) => invoke<void>("set_led_off", { zoneId }),
	setLedStatic: (zoneId: number, rgb: [number, number, number]) =>
		invoke<void>("set_led_static", { zoneId, rgb }),
	setLedBreathing: (zoneId: number, rgb: [number, number, number]) =>
		invoke<void>("set_led_breathing", { zoneId, rgb }),
	setLedSpectrum: (zoneId: number) => invoke<void>("set_led_spectrum", { zoneId }),
	setLedWave: (zoneId: number, direction: number) =>
		invoke<void>("set_led_wave", { zoneId, direction }),
	setLedBrightness: (zoneId: number, percent: number) =>
		invoke<void>("set_led_brightness", { zoneId, percent }),

	setLedOffAll: () => invoke<void>("set_led_off_all"),
	setLedStaticAll: (rgb: [number, number, number]) =>
		invoke<void>("set_led_static_all", { rgb }),
	setLedBreathingAll: (rgb: [number, number, number]) =>
		invoke<void>("set_led_breathing_all", { rgb }),
	setLedSpectrumAll: () => invoke<void>("set_led_spectrum_all"),
	setLedWaveAll: (direction: number) => invoke<void>("set_led_wave_all", { direction }),
	setLedBrightnessAll: (percent: number) => invoke<void>("set_led_brightness_all", { percent }),

	fillMatrix: (rgb: [number, number, number]) => invoke<void>("fill_matrix", { rgb }),
	setMatrixSpectrum: () => invoke<void>("set_matrix_spectrum"),
	setMatrixWave: (direction: number) => invoke<void>("set_matrix_wave", { direction }),
};

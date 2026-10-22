// Current DPI, kept in sync with hardware. The background worker polls the
// device and emits `dpi-changed` whenever the value diverges from what we
// last set (e.g. a physical DPI button) — see
// `crates/desktop/src-tauri/src/worker.rs`'s `poll_dpi`.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export const dpi: { x: number; y: number; loaded: boolean } = $state({
	x: 0,
	y: 0,
	loaded: false,
});

export async function refreshDpi() {
	try {
		const value = await invoke<{ x: number; y: number }>("get_dpi");
		dpi.x = value.x;
		dpi.y = value.y;
	} catch {
		// No device, or this model has no DPI control — leave at 0.
	} finally {
		dpi.loaded = true;
	}
}

export async function setDpi(x: number, y: number) {
	await invoke("set_dpi", { x, y });
	dpi.x = x;
	dpi.y = y;
}

let listenersStarted = false;

export function startDpiListeners() {
	if (listenersStarted) return;
	listenersStarted = true;

	listen<number>("dpi-changed", (event) => {
		dpi.x = event.payload;
		dpi.y = event.payload;
	});
}

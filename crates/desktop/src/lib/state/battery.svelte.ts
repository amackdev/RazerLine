// Battery level/charging state, pushed from the background worker's 2s
// poll loop (`crates/desktop/src-tauri/src/worker.rs`'s `poll_battery`) —
// the frontend never polls hardware directly.

import { listen } from "@tauri-apps/api/event";

export const battery: { percent: number | null; charging: boolean | null } = $state({
	percent: null,
	charging: null,
});

interface BatteryUpdate {
	percent: number | null;
	charging: boolean | null;
}

let listenersStarted = false;

export function startBatteryListeners() {
	if (listenersStarted) return;
	listenersStarted = true;

	listen<BatteryUpdate>("battery-updated", (event) => {
		battery.percent = event.payload.percent;
		battery.charging = event.payload.charging;
	});
}

//! Background device lifecycle + polling thread. Ports the scan/connect/
//! disconnect and battery-poll logic from the old egui GUI's
//! `GuiApp::rescan`/`poll_battery` (`crates/gui/src/app.rs`), adapted to
//! push updates to the frontend via Tauri events instead of mutating
//! `GuiApp` fields directly.
//!
//! Also owns the native Windows mouse hook (`input_hook.rs`) and processes
//! hardware-triggered commands (DPI cycle, Sniper Clutch, profile cycle)
//! by calling into the device client and emitting Tauri events.
//!
//! The hardware-button DPI-stage-cycling logic from the old
//! `spawn_background_worker` isn't ported yet — it's driven by
//! `UserProfile.dpi_stages`, which doesn't exist until the config/profile
//! system lands (migration Phase 4). This thread still polls DPI and emits
//! `dpi-changed` on any divergence, so the UI stays in sync with a
//! hardware DPI button even without the stage-cycling intelligence.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use razer_hid_client::RazerDevice;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::commands::{self, DeviceInfo};
#[cfg(windows)]
use crate::input_hook::{self, HookCommand};
use crate::osd;
use crate::state::AppState;

const SCAN_INTERVAL: Duration = Duration::from_millis(1000);
const DPI_POLL_INTERVAL: Duration = Duration::from_millis(400);
const BATTERY_POLL_INTERVAL: Duration = Duration::from_secs(2);
const TICK: Duration = Duration::from_millis(100);

#[derive(Serialize, Clone)]
struct BatteryUpdate {
    percent: Option<u8>,
    charging: Option<bool>,
}

pub fn spawn(app: AppHandle, state: AppState) {
    thread::Builder::new()
        .name("razerline-worker".into())
        .spawn(move || run(app, state))
        .expect("failed to spawn background worker thread");
}

fn run(app: AppHandle, state: AppState) {
    // Start the native mouse hook (Windows only)
    #[cfg(windows)]
    let hook_manager = {
        let mut mgr = input_hook::InputHookManager::new();
        let cmd_rx = mgr.take_command_rx();
        (mgr, cmd_rx)
    };

    let mut last_scan = Instant::now() - SCAN_INTERVAL;
    let mut last_battery_poll = Instant::now() - BATTERY_POLL_INTERVAL;
    let mut last_dpi_poll = Instant::now() - DPI_POLL_INTERVAL;

    loop {
        // Process hardware commands from the mouse hook
        #[cfg(windows)]
        if let Some(ref rx) = hook_manager.1 {
            while let Ok(cmd) = rx.try_recv() {
                process_hook_command(&app, &state, cmd);
            }
        }

        if last_scan.elapsed() >= SCAN_INTERVAL {
            last_scan = Instant::now();
            reconcile_connection(&app, &state);
        }

        if last_battery_poll.elapsed() >= BATTERY_POLL_INTERVAL {
            last_battery_poll = Instant::now();
            poll_battery(&app, &state);
        }

        if last_dpi_poll.elapsed() >= DPI_POLL_INTERVAL {
            last_dpi_poll = Instant::now();
            poll_dpi(&app, &state);
        }

        thread::sleep(TICK);
    }
}

#[cfg(windows)]
fn process_hook_command(app: &AppHandle, state: &AppState, cmd: HookCommand) {
    let guard = state.device.lock().unwrap();
    let Some(dev) = guard.as_ref() else { return };

    match cmd {
        HookCommand::CycleDpi => {
            if dev.profile.dpi.is_none() {
                return;
            }
            let Ok(current) = dev.get_dpi() else { return };
            let min = dev.profile.dpi.unwrap().min;
            let max = dev.profile.dpi.unwrap().max;
            let next = current.x.saturating_add(800).min(max).max(min);
            let _ = dev.set_dpi(next, next);
            state.current_dpi.store(next, Ordering::SeqCst);
            let _ = app.emit("dpi-changed", next);
            osd::show_dpi(next);
        }
        HookCommand::DpiUp => {
            if dev.profile.dpi.is_none() {
                return;
            }
            let Ok(current) = dev.get_dpi() else { return };
            let max = dev.profile.dpi.unwrap().max;
            let next = current.x.saturating_add(800).min(max);
            let _ = dev.set_dpi(next, next);
            state.current_dpi.store(next, Ordering::SeqCst);
            let _ = app.emit("dpi-changed", next);
            osd::show_dpi(next);
        }
        HookCommand::DpiDown => {
            if dev.profile.dpi.is_none() {
                return;
            }
            let Ok(current) = dev.get_dpi() else { return };
            let min = dev.profile.dpi.unwrap().min;
            let next = current.x.saturating_sub(800).max(min);
            let _ = dev.set_dpi(next, next);
            state.current_dpi.store(next, Ordering::SeqCst);
            let _ = app.emit("dpi-changed", next);
            osd::show_dpi(next);
        }
        HookCommand::SetClutchDpi(clutch_dpi) => {
            if dev.profile.dpi.is_none() {
                return;
            }
            let _ = dev.set_dpi(clutch_dpi, clutch_dpi);
            state.current_dpi.store(clutch_dpi, Ordering::SeqCst);
            let _ = app.emit("dpi-changed", clutch_dpi);
            osd::show_dpi(clutch_dpi);
        }
        HookCommand::RestoreClutchDpi => {
            // The UI will handle restoring the previous DPI via the profile system
            // For now, just trigger a DPI poll to sync
            poll_dpi(app, state);
        }
        HookCommand::CycleProfile => {
            // Emit event for frontend to handle profile cycling
            let _ = app.emit("profile-cycle", ());
        }
    }
}

/// Closes the active device if it's no longer plugged in, then opens the
/// first available device if none is currently open. Mirrors
/// `GuiApp::rescan`'s reconcile-against-what's-already-open logic, minus
/// the config/profile restore step (Phase 4).
fn reconcile_connection(app: &AppHandle, state: &AppState) {
    let connected = match razer_hid_client::list_connected(commands::known_devices()) {
        Ok(found) => found,
        Err(_) => return,
    };

    let active_pid = state.active_pid.load(Ordering::SeqCst) as u16;
    let still_present = active_pid != 0 && connected.iter().any(|p| p.product_id == active_pid);

    if active_pid != 0 && !still_present {
        *state.device.lock().unwrap() = None;
        state.active_pid.store(0, Ordering::SeqCst);
        let _ = app.emit("device-disconnected", ());
    }

    if state.active_pid.load(Ordering::SeqCst) == 0
        && let Some(profile) = connected.first().copied()
        && let Ok(dev) = RazerDevice::open(profile)
    {
        *state.device.lock().unwrap() = Some(dev);
        state
            .active_pid
            .store(profile.product_id as u32, Ordering::SeqCst);
        let info: DeviceInfo = commands::device_info(profile);
        let _ = app.emit("device-connected", info);
    }
}

fn poll_battery(app: &AppHandle, state: &AppState) {
    let (percent, charging) = {
        let guard = state.device.lock().unwrap();
        let Some(dev) = guard.as_ref() else { return };
        if dev.profile.battery.is_none() {
            return;
        }
        (dev.get_battery_level().ok(), dev.get_charging_status().ok())
    };
    let _ = app.emit("battery-updated", BatteryUpdate { percent, charging });
}

fn poll_dpi(app: &AppHandle, state: &AppState) {
    let x = {
        let guard = state.device.lock().unwrap();
        let Some(dev) = guard.as_ref() else { return };
        if dev.profile.dpi.is_none() {
            return;
        }
        match dev.get_dpi() {
            Ok(dpi) => dpi.x,
            Err(_) => return,
        }
    };
    let previous = state.current_dpi.swap(x, Ordering::SeqCst);
    if previous != x {
        let _ = app.emit("dpi-changed", x);
        osd::show_dpi(x);
    }
}

use std::sync::atomic::{AtomicU16, AtomicU32, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

/// Delay between consecutive zone writes when "All zones" fires more than
/// one LED command in a row. The CLI only ever sends one command per
/// invocation and works reliably; sending several back-to-back with no
/// gap appears to let the firmware drop/overwrite an earlier zone's
/// effect — observed as Scroll wheel's color getting clobbered by Logo's
/// write immediately after it. Keep this short; it's not meant to be
/// perceptible as lag, just enough settling time between HID writes.
const INTER_ZONE_DELAY: Duration = Duration::from_millis(60);

use eframe::egui::{self, Color32};
use razer_hid_abi::{DeviceProfile, LedZoneProfile};
use razer_hid_client::RazerDevice;

/// Library name (not a file path) of the device-catalog plugin —
/// `razer_hid_client::load_known_devices` looks for this next to the
/// running executable using the platform's DLL naming convention.
const DEVICES_PLUGIN: &str = "razer_hid_devices_mice";

fn known_devices() -> &'static [DeviceProfile] {
    static CELL: std::sync::OnceLock<&'static [DeviceProfile]> = std::sync::OnceLock::new();
    CELL.get_or_init(|| {
        razer_hid_client::load_known_devices(DEVICES_PLUGIN)
            .expect("couldn't load device catalog plugin")
    })
}

use crate::buttons::{
    ButtonAction, MediaAction, MouseButtonId, MouseFunction, SensitivityAction, WinShortcut,
};
use crate::config::AppConfig;
use crate::input_hook::{HookCommand, InputHookManager};
use crate::mouse_3d::Mouse3dView;
use crate::theme;
use crate::tray::TrayManager;
use crate::widgets::{self, PreviewKind};

/// How often to actually re-enumerate USB devices. Decoupled from egui's
/// repaint rate — HID enumeration is cheap but not free, and the UI
/// doesn't need to re-check more often than a human would notice a mouse
/// being plugged in.
const SCAN_INTERVAL: Duration = Duration::from_millis(1000);
const DPI_POLL_INTERVAL: Duration = Duration::from_millis(400);

#[derive(Clone)]
pub struct DpiRuntimeState {
    pub device: Arc<Mutex<Option<RazerDevice>>>,
    pub active_pid: Arc<AtomicU32>,
    pub current_dpi: Arc<AtomicU16>,
    pub dpi_stages: Arc<RwLock<Vec<u16>>>,
    pub current_stage_idx: Arc<AtomicUsize>,
    pub original_clutch_dpi: Arc<Mutex<Option<u16>>>,
    pub active_profile_id: Arc<RwLock<Option<String>>>,
}

impl DpiRuntimeState {
    pub fn new() -> Self {
        Self {
            device: Arc::new(Mutex::new(None)),
            active_pid: Arc::new(AtomicU32::new(0)),
            current_dpi: Arc::new(AtomicU16::new(1600)),
            dpi_stages: Arc::new(RwLock::new(vec![400, 800, 1600, 3200, 6400])),
            current_stage_idx: Arc::new(AtomicUsize::new(2)),
            original_clutch_dpi: Arc::new(Mutex::new(None)),
            active_profile_id: Arc::new(RwLock::new(None)),
        }
    }
}

/// Dedicated background worker thread that processes mouse button remapping commands
/// (DPI cycle, DPI up/down, sniper clutch, profile cycle) and continuously polls hardware
/// DPI even when the GUI window is hidden, minimized to the system tray, or while playing full-screen games.
fn spawn_background_worker(state: DpiRuntimeState, rx: Receiver<HookCommand>) {
    thread::Builder::new()
        .name("razerline-bg-worker".into())
        .spawn(move || {
            loop {
                // Wait up to 150ms for an input hook command. When timeout fires, poll hardware DPI.
                match rx.recv_timeout(Duration::from_millis(150)) {
                    Ok(cmd) => match cmd {
                        HookCommand::CycleDpi => {
                            let stages = state.dpi_stages.read().unwrap().clone();
                            if !stages.is_empty() {
                                let cur = state.current_stage_idx.load(Ordering::SeqCst);
                                let next = (cur + 1) % stages.len();
                                state.current_stage_idx.store(next, Ordering::SeqCst);
                                let target_dpi = stages[next];
                                state.current_dpi.store(target_dpi, Ordering::SeqCst);
                                if let Some(dev) = state.device.lock().unwrap().as_ref() {
                                    let _ = dev.set_dpi(target_dpi, target_dpi);
                                }
                                crate::osd::show_dpi(target_dpi);

                                let pid = state.active_pid.load(Ordering::SeqCst) as u16;
                                if pid > 0 {
                                    let mut cfg = AppConfig::load();
                                    let dev_entry = cfg.devices.entry(pid).or_default();
                                    let prof = dev_entry.ensure_profiles().active_profile_mut();
                                    prof.current_stage_idx = next;
                                    prof.dpi = target_dpi;
                                    let _ = cfg.save();
                                }
                            }
                        }
                        HookCommand::DpiUp => {
                            let stages = state.dpi_stages.read().unwrap().clone();
                            if !stages.is_empty() {
                                let cur = state.current_stage_idx.load(Ordering::SeqCst);
                                let next = (cur + 1).min(stages.len().saturating_sub(1));
                                state.current_stage_idx.store(next, Ordering::SeqCst);
                                let target_dpi = stages[next];
                                state.current_dpi.store(target_dpi, Ordering::SeqCst);
                                if let Some(dev) = state.device.lock().unwrap().as_ref() {
                                    let _ = dev.set_dpi(target_dpi, target_dpi);
                                }
                                crate::osd::show_dpi(target_dpi);

                                let pid = state.active_pid.load(Ordering::SeqCst) as u16;
                                if pid > 0 {
                                    let mut cfg = AppConfig::load();
                                    let dev_entry = cfg.devices.entry(pid).or_default();
                                    let prof = dev_entry.ensure_profiles().active_profile_mut();
                                    prof.current_stage_idx = next;
                                    prof.dpi = target_dpi;
                                    let _ = cfg.save();
                                }
                            }
                        }
                        HookCommand::DpiDown => {
                            let stages = state.dpi_stages.read().unwrap().clone();
                            if !stages.is_empty() {
                                let cur = state.current_stage_idx.load(Ordering::SeqCst);
                                let next = cur.saturating_sub(1);
                                state.current_stage_idx.store(next, Ordering::SeqCst);
                                let target_dpi = stages[next];
                                state.current_dpi.store(target_dpi, Ordering::SeqCst);
                                if let Some(dev) = state.device.lock().unwrap().as_ref() {
                                    let _ = dev.set_dpi(target_dpi, target_dpi);
                                }
                                crate::osd::show_dpi(target_dpi);

                                let pid = state.active_pid.load(Ordering::SeqCst) as u16;
                                if pid > 0 {
                                    let mut cfg = AppConfig::load();
                                    let dev_entry = cfg.devices.entry(pid).or_default();
                                    let prof = dev_entry.ensure_profiles().active_profile_mut();
                                    prof.current_stage_idx = next;
                                    prof.dpi = target_dpi;
                                    let _ = cfg.save();
                                }
                            }
                        }
                        HookCommand::SetClutchDpi(clutch_dpi) => {
                            {
                                let mut orig = state.original_clutch_dpi.lock().unwrap();
                                if orig.is_none() {
                                    *orig = Some(state.current_dpi.load(Ordering::SeqCst));
                                }
                            }
                            if let Some(dev) = state.device.lock().unwrap().as_ref() {
                                let _ = dev.set_dpi(clutch_dpi, clutch_dpi);
                            }
                            crate::osd::show_dpi(clutch_dpi);
                        }
                        HookCommand::RestoreClutchDpi => {
                            let orig = state.original_clutch_dpi.lock().unwrap().take();
                            if let Some(orig_dpi) = orig {
                                if let Some(dev) = state.device.lock().unwrap().as_ref() {
                                    let _ = dev.set_dpi(orig_dpi, orig_dpi);
                                }
                                crate::osd::show_dpi(orig_dpi);
                            }
                        }
                        HookCommand::CycleProfile => {
                            let pid = state.active_pid.load(Ordering::SeqCst) as u16;
                            if pid > 0 {
                                let mut cfg = AppConfig::load();
                                let dev_entry = cfg.devices.entry(pid).or_default();
                                let prof_cfg = dev_entry.ensure_profiles();
                                if !prof_cfg.profiles.is_empty() {
                                    let cur_pos = prof_cfg
                                        .profiles
                                        .iter()
                                        .position(|p| p.id == prof_cfg.active_profile_id)
                                        .unwrap_or(0);
                                    let next_pos = (cur_pos + 1) % prof_cfg.profiles.len();
                                    let next_id = prof_cfg.profiles[next_pos].id.clone();
                                    prof_cfg.active_profile_id = next_id.clone();
                                    let user_prof = prof_cfg.active_profile().clone();

                                    *state.active_profile_id.write().unwrap() = Some(next_id);
                                    *state.dpi_stages.write().unwrap() = user_prof.dpi_stages.clone();
                                    state.current_stage_idx.store(user_prof.current_stage_idx, Ordering::SeqCst);
                                    state.current_dpi.store(user_prof.dpi, Ordering::SeqCst);

                                    if let Some(dev) = state.device.lock().unwrap().as_ref() {
                                        let _ = dev.set_dpi(user_prof.dpi, user_prof.dpi);
                                        if let Some(hz) = user_prof.poll_rate_hz {
                                            let _ = dev.set_poll_rate(hz);
                                        }
                                        for zone in dev.profile.leds {
                                            let _ = dev.set_led_brightness(zone, user_prof.brightness);
                                        }
                                    }

                                    crate::input_hook::update_active_mappings(user_prof.button_mappings.clone());
                                    crate::osd::show_dpi(user_prof.dpi);
                                    let _ = cfg.save();
                                }
                            }
                        }
                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Periodic hardware DPI polling (e.g. user pressed physical mouse DPI button)
                        if let Some(dev) = state.device.lock().unwrap().as_ref() {
                            if let Ok(hw_dpi) = dev.get_dpi() {
                                let cur = state.current_dpi.load(Ordering::SeqCst);
                                if hw_dpi.x > 0 && hw_dpi.x != cur {
                                    state.current_dpi.store(hw_dpi.x, Ordering::SeqCst);
                                    let stages = state.dpi_stages.read().unwrap().clone();
                                    if let Some(pos) = stages.iter().position(|&s| s == hw_dpi.x) {
                                        state.current_stage_idx.store(pos, Ordering::SeqCst);
                                    }
                                    crate::osd::show_dpi(hw_dpi.x);

                                    let pid = state.active_pid.load(Ordering::SeqCst) as u16;
                                    if pid > 0 {
                                        let mut cfg = AppConfig::load();
                                        let dev_entry = cfg.devices.entry(pid).or_default();
                                        let prof = dev_entry.ensure_profiles().active_profile_mut();
                                        prof.dpi = hw_dpi.x;
                                        if let Some(pos) = prof.dpi_stages.iter().position(|&s| s == hw_dpi.x) {
                                            prof.current_stage_idx = pos;
                                        }
                                        let _ = cfg.save();
                                    }
                                }
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .expect("failed to spawn background worker thread");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Customize,
    Performance,
    Lighting,
    Profiles,
    Settings,
}

pub struct GuiApp {
    runtime_state: DpiRuntimeState,
    connected: Vec<&'static DeviceProfile>,
    active_pid: Option<u16>,
    last_scan: Instant,
    status: String,
    status_is_error: bool,

    // Active Navigation Tab
    active_tab: ActiveTab,

    // Vector Blueprint & Button Mapping
    mouse_3d: Mouse3dView,
    selected_button: Option<MouseButtonId>,
    input_hook: InputHookManager,

    // Performance
    dpi_x: u16,
    dpi_y: u16,
    custom_dpi_input: u32,
    current_poll_hz: Option<u32>,

    // Lighting
    selected_zone_idx: usize,
    sync_zones: bool,
    led_color: [u8; 3],
    brightness: u8,
    last_preview: PreviewKind,
    current_effect_name: Option<String>,
    matrix_color: [u8; 3],

    info_text: Option<String>,

    // Profiles management state
    new_profile_input: String,

    // Background & Tray & Config state
    config: AppConfig,
    _tray: Option<TrayManager>,
}

impl GuiApp {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let tray = TrayManager::new().ok();
        let mut input_hook = InputHookManager::new();
        let runtime_state = DpiRuntimeState::new();

        if let Some(rx) = input_hook.take_command_rx() {
            spawn_background_worker(runtime_state.clone(), rx);
        }

        let mut app = Self {
            runtime_state,
            connected: Vec::new(),
            active_pid: None,
            last_scan: Instant::now(),
            status: "Scanning for devices...".to_string(),
            status_is_error: false,
            active_tab: ActiveTab::Customize,
            mouse_3d: Mouse3dView::default(),
            selected_button: Some(MouseButtonId::MiddleClick),
            input_hook,
            dpi_x: 1600,
            dpi_y: 1600,
            custom_dpi_input: 1600,
            current_poll_hz: None,
            selected_zone_idx: 0,
            sync_zones: true,
            led_color: [64, 209, 128],
            brightness: 100,
            last_preview: PreviewKind::Static,
            current_effect_name: Some("static".to_string()),
            matrix_color: [64, 209, 128],
            info_text: None,
            new_profile_input: String::new(),
            config,
            _tray: tray,
        };
        app.rescan();
        if let Some(pid) = app.active_pid {
            let dev_entry = app.config.devices.entry(pid).or_default();
            let prof = dev_entry.ensure_profiles().active_profile();
            app.input_hook.update_mappings(prof.button_mappings.clone());
        }
        app
    }

    pub fn active_device_profile(&self) -> Option<&'static DeviceProfile> {
        self.runtime_state.device.lock().unwrap().as_ref().map(|d| d.profile)
    }

    pub fn has_device(&self) -> bool {
        self.runtime_state.device.lock().unwrap().is_some()
    }

    /// Re-enumerates connected devices and reconciles state against what
    /// was already open.
    fn rescan(&mut self) {
        let found = match razer_hid_client::list_connected(known_devices()) {
            Ok(found) => found,
            Err(e) => {
                self.set_status(format!("Scan error: {e}"), true);
                return;
            }
        };

        let active_still_present = self
            .active_pid
            .is_some_and(|pid| found.iter().any(|p| p.product_id == pid));

        self.connected = found;

        if !active_still_present {
            *self.runtime_state.device.lock().unwrap() = None;
            self.runtime_state.active_pid.store(0, Ordering::SeqCst);
            self.active_pid = None;
            match self.connected.first().copied() {
                Some(profile) => self.open(profile),
                None => self.set_status("No device connected", false),
            }
        }
    }

    /// Opens the specified device profile and restores its active profile settings.
    fn open(&mut self, profile: &'static DeviceProfile) {
        match RazerDevice::open(profile) {
            Ok(dev) => {
                let pid = profile.product_id;
                self.active_pid = Some(pid);
                self.info_text = None;
                self.selected_zone_idx = 0;

                // Read current hardware DPI on connection
                let mut initial_dpi = 1600;
                if let Ok(hw_dpi) = dev.get_dpi() {
                    if hw_dpi.x > 0 {
                        self.dpi_x = hw_dpi.x;
                        self.dpi_y = hw_dpi.y;
                        self.custom_dpi_input = hw_dpi.x as u32;
                        initial_dpi = hw_dpi.x;
                    }
                }

                *self.runtime_state.device.lock().unwrap() = Some(dev);
                self.runtime_state.active_pid.store(pid as u32, Ordering::SeqCst);
                self.runtime_state.current_dpi.store(initial_dpi, Ordering::SeqCst);

                // Ensure device profile collection exists
                let dev_entry = self.config.devices.entry(pid).or_default();
                let prof_cfg = dev_entry.ensure_profiles();
                let active_prof = prof_cfg.active_profile().clone();

                *self.runtime_state.dpi_stages.write().unwrap() = active_prof.dpi_stages.clone();
                self.runtime_state.current_stage_idx.store(active_prof.current_stage_idx, Ordering::SeqCst);
                *self.runtime_state.active_profile_id.write().unwrap() = Some(active_prof.id.clone());

                // Apply active profile
                self.apply_user_profile(&active_prof, profile);
            }
            Err(e) => {
                *self.runtime_state.device.lock().unwrap() = None;
                self.runtime_state.active_pid.store(0, Ordering::SeqCst);
                self.active_pid = None;
                self.set_status(format!("Couldn't open {}: {e}", profile.name), true);
            }
        }
    }

    /// Applies a UserProfile to the active hardware and GUI state.
    fn apply_user_profile(
        &mut self,
        user_prof: &crate::profiles::UserProfile,
        profile: &'static DeviceProfile,
    ) {
        let mut applied = Vec::new();

        // 1. DPI
        self.dpi_x = user_prof.dpi;
        self.dpi_y = user_prof.dpi;
        self.custom_dpi_input = user_prof.dpi as u32;

        self.runtime_state.current_dpi.store(user_prof.dpi, Ordering::SeqCst);
        *self.runtime_state.dpi_stages.write().unwrap() = user_prof.dpi_stages.clone();
        self.runtime_state.current_stage_idx.store(user_prof.current_stage_idx, Ordering::SeqCst);

        let mut set_poll = None;
        {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            if let Some(dev) = dev_guard.as_ref() {
                if profile.dpi.is_some() && dev.set_dpi(self.dpi_x, self.dpi_y).is_ok() {
                    applied.push("DPI");
                }

                // 2. Polling rate
                if let Some(hz) = user_prof.poll_rate_hz {
                    if profile.poll_rate.is_some() && dev.set_poll_rate(hz).is_ok() {
                        set_poll = Some(hz);
                        applied.push("Polling rate");
                    }
                }

                // 3. Brightness & Color
                self.brightness = user_prof.brightness;
                self.led_color = user_prof.led_color;
                self.sync_zones = user_prof.sync_zones;
                self.current_effect_name = Some(user_prof.effect.clone());

                for zone in profile.leds {
                    let _ = dev.set_led_brightness(zone, user_prof.brightness);
                }

                // Apply effect
                match user_prof.effect.as_str() {
                    "static" => {
                        self.last_preview = PreviewKind::Static;
                        for (i, zone) in profile.leds.iter().enumerate() {
                            if i > 0 {
                                sleep(INTER_ZONE_DELAY);
                            }
                            let _ = dev.set_led_static(zone, user_prof.led_color);
                        }
                        if profile.matrix.is_some() {
                            sleep(INTER_ZONE_DELAY);
                            let _ = dev.fill_matrix(user_prof.led_color);
                        }
                        applied.push("Lighting");
                    }
                    "spectrum" => {
                        self.last_preview = PreviewKind::Spectrum;
                        for (i, zone) in profile.leds.iter().enumerate() {
                            if i > 0 {
                                sleep(INTER_ZONE_DELAY);
                            }
                            let _ = dev.set_led_spectrum(zone);
                        }
                        if profile.matrix.and_then(|m| m.effect_led_id).is_some() {
                            sleep(INTER_ZONE_DELAY);
                            let _ = dev.set_matrix_spectrum();
                        }
                        applied.push("Lighting");
                    }
                    "breathing" => {
                        self.last_preview = PreviewKind::Breathing;
                        for (i, zone) in profile.leds.iter().enumerate() {
                            if i > 0 {
                                sleep(INTER_ZONE_DELAY);
                            }
                            let _ = dev.set_led_breathing(zone, user_prof.led_color);
                        }
                        applied.push("Lighting");
                    }
                    "wave" => {
                        self.last_preview = PreviewKind::Wave;
                        for (i, zone) in profile.leds.iter().enumerate() {
                            if i > 0 {
                                sleep(INTER_ZONE_DELAY);
                            }
                            let _ = dev.set_led_wave(zone, 1);
                        }
                        if profile.matrix.and_then(|m| m.effect_led_id).is_some() {
                            sleep(INTER_ZONE_DELAY);
                            let _ = dev.set_matrix_wave(1);
                        }
                        applied.push("Lighting");
                    }
                    "off" => {
                        self.last_preview = PreviewKind::Off;
                        for (i, zone) in profile.leds.iter().enumerate() {
                            if i > 0 {
                                sleep(INTER_ZONE_DELAY);
                            }
                            let _ = dev.set_led_off(zone);
                        }
                        if profile.matrix.is_some() {
                            sleep(INTER_ZONE_DELAY);
                            let _ = dev.fill_matrix([0, 0, 0]);
                        }
                        applied.push("Lighting");
                    }
                    _ => {}
                }
            }
        }

        if let Some(hz) = set_poll {
            self.current_poll_hz = Some(hz);
        }

        // 4. Button remapping hook table
        self.input_hook.update_mappings(user_prof.button_mappings.clone());

        let msg = if applied.is_empty() {
            format!("Loaded profile: {}", user_prof.name)
        } else {
            format!(
                "Loaded profile: {} ({})",
                user_prof.name,
                applied.join(", ")
            )
        };
        self.set_status(msg, false);
    }

    /// Switches the active profile and applies it.
    fn switch_profile(&mut self, profile_id: &str) {
        if let Some(pid) = self.active_pid {
            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof_cfg = dev_entry.ensure_profiles();
            prof_cfg.active_profile_id = profile_id.to_string();
            let user_prof = prof_cfg.active_profile().clone();

            *self.runtime_state.active_profile_id.write().unwrap() = Some(profile_id.to_string());
            *self.runtime_state.dpi_stages.write().unwrap() = user_prof.dpi_stages.clone();
            self.runtime_state.current_stage_idx.store(user_prof.current_stage_idx, Ordering::SeqCst);
            self.runtime_state.current_dpi.store(user_prof.dpi, Ordering::SeqCst);

            if let Some(profile) = self
                .connected
                .iter()
                .find(|p| p.product_id == pid)
                .copied()
            {
                let old_dpi = self.dpi_x;
                self.apply_user_profile(&user_prof, profile);
                if old_dpi != user_prof.dpi {
                    crate::osd::show_dpi(user_prof.dpi);
                }
            }
            let _ = self.config.save();
        }
    }

    /// Persists current device settings into the active profile.
    fn save_current_device_config(&mut self) {
        if let Some(pid) = self.active_pid {
            let name = self
                .active_device_profile()
                .map(|p| p.name)
                .unwrap_or("Razer Device");
            let dpi = self.dpi_x;
            let poll = self.current_poll_hz;
            let brightness = self.brightness;
            let color = self.led_color;
            let sync = self.sync_zones;
            let effect = self.current_effect_name.clone();
            let matrix = {
                let dev_guard = self.runtime_state.device.lock().unwrap();
                dev_guard
                    .as_ref()
                    .and_then(|d| d.matrix_dims())
                    .map(|_| self.matrix_color)
            };

            self.config.update_device(
                pid,
                name,
                dpi,
                poll,
                brightness,
                color,
                sync,
                effect,
                matrix,
            );
            let _ = self.config.save();
        }
    }

    /// Synchronizes GUI state with changes made by the background worker thread.
    fn sync_from_background(&mut self) {
        let bg_dpi = self.runtime_state.current_dpi.load(Ordering::Relaxed);
        let bg_stage = self.runtime_state.current_stage_idx.load(Ordering::Relaxed);
        if bg_dpi > 0 && bg_dpi != self.dpi_x {
            let old_dpi = self.dpi_x;
            self.dpi_x = bg_dpi;
            self.dpi_y = bg_dpi;
            self.custom_dpi_input = bg_dpi as u32;
            if let Some(pid) = self.active_pid {
                let dev_entry = self.config.devices.entry(pid).or_default();
                let prof = dev_entry.ensure_profiles().active_profile_mut();
                prof.dpi = bg_dpi;
                prof.current_stage_idx = bg_stage;
                if let Some(pos) = prof.dpi_stages.iter().position(|&s| s == bg_dpi) {
                    self.set_status(format!("DPI switched: {} DPI (Stage {})", bg_dpi, pos + 1), false);
                } else {
                    self.set_status(format!("DPI switched: {} -> {} DPI", old_dpi, bg_dpi), false);
                }
            }
        }

        // Check if profile was switched in background
        if let Some(bg_prof_id) = self.runtime_state.active_profile_id.read().unwrap().clone() {
            if let Some(pid) = self.active_pid {
                let dev_entry = self.config.devices.entry(pid).or_default();
                let prof_cfg = dev_entry.ensure_profiles();
                if prof_cfg.active_profile_id != bg_prof_id {
                    self.config = AppConfig::load();
                }
            }
        }
    }

    /// Cycles to the next DPI stage on both the hardware mouse and GUI config.
    pub fn cycle_dpi_stage(&mut self) {
        let Some(pid) = self.active_pid else { return };

        let (target_dpi, next_idx) = {
            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof = dev_entry.ensure_profiles().active_profile_mut();
            if prof.dpi_stages.is_empty() {
                return;
            }
            let next_idx = (prof.current_stage_idx + 1) % prof.dpi_stages.len();
            prof.current_stage_idx = next_idx;
            let target_dpi = prof.dpi_stages[next_idx];
            prof.dpi = target_dpi;
            (target_dpi, next_idx)
        };

        self.dpi_x = target_dpi;
        self.dpi_y = target_dpi;
        self.custom_dpi_input = target_dpi as u32;

        self.runtime_state.current_stage_idx.store(next_idx, Ordering::SeqCst);
        self.runtime_state.current_dpi.store(target_dpi, Ordering::SeqCst);

        crate::osd::show_dpi(target_dpi);

        let r = {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            let Some(dev) = dev_guard.as_ref() else { return };
            dev.set_dpi(target_dpi, target_dpi)
        };
        self.apply(r, &format!("Switched to DPI Stage {}: {} DPI", next_idx + 1, target_dpi));
        let _ = self.config.save();
    }

    /// Switches directly to a specific DPI stage index.
    pub fn set_dpi_stage(&mut self, stage_idx: usize) {
        let Some(pid) = self.active_pid else { return };

        let target_dpi = {
            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof = dev_entry.ensure_profiles().active_profile_mut();
            if stage_idx >= prof.dpi_stages.len() {
                return;
            }
            prof.current_stage_idx = stage_idx;
            let target_dpi = prof.dpi_stages[stage_idx];
            prof.dpi = target_dpi;
            target_dpi
        };

        self.dpi_x = target_dpi;
        self.dpi_y = target_dpi;
        self.custom_dpi_input = target_dpi as u32;

        self.runtime_state.current_stage_idx.store(stage_idx, Ordering::SeqCst);
        self.runtime_state.current_dpi.store(target_dpi, Ordering::SeqCst);

        crate::osd::show_dpi(target_dpi);

        let r = {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            let Some(dev) = dev_guard.as_ref() else { return };
            dev.set_dpi(target_dpi, target_dpi)
        };
        self.apply(r, &format!("Switched to DPI Stage {}: {} DPI", stage_idx + 1, target_dpi));
        let _ = self.config.save();
    }

    /// Sets a specific DPI value directly on hardware and saves it.
    pub fn apply_dpi_value(&mut self, target_dpi: u16) {
        let Some(pid) = self.active_pid else { return };

        self.dpi_x = target_dpi;
        self.dpi_y = target_dpi;
        self.custom_dpi_input = target_dpi as u32;

        self.runtime_state.current_dpi.store(target_dpi, Ordering::SeqCst);

        crate::osd::show_dpi(target_dpi);

        {
            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof = dev_entry.ensure_profiles().active_profile_mut();
            prof.dpi = target_dpi;
            if let Some(pos) = prof.dpi_stages.iter().position(|&s| s == target_dpi) {
                prof.current_stage_idx = pos;
                self.runtime_state.current_stage_idx.store(pos, Ordering::SeqCst);
            }
        }

        let r = {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            let Some(dev) = dev_guard.as_ref() else { return };
            dev.set_dpi(target_dpi, target_dpi)
        };
        self.apply(r, &format!("DPI set to {} DPI", target_dpi));
        let _ = self.config.save();
    }

    /// Updates the configured DPI value for a specific stage index in the cycle.
    pub fn set_dpi_stage_value(&mut self, stage_idx: usize, new_dpi: u16) {
        let Some(pid) = self.active_pid else { return };

        let is_current = {
            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof = dev_entry.ensure_profiles().active_profile_mut();
            if stage_idx >= prof.dpi_stages.len() {
                return;
            }
            prof.set_stage_dpi(stage_idx, new_dpi);
            let is_curr = prof.current_stage_idx == stage_idx;
            *self.runtime_state.dpi_stages.write().unwrap() = prof.dpi_stages.clone();
            is_curr
        };

        let _ = self.config.save();

        if is_current {
            self.dpi_x = new_dpi;
            self.dpi_y = new_dpi;
            self.custom_dpi_input = new_dpi as u32;
            self.runtime_state.current_dpi.store(new_dpi, Ordering::SeqCst);
            let r = {
                let dev_guard = self.runtime_state.device.lock().unwrap();
                let Some(dev) = dev_guard.as_ref() else { return };
                dev.set_dpi(new_dpi, new_dpi)
            };
            crate::osd::show_dpi(new_dpi);
            self.apply(r, &format!("Stage {} DPI updated to {} DPI", stage_idx + 1, new_dpi));
        } else {
            self.set_status(format!("Stage {} DPI set to {} DPI", stage_idx + 1, new_dpi), false);
        }
    }

    /// Sets the number of DPI stages in the cycle (between 1 and 5).
    pub fn set_dpi_stage_count(&mut self, count: usize, min_dpi: u16, max_dpi: u16) {
        let Some(pid) = self.active_pid else { return };

        let (new_stages, new_idx, new_dpi) = {
            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof = dev_entry.ensure_profiles().active_profile_mut();
            prof.set_stage_count(count, min_dpi, max_dpi);
            (prof.dpi_stages.clone(), prof.current_stage_idx, prof.dpi)
        };

        self.dpi_x = new_dpi;
        self.dpi_y = new_dpi;
        self.custom_dpi_input = new_dpi as u32;

        *self.runtime_state.dpi_stages.write().unwrap() = new_stages;
        self.runtime_state.current_stage_idx.store(new_idx, Ordering::SeqCst);
        self.runtime_state.current_dpi.store(new_dpi, Ordering::SeqCst);

        let _ = self.config.save();

        let r = {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            let Some(dev) = dev_guard.as_ref() else { return };
            dev.set_dpi(new_dpi, new_dpi)
        };
        crate::osd::show_dpi(new_dpi);
        self.apply(r, &format!("Configured {count} sensitivity stages"));
    }

    /// Adds a new DPI stage to the cycle (up to 5 max).
    pub fn add_dpi_stage(&mut self, min_dpi: u16, max_dpi: u16) {
        let Some(pid) = self.active_pid else { return };
        let dev_entry = self.config.devices.entry(pid).or_default();
        let prof = dev_entry.ensure_profiles().active_profile_mut();
        if prof.add_stage(min_dpi, max_dpi) {
            *self.runtime_state.dpi_stages.write().unwrap() = prof.dpi_stages.clone();
            let new_len = prof.dpi_stages.len();
            let new_dpi = *prof.dpi_stages.last().unwrap();
            let _ = self.config.save();
            self.set_status(format!("Added Stage {new_len}: {new_dpi} DPI"), false);
        }
    }

    /// Removes a DPI stage from the cycle by index (keeps at least 1).
    pub fn remove_dpi_stage(&mut self, stage_idx: usize) {
        let Some(pid) = self.active_pid else { return };

        let (new_stages, new_idx, new_dpi) = {
            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof = dev_entry.ensure_profiles().active_profile_mut();
            if !prof.remove_stage(stage_idx) {
                return;
            }
            (prof.dpi_stages.clone(), prof.current_stage_idx, prof.dpi)
        };

        self.dpi_x = new_dpi;
        self.dpi_y = new_dpi;
        self.custom_dpi_input = new_dpi as u32;

        *self.runtime_state.dpi_stages.write().unwrap() = new_stages;
        self.runtime_state.current_stage_idx.store(new_idx, Ordering::SeqCst);
        self.runtime_state.current_dpi.store(new_dpi, Ordering::SeqCst);

        let _ = self.config.save();

        let r = {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            let Some(dev) = dev_guard.as_ref() else { return };
            dev.set_dpi(new_dpi, new_dpi)
        };
        crate::osd::show_dpi(new_dpi);
        self.apply(r, &format!("Stage removed. Active: Stage {}: {} DPI", new_idx + 1, new_dpi));
    }

    /// Sorts all DPI stages from lowest to highest.
    pub fn sort_dpi_stages(&mut self) {
        let Some(pid) = self.active_pid else { return };
        let dev_entry = self.config.devices.entry(pid).or_default();
        let prof = dev_entry.ensure_profiles().active_profile_mut();
        prof.sort_stages();

        *self.runtime_state.dpi_stages.write().unwrap() = prof.dpi_stages.clone();
        self.runtime_state.current_stage_idx.store(prof.current_stage_idx, Ordering::SeqCst);
        let _ = self.config.save();
        self.set_status("Sorted DPI stages from low to high", false);
    }

    /// Resets DPI stages to factory defaults (400, 800, 1600, 3200, 6400).
    pub fn reset_dpi_stages_to_default(&mut self) {
        let Some(pid) = self.active_pid else { return };
        let dev_entry = self.config.devices.entry(pid).or_default();
        let prof = dev_entry.ensure_profiles().active_profile_mut();
        prof.dpi_stages = vec![400, 800, 1600, 3200, 6400];
        prof.current_stage_idx = 2;
        prof.dpi = 1600;

        self.dpi_x = 1600;
        self.dpi_y = 1600;
        self.custom_dpi_input = 1600;

        *self.runtime_state.dpi_stages.write().unwrap() = prof.dpi_stages.clone();
        self.runtime_state.current_stage_idx.store(2, Ordering::SeqCst);
        self.runtime_state.current_dpi.store(1600, Ordering::SeqCst);

        let _ = self.config.save();

        let r = {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            let Some(dev) = dev_guard.as_ref() else { return };
            dev.set_dpi(1600, 1600)
        };
        crate::osd::show_dpi(1600);
        self.apply(r, "Reset DPI stages to defaults (400, 800, 1600, 3200, 6400)");
    }

    /// Cycles the active profile (triggered by profile switch button).
    pub fn cycle_active_profile(&mut self) {
        let Some(pid) = self.active_pid else { return };
        let dev_entry = self.config.devices.entry(pid).or_default();
        let prof_cfg = dev_entry.ensure_profiles();
        if prof_cfg.profiles.is_empty() {
            return;
        }

        let current_pos = prof_cfg
            .profiles
            .iter()
            .position(|p| p.id == prof_cfg.active_profile_id)
            .unwrap_or(0);
        let next_pos = (current_pos + 1) % prof_cfg.profiles.len();
        let next_id = prof_cfg.profiles[next_pos].id.clone();
        self.switch_profile(&next_id);
    }

    fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status = msg.into();
        self.status_is_error = is_error;
    }

    fn apply<T>(&mut self, result: Result<T, razer_hid_client::ClientError>, ok_msg: &str) {
        match result {
            Ok(_) => self.set_status(ok_msg, false),
            Err(e) => self.set_status(format!("Error: {e}"), true),
        }
    }

    fn apply_to_zones(
        &mut self,
        zones: &'static [LedZoneProfile],
        selected_idx: usize,
        mut f: impl FnMut(
            &RazerDevice,
            &'static LedZoneProfile,
        ) -> Result<(), razer_hid_client::ClientError>,
        ok_msg: &str,
    ) {
        let targets: &'static [LedZoneProfile] = if self.sync_zones {
            zones
        } else {
            &zones[selected_idx..=selected_idx]
        };
        let mut failure = None;
        {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            let Some(dev) = dev_guard.as_ref() else { return };
            for (i, zone) in targets.iter().enumerate() {
                if i > 0 {
                    sleep(INTER_ZONE_DELAY);
                }
                if let Err(e) = f(dev, zone) {
                    failure.get_or_insert_with(|| format!("{} failed: {e}", zone.name));
                }
            }
        }
        match failure {
            None => self.set_status(ok_msg, false),
            Some(msg) => self.set_status(msg, true),
        }
    }

    fn apply_to_zones_and_matrix(
        &mut self,
        zones: &'static [LedZoneProfile],
        selected_idx: usize,
        mut zone_f: impl FnMut(
            &RazerDevice,
            &'static LedZoneProfile,
        ) -> Result<(), razer_hid_client::ClientError>,
        matrix_f: Option<impl FnOnce(&RazerDevice) -> Result<(), razer_hid_client::ClientError>>,
        ok_msg: &str,
    ) {
        let targets: &'static [LedZoneProfile] = if self.sync_zones {
            zones
        } else {
            &zones[selected_idx..=selected_idx]
        };
        let mut failure = None;
        {
            let dev_guard = self.runtime_state.device.lock().unwrap();
            let Some(dev) = dev_guard.as_ref() else { return };
            for (i, zone) in targets.iter().enumerate() {
                if i > 0 {
                    sleep(INTER_ZONE_DELAY);
                }
                if let Err(e) = zone_f(dev, zone) {
                    failure.get_or_insert_with(|| format!("{} failed: {e}", zone.name));
                }
            }
            if self.sync_zones {
                if let Some(matrix_f) = matrix_f {
                    if !targets.is_empty() {
                        sleep(INTER_ZONE_DELAY);
                    }
                    if let Err(e) = matrix_f(dev) {
                        failure.get_or_insert_with(|| format!("Underglow failed: {e}"));
                    }
                }
            }
        }
        match failure {
            None => self.set_status(ok_msg, false),
            Some(msg) => self.set_status(msg, true),
        }
    }
}

impl Default for GuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Window close interception: if close_to_tray is true, hide to tray instead of quitting
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.config.close_to_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                crate::win_util::hide_window();
            }
        }

        // Live USB detection: re-scan periodically
        if self.last_scan.elapsed() >= SCAN_INTERVAL {
            self.rescan();
            self.last_scan = Instant::now();
        }

        // Sync DPI, stage index, and active profile updated by the background worker
        self.sync_from_background();

        static EGUI_CTX: std::sync::OnceLock<egui::Context> = std::sync::OnceLock::new();
        if EGUI_CTX.get().is_none() {
            let _ = EGUI_CTX.set(ctx.clone());
            let _ = std::thread::Builder::new()
                .name("razerline-bg-heartbeat".into())
                .spawn(|| loop {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    if let Some(c) = EGUI_CTX.get() {
                        c.request_repaint();
                    }
                });
        }

        ctx.request_repaint_after(DPI_POLL_INTERVAL);

        // Top Navigation Bar
        egui::TopBottomPanel::top("top_navbar")
            .frame(
                egui::Frame::none()
                    .fill(theme::SIDEBAR_BG)
                    .inner_margin(egui::Margin::symmetric(20.0, 12.0)),
            )
            .show(ctx, |ui| self.ui_top_navbar(ui));

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::none()
                    .fill(theme::SIDEBAR_BG)
                    .inner_margin(egui::Margin::symmetric(20.0, 8.0)),
            )
            .show(ctx, |ui| self.ui_status_bar(ui));

        // Central Content Area
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(theme::APP_BG)
                    .inner_margin(egui::Margin::symmetric(24.0, 18.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.ui_content(ui));
            });
    }
}

fn ui_nav_tab(ui: &mut egui::Ui, label: &str, is_active: bool) -> bool {
    let font_id = egui::FontId::proportional(12.5);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font_id.clone(), Color32::WHITE);
    let text_size = galley.size();
    let pad_x = 16.0;
    let tab_size = egui::vec2(text_size.x + pad_x * 2.0, 36.0);

    let (rect, response) = ui.allocate_exact_size(tab_size, egui::Sense::click());
    let is_hovered = response.hovered();

    let text_color = if is_active {
        theme::ACCENT
    } else if is_hovered {
        Color32::WHITE
    } else {
        Color32::from_gray(160)
    };

    let text_pos = egui::pos2(
        rect.min.x + pad_x,
        rect.center().y - text_size.y * 0.5 - 1.0,
    );
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        label,
        font_id,
        text_color,
    );

    if is_active {
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 8.0, rect.max.y - 2.5),
            egui::pos2(rect.max.x - 8.0, rect.max.y),
        );
        ui.painter().rect_filled(bar_rect, 1.25, theme::ACCENT);
    } else if is_hovered {
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + 12.0, rect.max.y - 2.0),
            egui::pos2(rect.max.x - 12.0, rect.max.y),
        );
        ui.painter().rect_filled(bar_rect, 1.0, Color32::from_white_alpha(50));
    }

    response.clicked()
}

impl GuiApp {
    fn ui_top_navbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Brand Logo & Connected Dot
            ui.horizontal(|ui| {
                widgets::status_dot(ui, self.has_device());
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("RazerLine")
                        .strong()
                        .size(17.0)
                        .color(theme::ACCENT),
                );
            });

            ui.add_space(28.0);

            // Large Scale Suite Navigation Tabs
            let tabs = [
                (ActiveTab::Customize, "CUSTOMIZE"),
                (ActiveTab::Performance, "PERFORMANCE"),
                (ActiveTab::Lighting, "LIGHTING"),
                (ActiveTab::Profiles, "PROFILES"),
                (ActiveTab::Settings, "SETTINGS"),
            ];

            for (tab, label) in tabs {
                let is_active = self.active_tab == tab;
                if ui_nav_tab(ui, label, is_active) {
                    self.active_tab = tab;
                }
                ui.add_space(2.0);
            }

            // Right-aligned Profile Quick Switcher & Device Badge
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(pid) = self.active_pid {
                    let dev_entry = self.config.devices.entry(pid).or_default();
                    let prof_cfg = dev_entry.ensure_profiles();
                    let active_name = prof_cfg.active_profile().name.clone();
                    let active_id = prof_cfg.active_profile().id.clone();
                    let profiles: Vec<(String, String)> = prof_cfg
                        .profiles
                        .iter()
                        .map(|p| (p.id.clone(), p.name.clone()))
                        .collect();

                    let mut target_profile = None;
                    egui::ComboBox::from_id_source("top_profile_selector")
                        .selected_text(format!("Profile: {active_name}"))
                        .show_ui(ui, |ui| {
                            for (id, name) in profiles {
                                if ui.selectable_label(id == active_id, name).clicked() {
                                    target_profile = Some(id);
                                }
                            }
                        });

                    if let Some(id) = target_profile {
                        self.switch_profile(&id);
                    }

                    // Connected Device Name
                    if let Some(profile) = self
                        .connected
                        .iter()
                        .find(|p| p.product_id == pid)
                        .copied()
                    {
                        ui.label(
                            egui::RichText::new(profile.name)
                                .small()
                                .color(theme::MUTED),
                        );
                    }
                } else {
                    ui.label(
                        egui::RichText::new("No Device")
                            .small()
                            .color(theme::MUTED),
                    );
                }
            });
        });
    }

    fn ui_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let color = if self.status_is_error {
                theme::DANGER
            } else {
                theme::MUTED
            };
            ui.label(egui::RichText::new(&self.status).small().color(color));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("RazerLine Suite v0.1.0")
                        .small()
                        .color(theme::MUTED),
                );
            });
        });
    }

    fn ui_content(&mut self, ui: &mut egui::Ui) {
        let Some(profile) = self.active_device_profile() else {
            ui.add_space(60.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No Razer device connected")
                        .size(18.0)
                        .color(theme::MUTED),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "Plug in your Razer mouse via USB — it will be detected automatically.",
                    )
                    .color(theme::MUTED),
                );
            });
            ui.add_space(40.0);
            self.ui_system_settings_card(ui);
            return;
        };

        match self.active_tab {
            ActiveTab::Customize => self.ui_tab_customize(ui, profile),
            ActiveTab::Performance => self.ui_tab_performance(ui, profile),
            ActiveTab::Lighting => self.ui_tab_lighting(ui, profile),
            ActiveTab::Profiles => self.ui_tab_profiles(ui),
            ActiveTab::Settings => self.ui_tab_settings(ui),
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // TAB 1: CUSTOMIZE (3D Mouse & Button Remapping)
    // ═════════════════════════════════════════════════════════════════════
    fn ui_tab_customize(&mut self, ui: &mut egui::Ui, profile: &'static DeviceProfile) {
        ui.horizontal(|ui| {
            ui.heading("Button Customization");
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Configure physical mouse button assignments")
                    .color(theme::MUTED)
                    .small(),
            );
        });
        ui.add_space(12.0);

        let time = ui.input(|i| i.time);
        let color = self.led_color;

        let total_w = ui.available_width();
        let left_w = (total_w * 0.63).max(560.0);
        let right_w = (total_w - left_w - 20.0).max(320.0);

        ui.horizontal_top(|ui| {
            // Left Column: Vector Blueprint Visualizer (Top / Bottom View) (63% width)
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    if let Some(clicked_btn) = self.mouse_3d.show(ui, profile.name, self.selected_button, color, time) {
                        if clicked_btn.is_configurable() {
                            self.selected_button = Some(clicked_btn);
                        }
                    }

                    ui.add_space(14.0);

                    // Hardware Specifications & Blueprint Summary
                    widgets::card(ui, "Hardware Blueprint Summary", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(profile.name)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new("• Focus+ 26K Sensor • 11-Zone Chroma Underglow • Pure PTFE Skates")
                                    .small()
                                    .color(theme::MUTED),
                            );
                        });
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new("Click any pin on the Top or Bottom (Profile) blueprint above, or use the Target Button dropdown on the right to remap.")
                                .small()
                                .color(theme::MUTED),
                        );
                    });
                },
            );

            ui.add_space(14.0);

            // Right Column: Button Action Inspector (37% width)
            ui.allocate_ui_with_layout(
                egui::vec2(right_w, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    self.ui_button_inspector(ui);
                },
            );
        });
    }

    fn ui_button_inspector(&mut self, ui: &mut egui::Ui) {
        if let Some(b) = self.selected_button {
            if !b.is_configurable() {
                self.selected_button = Some(MouseButtonId::MiddleClick);
            }
        }
        let Some(btn_id) = self.selected_button.filter(|b| b.is_configurable()) else {
            widgets::card(ui, "Button Configuration", |ui| {
                ui.label("Select a configurable button on the blueprint to remap.");
            });
            return;
        };

        let mut trigger_dpi_cycle = false;
        let mut trigger_profile_cycle = false;

        widgets::card(ui, &format!("Configuring: {}", btn_id.display_name()), |ui| {
            let pid = match self.active_pid {
                Some(pid) => pid,
                None => return,
            };

            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof = dev_entry.ensure_profiles().active_profile_mut();
            let current_action = prof.get_button_action(btn_id);

            // Target Button selector ComboBox
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Target Button:").small().color(theme::MUTED));
                egui::ComboBox::from_id_source("inspector_target_btn_combo")
                    .selected_text(btn_id.display_name())
                    .show_ui(ui, |ui| {
                        for b in MouseButtonId::CONFIGURABLE {
                            let is_sel = self.selected_button == Some(b);
                            if ui.selectable_label(is_sel, b.display_name()).clicked() {
                                self.selected_button = Some(b);
                            }
                        }
                    });
            });
            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(format!("Current assignment: {}", current_action.label()))
                    .strong()
                    .color(theme::ACCENT),
            );
            ui.add_space(8.0);

            // Immediate Trigger Action button if inspecting DPI Switch or Profile Switch
            if btn_id == MouseButtonId::DpiSwitch || matches!(current_action, ButtonAction::Sensitivity(SensitivityAction::CycleDpi)) {
                if ui.button("Test / Cycle DPI Now (Hardware)").clicked() {
                    trigger_dpi_cycle = true;
                }
                ui.add_space(8.0);
            } else if btn_id == MouseButtonId::ProfileSwitch || matches!(current_action, ButtonAction::ProfileCycle) {
                if ui.button("Test / Cycle Profile Now").clicked() {
                    trigger_profile_cycle = true;
                }
                ui.add_space(8.0);
            }

            ui.label(egui::RichText::new("REMAP ACTION CATEGORY").small().color(theme::MUTED));
            ui.add_space(6.0);

            let mut new_action = None;

            // Option 1: Default
            if ui.button("Restore Default Function").clicked() {
                new_action = Some(btn_id.default_action());
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Option 2: Profile Cycle
            ui.label(egui::RichText::new("Profile Switching").small().color(theme::MUTED));
            ui.horizontal_wrapped(|ui| {
                if ui.button("Switch / Cycle Profile").clicked() {
                    new_action = Some(ButtonAction::ProfileCycle);
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Option 3: Mouse Functions
            ui.label(egui::RichText::new("Mouse Clicks & Navigation").small().color(theme::MUTED));
            ui.horizontal_wrapped(|ui| {
                let funcs = [
                    (MouseFunction::LeftClick, "Left Click"),
                    (MouseFunction::RightClick, "Right Click"),
                    (MouseFunction::MiddleClick, "Middle Click"),
                    (MouseFunction::ScrollLeft, "Scroll Left"),
                    (MouseFunction::ScrollRight, "Scroll Right"),
                    (MouseFunction::Back, "Back (M4)"),
                    (MouseFunction::Forward, "Forward (M5)"),
                    (MouseFunction::DoubleClick, "Double Click"),
                ];
                for (mf, label) in funcs {
                    if ui.button(label).clicked() {
                        new_action = Some(ButtonAction::Mouse(mf));
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Option 4: Sensitivity & DPI
            ui.label(egui::RichText::new("Sensitivity Controls").small().color(theme::MUTED));
            ui.horizontal_wrapped(|ui| {
                if ui.button("DPI Cycle").clicked() {
                    new_action = Some(ButtonAction::Sensitivity(SensitivityAction::CycleDpi));
                }
                if ui.button("DPI Stage Up").clicked() {
                    new_action = Some(ButtonAction::Sensitivity(SensitivityAction::DpiUp));
                }
                if ui.button("DPI Stage Down").clicked() {
                    new_action = Some(ButtonAction::Sensitivity(SensitivityAction::DpiDown));
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Sniper Clutch:").small().color(theme::MUTED));
                for clutch_dpi in [200, 400, 800] {
                    if ui.button(format!("{clutch_dpi} DPI")).clicked() {
                        new_action = Some(ButtonAction::Sensitivity(SensitivityAction::SniperClutch { clutch_dpi }));
                    }
                }
            });

            if matches!(current_action, ButtonAction::Sensitivity(SensitivityAction::CycleDpi)) {
                let stage_seq = prof.dpi_stages.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" -> ");
                ui.label(
                    egui::RichText::new(format!("Configured Sequence: [ {stage_seq} ] DPI"))
                        .small()
                        .color(theme::ACCENT),
                );
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Option 5: Multimedia
            ui.label(egui::RichText::new("Media & Audio").small().color(theme::MUTED));
            ui.horizontal_wrapped(|ui| {
                let media = [
                    (MediaAction::PlayPause, "Play/Pause"),
                    (MediaAction::VolumeUp, "Vol +"),
                    (MediaAction::VolumeDown, "Vol -"),
                    (MediaAction::Mute, "Mute"),
                    (MediaAction::NextTrack, "Next"),
                    (MediaAction::PrevTrack, "Prev"),
                ];
                for (ma, label) in media {
                    if ui.button(label).clicked() {
                        new_action = Some(ButtonAction::Multimedia(ma));
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Option 6: Windows Shortcuts
            ui.label(egui::RichText::new("Windows Shortcuts").small().color(theme::MUTED));
            ui.horizontal_wrapped(|ui| {
                let shortcuts = [
                    (WinShortcut::LockPc, "Lock PC"),
                    (WinShortcut::TaskView, "Task View"),
                    (WinShortcut::ShowDesktop, "Show Desktop"),
                    (WinShortcut::OpenExplorer, "Explorer"),
                ];
                for (ws, label) in shortcuts {
                    if ui.button(label).clicked() {
                        new_action = Some(ButtonAction::WindowsShortcut(ws));
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Option 7: Disable
            if ui.button("Disable Button").clicked() {
                new_action = Some(ButtonAction::Disabled);
            }

            if let Some(action) = new_action {
                prof.set_button_action(btn_id, action);
                let actions = prof.button_mappings.clone();
                let _ = self.config.save();
                self.input_hook.update_mappings(actions);
                self.set_status(format!("Updated {}", btn_id.short_name()), false);
            }
        });

        if trigger_dpi_cycle {
            self.cycle_dpi_stage();
        }
        if trigger_profile_cycle {
            self.cycle_active_profile();
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // TAB 2: PERFORMANCE (DPI, Stages, Polling Rate)
    // ═════════════════════════════════════════════════════════════════════
    fn ui_tab_performance(&mut self, ui: &mut egui::Ui, profile: &'static DeviceProfile) {
        ui.heading("Performance & Sensitivity");
        ui.add_space(14.0);

        if let Some(dpi) = profile.dpi {
            widgets::card(ui, "Sensitivity Stages (DPI Cycle)", |ui| {
                let Some(pid) = self.active_pid else { return };

                let (dpi_stages, current_idx) = {
                    let dev_entry = self.config.devices.entry(pid).or_default();
                    let prof = dev_entry.ensure_profiles().active_profile();
                    (prof.dpi_stages.clone(), prof.current_stage_idx)
                };

                // Number of Stages selector bar (Razer Synapse style: 1 to 5 stages)
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Number of Stages:").strong().color(Color32::WHITE));
                    ui.add_space(8.0);
                    for count in 1..=5 {
                        let is_curr = dpi_stages.len() == count;
                        if ui.selectable_label(is_curr, format!("{count}")).clicked() {
                            self.set_dpi_stage_count(count, dpi.min, dpi.max);
                        }
                    }
                    ui.add_space(16.0);
                    if ui.button("Cycle Next Stage").on_hover_text("Advance hardware DPI to the next configured stage").clicked() {
                        self.cycle_dpi_stage();
                    }
                });

                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Mouse buttons configured for 'DPI Cycle' or 'DPI Up/Down' will sequentially cycle through these configured stages.")
                        .small()
                        .color(theme::MUTED),
                );
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                let mut stage_to_activate = None;
                let mut stage_to_remove = None;
                let mut stage_edit = None;

                for (idx, &stage_dpi) in dpi_stages.iter().enumerate() {
                    let is_active = current_idx == idx;

                    egui::Frame::none()
                        .fill(if is_active {
                            Color32::from_rgb(20, 34, 26)
                        } else {
                            Color32::from_rgb(26, 27, 31)
                        })
                        .stroke(egui::Stroke::new(
                            if is_active { 1.5_f32 } else { 1.0_f32 },
                            if is_active { theme::ACCENT } else { Color32::from_rgb(46, 48, 56) },
                        ))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Stage Number & Status Badge
                                if is_active {
                                    ui.label(
                                        egui::RichText::new(format!("STAGE {}", idx + 1))
                                            .strong()
                                            .color(Color32::WHITE),
                                    );
                                    egui::Frame::none()
                                        .fill(Color32::from_rgb(18, 52, 32))
                                        .stroke(egui::Stroke::new(1.0_f32, theme::ACCENT))
                                        .rounding(4.0)
                                        .inner_margin(egui::Margin::symmetric(7.0, 2.5))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new("ACTIVE")
                                                    .small()
                                                    .strong()
                                                    .color(theme::ACCENT),
                                            );
                                        });
                                } else {
                                    ui.label(
                                        egui::RichText::new(format!("STAGE {}", idx + 1))
                                            .strong()
                                            .color(Color32::from_rgb(160, 165, 175)),
                                    );
                                    let act_btn = egui::Button::new(
                                        egui::RichText::new("Activate")
                                            .small()
                                            .color(Color32::from_rgb(200, 205, 215)),
                                    );
                                    if ui.add(act_btn).clicked() {
                                        stage_to_activate = Some(idx);
                                    }
                                }

                                ui.add_space(12.0);

                                // Slider / DragValue for this stage's DPI
                                let mut val = stage_dpi;
                                let drag_res = ui.add(
                                    egui::DragValue::new(&mut val)
                                        .range(dpi.min..=dpi.max)
                                        .speed(50.0)
                                        .suffix(" DPI"),
                                );
                                if drag_res.changed() {
                                    stage_edit = Some((idx, val));
                                }

                                ui.add_space(8.0);
                                let slider_res = ui.add(
                                    egui::Slider::new(&mut val, dpi.min..=dpi.max)
                                        .step_by(50.0)
                                        .show_value(false),
                                );
                                if slider_res.changed() {
                                    stage_edit = Some((idx, val));
                                }

                                // Step buttons: -100, +100
                                if ui.small_button("-100").clicked() {
                                    let new_val = (stage_dpi.saturating_sub(100)).max(dpi.min);
                                    stage_edit = Some((idx, new_val));
                                }
                                if ui.small_button("+100").clicked() {
                                    let new_val = (stage_dpi.saturating_add(100)).min(dpi.max);
                                    stage_edit = Some((idx, new_val));
                                }

                                // Delete Stage Button (if > 1 stage)
                                if dpi_stages.len() > 1 {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let del_btn = egui::Button::new(
                                            egui::RichText::new("Delete")
                                                .small()
                                                .color(Color32::from_rgb(225, 95, 95)),
                                        );
                                        if ui.add(del_btn).on_hover_text("Delete this DPI stage").clicked() {
                                            stage_to_remove = Some(idx);
                                        }
                                    });
                                }
                            });
                        });
                    ui.add_space(8.0);
                }

                // Apply pending actions
                if let Some(idx) = stage_to_activate {
                    self.set_dpi_stage(idx);
                }
                if let Some((idx, new_val)) = stage_edit {
                    self.set_dpi_stage_value(idx, new_val);
                }
                if let Some(idx) = stage_to_remove {
                    self.remove_dpi_stage(idx);
                }

                ui.add_space(8.0);

                // Stage Management Action Buttons
                ui.horizontal(|ui| {
                    if dpi_stages.len() < 5 {
                        let add_btn = egui::Button::new(
                            egui::RichText::new("+ Add Stage").color(theme::ACCENT),
                        );
                        if ui.add(add_btn).on_hover_text("Add another DPI stage (up to 5 stages)").clicked() {
                            self.add_dpi_stage(dpi.min, dpi.max);
                        }
                    }
                    if ui.button("Sort Low to High").on_hover_text("Sort stages in ascending order").clicked() {
                        self.sort_dpi_stages();
                    }
                    if ui.button("Reset Defaults (400 - 6400)").on_hover_text("Reset to standard Razer 5-stage defaults").clicked() {
                        self.reset_dpi_stages_to_default();
                    }
                });
            });

            ui.add_space(14.0);

            // Direct Sensitivity Override Card
            widgets::card(ui, "Direct Sensitivity Override", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Custom DPI:");
                    ui.add(
                        egui::DragValue::new(&mut self.custom_dpi_input)
                            .range(dpi.min as u32..=dpi.max as u32)
                            .speed(50.0)
                            .suffix(" DPI"),
                    );
                    if ui.button("Set DPI Immediately").clicked() {
                        self.apply_dpi_value(self.custom_dpi_input as u16);
                    }
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!("Current: {} DPI", self.dpi_x))
                            .strong()
                            .color(theme::ACCENT),
                    );
                });

                ui.add_space(10.0);
                ui.label(egui::RichText::new("Quick Presets").small().color(theme::MUTED));
                ui.horizontal_wrapped(|ui| {
                    for preset in [400, 800, 1200, 1600, 2400, 3200, 6400, 12000, 16000, 20000] {
                        if preset >= dpi.min && preset <= dpi.max {
                            let is_curr = self.dpi_x == preset as u16;
                            if ui.selectable_label(is_curr, preset.to_string()).clicked() {
                                self.apply_dpi_value(preset as u16);
                            }
                        }
                    }
                });
            });
        }

        if let Some(poll) = profile.poll_rate {
            widgets::card(ui, "Polling Rate", |ui| {
                ui.horizontal(|ui| {
                    for hz in poll.supported_hz {
                        let is_current = self.current_poll_hz == Some(*hz);
                        if ui.selectable_label(is_current, format!("{hz} Hz")).clicked() {
                            let result = {
                                let dev_guard = self.runtime_state.device.lock().unwrap();
                                dev_guard
                                    .as_ref()
                                    .map(|d| d.set_poll_rate(*hz))
                                    .unwrap_or_else(|| Err(razer_hid_client::ClientError::Device("no device connected".into())))
                            };
                            if result.is_ok() {
                                self.current_poll_hz = Some(*hz);
                                self.save_current_device_config();
                            }
                            self.apply(result, &format!("Polling rate set to {hz}Hz"));
                        }
                    }
                });
            });
        }

        if profile.identity.is_some() {
            self.ui_info_card(ui);
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // TAB 3: LIGHTING (Chroma RGB & Underglow)
    // ═════════════════════════════════════════════════════════════════════
    fn ui_tab_lighting(&mut self, ui: &mut egui::Ui, profile: &'static DeviceProfile) {
        ui.heading("Lighting & Chroma Effects");
        ui.add_space(14.0);

        self.ui_lighting_card(ui, profile);

        if profile.matrix.is_some() && !self.sync_zones {
            self.ui_matrix_card(ui);
        }
    }

    // ═════════════════════════════════════════════════════════════════════
    // TAB 4: PROFILES (Profile Management & Switching)
    // ═════════════════════════════════════════════════════════════════════
    fn ui_tab_profiles(&mut self, ui: &mut egui::Ui) {
        ui.heading("Profile Management");
        ui.add_space(14.0);

        widgets::card(ui, "Stored Profiles", |ui| {
            let pid = match self.active_pid {
                Some(pid) => pid,
                None => return,
            };

            let dev_entry = self.config.devices.entry(pid).or_default();
            let prof_cfg = dev_entry.ensure_profiles();
            let active_id = prof_cfg.active_profile_id.clone();

            let mut switch_to = None;
            let mut duplicate = false;
            let mut delete_id = None;

            let profiles = prof_cfg.profiles.clone();
            for prof in &profiles {
                let is_active = prof.id == active_id;

                egui::Frame::none()
                    .fill(if is_active { theme::CARD_BG_HOVER } else { theme::CARD_BG })
                    .rounding(8.0)
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            widgets::status_dot(ui, is_active);
                            ui.label(
                                egui::RichText::new(&prof.name)
                                    .strong()
                                    .color(if is_active { theme::ACCENT } else { Color32::WHITE }),
                            );
                            ui.label(
                                egui::RichText::new(format!("— DPI: {}, {} Hz", prof.dpi, prof.poll_rate_hz.unwrap_or(1000)))
                                    .small()
                                    .color(theme::MUTED),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if !is_active {
                                    if ui.button("Activate").clicked() {
                                        switch_to = Some(prof.id.clone());
                                    }
                                    if ui.button("Delete").clicked() {
                                        delete_id = Some(prof.id.clone());
                                    }
                                } else {
                                    ui.label(egui::RichText::new("ACTIVE").small().color(theme::ACCENT));
                                }
                            });
                        });
                    });
                ui.add_space(4.0);
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Duplicate Active Profile").clicked() {
                    duplicate = true;
                }
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);

            // Create New Profile
            ui.horizontal(|ui| {
                ui.label("New Profile Name:");
                ui.text_edit_singleline(&mut self.new_profile_input);
                if ui.button("Create Profile").clicked() && !self.new_profile_input.trim().is_empty() {
                    let name = self.new_profile_input.trim().to_string();
                    let new_id = prof_cfg.create_profile(&name);
                    self.new_profile_input.clear();
                    switch_to = Some(new_id);
                }
            });

            if duplicate {
                let new_id = prof_cfg.duplicate_active();
                switch_to = Some(new_id);
            }

            if let Some(del) = delete_id {
                prof_cfg.delete_profile(&del);
                let _ = self.config.save();
                self.set_status("Profile deleted", false);
            }

            if let Some(target) = switch_to {
                self.switch_profile(&target);
            }
        });
    }

    // ═════════════════════════════════════════════════════════════════════
    // TAB 5: SETTINGS (Windows autostart, tray, system config)
    // ═════════════════════════════════════════════════════════════════════
    fn ui_tab_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Application Settings");
        ui.add_space(14.0);

        self.ui_system_settings_card(ui);
    }

    fn ui_lighting_card(&mut self, ui: &mut egui::Ui, profile: &'static DeviceProfile) {
        widgets::card(ui, "Lighting", |ui| {
            let zones = profile.leds;
            let zone_idx = self.selected_zone_idx.min(zones.len() - 1);

            let has_matrix = profile.matrix.is_some();
            let matrix_has_effects = profile.matrix.and_then(|m| m.effect_led_id).is_some();
            let supports = |effect: razer_hid_abi::commands::LedEffect| {
                !profile.unsupported_led_effects.contains(&effect)
            };

            if zones.len() > 1 || has_matrix {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sync_zones, true, "All zones");
                    ui.selectable_value(&mut self.sync_zones, false, "Individually");
                });
                ui.add_space(6.0);

                if !self.sync_zones {
                    if zones.len() > 1 {
                        egui::ComboBox::from_label("Zone")
                            .selected_text(zones[zone_idx].name)
                            .show_ui(ui, |ui| {
                                for (i, z) in zones.iter().enumerate() {
                                    ui.selectable_value(&mut self.selected_zone_idx, i, z.name);
                                }
                            });
                    }
                }
            }

            let any_brightness = if self.sync_zones {
                zones.iter().any(|z| z.brightness_command_id.is_some())
            } else {
                zones[zone_idx].brightness_command_id.is_some()
            };

            widgets::effect_preview(ui, self.last_preview, self.led_color, ui.input(|i| i.time));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(&mut self.led_color);
                widgets::color_swatch(ui, self.led_color, 20.0);
            });
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                if ui.button("Static").clicked() {
                    let color = self.led_color;
                    self.last_preview = PreviewKind::Static;
                    self.current_effect_name = Some("static".to_string());
                    self.apply_to_zones_and_matrix(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_static(zone, color),
                        has_matrix.then_some(move |dev: &RazerDevice| dev.fill_matrix(color)),
                        "LED set to static color",
                    );
                    self.save_current_device_config();
                }
                if supports(razer_hid_abi::commands::LedEffect::Breathing)
                    && ui.button("Breathing").clicked()
                {
                    let color = self.led_color;
                    self.last_preview = PreviewKind::Breathing;
                    self.current_effect_name = Some("breathing".to_string());
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_breathing(zone, color),
                        "LED breathing",
                    );
                    self.save_current_device_config();
                }
                if ui.button("Spectrum").clicked() {
                    self.last_preview = PreviewKind::Spectrum;
                    self.current_effect_name = Some("spectrum".to_string());
                    self.apply_to_zones_and_matrix(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_spectrum(zone),
                        matrix_has_effects.then_some(|dev: &RazerDevice| dev.set_matrix_spectrum()),
                        "LED cycling spectrum",
                    );
                    self.save_current_device_config();
                }
                if ui.button("Wave").clicked() {
                    self.last_preview = PreviewKind::Wave;
                    self.current_effect_name = Some("wave".to_string());
                    self.apply_to_zones_and_matrix(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_wave(zone, 1),
                        matrix_has_effects.then_some(|dev: &RazerDevice| dev.set_matrix_wave(1)),
                        "LED wave",
                    );
                    self.save_current_device_config();
                }
                if ui.button("Off").clicked() {
                    self.last_preview = PreviewKind::Off;
                    self.current_effect_name = Some("off".to_string());
                    self.apply_to_zones_and_matrix(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_off(zone),
                        has_matrix.then_some(|dev: &RazerDevice| dev.fill_matrix([0, 0, 0])),
                        "LED off",
                    );
                    self.save_current_device_config();
                }
            });

            if any_brightness {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Brightness")
                        .color(theme::MUTED)
                        .small(),
                );
                let slider = ui.add(egui::Slider::new(&mut self.brightness, 0..=100).suffix("%"));
                if slider.drag_stopped() || slider.lost_focus() {
                    let b = self.brightness;
                    self.apply_to_zones(
                        zones,
                        zone_idx,
                        |dev, zone| dev.set_led_brightness(zone, b),
                        "Brightness updated",
                    );
                    self.save_current_device_config();
                }
            }
        });
    }

    fn ui_matrix_card(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, "Underglow", |ui| {
            let dims = {
                let dev_guard = self.runtime_state.device.lock().unwrap();
                dev_guard.as_ref().and_then(|d| d.matrix_dims())
            };
            let Some((rows, cols)) = dims else {
                return;
            };
            ui.label(
                egui::RichText::new(format!("{rows} row x {cols} individually addressable LEDs"))
                    .small()
                    .color(theme::MUTED),
            );
            ui.add_space(6.0);

            widgets::effect_preview(ui, PreviewKind::Static, self.matrix_color, 0.0);
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(&mut self.matrix_color);
                widgets::color_swatch(ui, self.matrix_color, 20.0);
            });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Fill").clicked() {
                    let color = self.matrix_color;
                    let r = {
                        let dev_guard = self.runtime_state.device.lock().unwrap();
                        dev_guard.as_ref().map(|d| d.fill_matrix(color)).unwrap_or(Ok(()))
                    };
                    self.apply(r, "Underglow filled");
                    self.save_current_device_config();
                }
                if ui.button("Off").clicked() {
                    let r = {
                        let dev_guard = self.runtime_state.device.lock().unwrap();
                        dev_guard.as_ref().map(|d| d.fill_matrix([0, 0, 0])).unwrap_or(Ok(()))
                    };
                    self.apply(r, "Underglow off");
                    self.save_current_device_config();
                }
            });
        });
    }

    fn ui_info_card(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, "Device Info", |ui| {
            if ui.button("Refresh Details").clicked() {
                let (fw, serial) = {
                    let dev_guard = self.runtime_state.device.lock().unwrap();
                    if let Some(dev) = dev_guard.as_ref() {
                        (dev.get_firmware_version(), dev.get_serial())
                    } else {
                        (
                            Err(razer_hid_client::ClientError::Device("no device connected".into())),
                            Err(razer_hid_client::ClientError::Device("no device connected".into())),
                        )
                    }
                };
                match (fw, serial) {
                    (Ok(fw), Ok(serial)) => {
                        self.info_text = Some(format!("Firmware: {fw}\nSerial:   {serial}"));
                        self.set_status("Info refreshed", false);
                    }
                    (Err(e), _) | (_, Err(e)) => self.set_status(format!("Error: {e}"), true),
                }
            }
            if let Some(text) = &self.info_text {
                ui.add_space(8.0);
                ui.monospace(text);
            }
        });
    }

    fn ui_system_settings_card(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, "System & Startup", |ui| {
            let mut autostart_changed = false;
            let mut close_tray_changed = false;

            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.start_with_windows, "Start with Windows")
                    .changed()
                {
                    autostart_changed = true;
                }
            });
            ui.label(
                egui::RichText::new("Automatically launch RazerLine in the system tray on boot.")
                    .small()
                    .color(theme::MUTED),
            );

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.close_to_tray, "Minimize to tray on close")
                    .changed()
                {
                    close_tray_changed = true;
                }
            });
            ui.label(
                egui::RichText::new("Keep running in the background when clicking 'X'.")
                    .small()
                    .color(theme::MUTED),
            );

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Save Settings as Default").clicked() {
                    self.save_current_device_config();
                    self.set_status("Configured settings saved", false);
                }
                if ui.button("Reload from Disk").clicked() {
                    self.config = AppConfig::load();
                    if let Some(profile) = self
                        .connected
                        .iter()
                        .find(|p| Some(p.product_id) == self.active_pid)
                        .copied()
                    {
                        self.open(profile);
                    }
                    self.set_status("Settings reloaded from disk", false);
                }
            });

            if autostart_changed {
                let enabled = self.config.start_with_windows;
                if let Err(e) = crate::autostart::set_autostart(enabled) {
                    self.set_status(format!("Autostart setting failed: {e}"), true);
                } else {
                    let msg = if enabled {
                        "Start with Windows enabled (runs in background on boot)"
                    } else {
                        "Start with Windows disabled"
                    };
                    self.set_status(msg, false);
                    let _ = self.config.save();
                }
            }

            if close_tray_changed {
                let _ = self.config.save();
            }

            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(format!(
                    "Config file: {}",
                    AppConfig::file_path().display()
                ))
                .small()
                .color(theme::MUTED),
            );
        });
    }
}

pub fn run(start_in_background: bool) -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("bundled icon.png is a valid PNG — generated from assets/razer-inspired-logo.svg");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 740.0])
            .with_min_inner_size([880.0, 600.0])
            .with_icon(icon)
            .with_visible(!start_in_background),
        ..Default::default()
    };
    eframe::run_native(
        "RazerLine",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(GuiApp::new()))
        }),
    )
}

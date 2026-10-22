//! Shared runtime state, registered with Tauri via `app.manage(...)` and
//! reached from commands via `tauri::State<AppState>`. Mirrors the shape of
//! the old egui GUI's `DpiRuntimeState` (`crates/gui/src/app.rs`) — now
//! includes the persistent config/profile system (Phase 4).

use std::sync::atomic::{AtomicU16, AtomicU32};
use std::sync::{Arc, Mutex};

use crate::config::AppConfig;
use razer_hid_client::RazerDevice;

#[derive(Clone)]
pub struct AppState {
    pub device: Arc<Mutex<Option<RazerDevice>>>,
    /// 0 means "no device open". Stored as `u32` (not `u16`) purely so it
    /// has an `AtomicU32`/`AtomicU16` split matching the old code — not a
    /// meaningful distinction, `product_id` itself is always a `u16`.
    pub active_pid: Arc<AtomicU32>,
    pub current_dpi: Arc<AtomicU16>,
    /// Persistent application config + per-device profiles.
    pub config: Arc<Mutex<AppConfig>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            device: Arc::new(Mutex::new(None)),
            active_pid: Arc::new(AtomicU32::new(0)),
            current_dpi: Arc::new(AtomicU16::new(0)),
            config: Arc::new(Mutex::new(AppConfig::load())),
        }
    }
}

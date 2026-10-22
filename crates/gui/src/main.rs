//! Desktop control panel. Generic over whatever `DeviceProfile` is
//! connected — sections only appear for capabilities the connected
//! device's profile actually declares.
//!
//! The attribute below stops a console window from appearing behind the
//! GUI on Windows release builds (Rust binaries default to the "console"
//! subsystem there). Debug builds keep the console so `println!`/panic
//! output is still visible while developing; it's a no-op on Linux/macOS.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod autostart;
mod buttons;
mod config;
mod input_hook;
mod mouse_3d;
mod osd;
mod profiles;
mod theme;
mod tray;
mod widgets;
mod win_util;

/// tray-icon's Linux/BSD backend (GTK 3 + AppIndicator) panics ("GTK has
/// not been initialized") if a menu is built before `gtk::init()` runs.
/// Must happen on whichever thread calls this (GTK enforces single-
/// thread affinity — `gtk::main()`/`gtk::main_iteration()` later must run
/// on this exact same thread, which is why `app.rs`'s `update()` pumps
/// GTK's queue inline rather than spawning a separate thread for it). A
/// no-op on Windows, where tray-icon talks to native win32 APIs directly
/// and none of this is needed.
#[cfg(not(windows))]
fn init_gtk() {
    gtk::init().expect("failed to initialize GTK (required by tray-icon's Linux/BSD backend)");
}

#[cfg(windows)]
fn init_gtk() {}

fn main() -> eframe::Result<()> {
    // Single instance enforcement: if an instance is already running,
    // bring it to the foreground and exit this duplicate process immediately.
    if !win_util::acquire_single_instance() {
        return Ok(());
    }

    init_gtk();

    // Initialize DPI OSD overlay HUD
    osd::init();

    let args: Vec<String> = std::env::args().collect();
    let start_in_background = args
        .iter()
        .any(|arg| arg == "--background" || arg == "--minimized" || arg == "-b");

    app::run(start_in_background)
}

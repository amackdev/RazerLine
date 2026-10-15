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
mod theme;
mod widgets;

fn main() -> eframe::Result<()> {
    app::run()
}

//! Desktop control panel. Generic over whatever `DeviceProfile` is
//! connected — sections only appear for capabilities the connected
//! device's profile actually declares.

mod app;
mod theme;
mod widgets;

fn main() -> eframe::Result<()> {
    app::run()
}

use clap::{Parser, Subcommand};
use razer_hid_core::devices::{DeviceProfile, KNOWN_DEVICES};
use razer_hid_core::RazerDevice;

#[derive(Parser)]
#[command(
    name = "razer-ctl",
    about = "Unofficial control utility for Razer HID peripherals"
)]
struct Cli {
    /// Select a specific known device by (partial, case-insensitive) name,
    /// e.g. "basilisk". Defaults to the first supported device found.
    #[arg(long, global = true)]
    device: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List known device profiles and which of them are currently connected.
    ListDevices,
    /// Dump every raw HID interface under Razer's vendor id (0x1532),
    /// regardless of whether it's a known profile. Use this to figure out
    /// why `open` can't find a working control interface for your device.
    DebugUsb,
    /// Show firmware version and serial number of the selected device.
    Info,
    /// List the LED zones the selected device exposes.
    ListLeds,
    /// Set sensor DPI. Pass one value for X=Y, or two for independent X/Y.
    Dpi { dpi_x: u16, dpi_y: Option<u16> },
    /// Set USB polling rate in Hz.
    PollRate { hz: u32 },
    /// Control one of the device's LED zones.
    Led {
        #[command(subcommand)]
        action: LedAction,
    },
    /// Control the device's addressable LED matrix/strip (e.g. an
    /// underglow strip), separate from the simple single-color zones.
    Matrix {
        #[command(subcommand)]
        action: MatrixAction,
    },
}

#[derive(Subcommand)]
enum MatrixAction {
    /// Print the matrix's (rows, cols) dimensions.
    Dims,
    /// Fill every LED with one solid color and commit immediately.
    Fill { color: String },
    /// Turn every LED off.
    Off,
    /// Write one or more colors starting at a column, then commit.
    /// Colors are comma-separated hex, e.g. FF0000,00FF00,0000FF
    Row {
        #[arg(long, default_value_t = 0)]
        row: u8,
        #[arg(long, default_value_t = 0)]
        start: u8,
        colors: String,
    },
    /// Hardware-animated Spectrum cycling across the whole matrix. Only
    /// supported on devices where this is confirmed from source/hardware.
    Spectrum,
    /// Hardware-animated Wave across the whole matrix. Only supported on
    /// devices where this is confirmed from source/hardware.
    Wave {
        #[arg(long, default_value_t = 1)]
        direction: u8,
    },
}

#[derive(Subcommand)]
enum LedAction {
    Off {
        #[arg(long)]
        zone: String,
    },
    /// Set a solid color, e.g. `FF0000`.
    Static {
        color: String,
        #[arg(long)]
        zone: String,
    },
    Breathing {
        color: String,
        #[arg(long)]
        zone: String,
    },
    Spectrum {
        #[arg(long)]
        zone: String,
    },
    Wave {
        #[arg(long)]
        zone: String,
        #[arg(long, default_value_t = 1)]
        direction: u8,
    },
    Brightness {
        percent: u8,
        #[arg(long)]
        zone: String,
    },
}

fn hex_rgb(value: &str) -> Result<[u8; 3], String> {
    let value = value.trim_start_matches('#');
    if value.len() != 6 {
        return Err("expected a hex color like FF8800".into());
    }
    let mut rgb = [0u8; 3];
    for (i, chunk) in value.as_bytes().chunks(2).enumerate() {
        let s = std::str::from_utf8(chunk).unwrap();
        rgb[i] = u8::from_str_radix(s, 16).map_err(|e| e.to_string())?;
    }
    Ok(rgb)
}

fn select_profile(name: Option<&str>) -> Result<&'static DeviceProfile, String> {
    let connected = RazerDevice::list_connected().map_err(|e| e.to_string())?;
    match name {
        Some(needle) => connected
            .into_iter()
            .find(|p| p.name.to_lowercase().contains(&needle.to_lowercase()))
            .ok_or_else(|| format!("no connected device matching \"{needle}\"")),
        None => connected
            .into_iter()
            .next()
            .ok_or_else(|| "no supported device connected".to_string()),
    }
}

fn find_zone(
    dev: &RazerDevice,
    name: &str,
) -> Result<razer_hid_core::devices::LedZoneProfile, String> {
    dev.led_zones()
        .iter()
        .find(|z| z.name.to_lowercase() == name.to_lowercase())
        .copied()
        .ok_or_else(|| {
            let available: Vec<_> = dev.led_zones().iter().map(|z| z.name).collect();
            format!("no LED zone named \"{name}\" (available: {available:?})")
        })
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::ListDevices => return list_devices(),
        Command::DebugUsb => return debug_usb(),
        _ => {}
    }

    let profile = select_profile(cli.device.as_deref())?;
    let dev = RazerDevice::open(profile).map_err(|e| e.to_string())?;
    println!("Using {}", profile.name);

    match cli.command {
        Command::ListDevices | Command::DebugUsb => unreachable!(),
        Command::Info => {
            if let Ok(fw) = dev.get_firmware_version() {
                println!("Firmware:    {fw}");
            }
            if let Ok(serial) = dev.get_serial() {
                println!("Serial:      {serial}");
            }
            if let Ok(dpi) = dev.get_dpi() {
                println!("DPI:         {}x{}", dpi.x, dpi.y);
            }
            if let Ok(hz) = dev.get_poll_rate() {
                println!("Poll rate:   {hz}Hz");
            }
        }
        Command::ListLeds => {
            for zone in dev.led_zones() {
                println!("- {}", zone.name);
            }
        }
        Command::Dpi { dpi_x, dpi_y } => {
            let y = dpi_y.unwrap_or(dpi_x);
            dev.set_dpi(dpi_x, y).map_err(|e| e.to_string())?;
            println!("DPI set to {dpi_x}x{y}");
        }
        Command::PollRate { hz } => {
            dev.set_poll_rate(hz).map_err(|e| e.to_string())?;
            println!("Polling rate set to {hz}Hz");
        }
        Command::Led { action } => run_led(&dev, action)?,
        Command::Matrix { action } => run_matrix(&dev, action)?,
    }
    Ok(())
}

fn list_devices() -> Result<(), String> {
    let connected = RazerDevice::list_connected().map_err(|e| e.to_string())?;
    for profile in KNOWN_DEVICES {
        let status = if connected.iter().any(|c| c.product_id == profile.product_id) {
            "connected"
        } else {
            "not connected"
        };
        println!("{} [{:?}] - {}", profile.name, profile.confidence, status);
    }
    Ok(())
}

fn debug_usb() -> Result<(), String> {
    let interfaces = RazerDevice::debug_list_interfaces(razer_hid_core::constants::RAZER_VENDOR_ID)
        .map_err(|e| e.to_string())?;
    if interfaces.is_empty() {
        println!("No HID interfaces found under vendor id 0x1532 (Razer). Is anything plugged in?");
        return Ok(());
    }
    for i in interfaces {
        println!(
            "pid=0x{:04x} iface={} usage_page=0x{:04x} usage=0x{:04x} product={:?}\n  path={}",
            i.product_id, i.interface_number, i.usage_page, i.usage, i.product_string, i.path
        );
    }
    Ok(())
}

/// `--zone all` (case-insensitive) applies to every zone on the device —
/// and the underglow/matrix too, for the actions that support it — with
/// a short delay between writes, matching the GUI's "All zones" mode.
/// Anything else is looked up as a single named zone, as before.
fn is_all_zones(zone: &str) -> bool {
    zone.eq_ignore_ascii_case("all")
}

fn run_led(dev: &RazerDevice, action: LedAction) -> Result<(), String> {
    match action {
        LedAction::Off { zone } => {
            if is_all_zones(&zone) {
                dev.set_led_off_all().map_err(|e| e.to_string())?;
                println!("All zones off");
            } else {
                let zone = find_zone(dev, &zone)?;
                dev.set_led_off(&zone).map_err(|e| e.to_string())?;
                println!("{} LED off", zone.name);
            }
        }
        LedAction::Static { color, zone } => {
            let rgb = hex_rgb(&color)?;
            if is_all_zones(&zone) {
                dev.set_led_static_all(rgb).map_err(|e| e.to_string())?;
                println!("All zones set to #{color}");
            } else {
                let zone = find_zone(dev, &zone)?;
                dev.set_led_static(&zone, rgb).map_err(|e| e.to_string())?;
                println!("{} LED set to #{color}", zone.name);
            }
        }
        LedAction::Breathing { color, zone } => {
            let rgb = hex_rgb(&color)?;
            if is_all_zones(&zone) {
                dev.set_led_breathing_all(rgb).map_err(|e| e.to_string())?;
                println!("All zones breathing #{color}");
            } else {
                let zone = find_zone(dev, &zone)?;
                dev.set_led_breathing(&zone, rgb)
                    .map_err(|e| e.to_string())?;
                println!("{} LED breathing #{color}", zone.name);
            }
        }
        LedAction::Spectrum { zone } => {
            if is_all_zones(&zone) {
                dev.set_led_spectrum_all().map_err(|e| e.to_string())?;
                println!("All zones cycling spectrum");
            } else {
                let zone = find_zone(dev, &zone)?;
                dev.set_led_spectrum(&zone).map_err(|e| e.to_string())?;
                println!("{} LED cycling spectrum", zone.name);
            }
        }
        LedAction::Wave { zone, direction } => {
            if is_all_zones(&zone) {
                dev.set_led_wave_all(direction).map_err(|e| e.to_string())?;
                println!("All zones wave");
            } else {
                let zone = find_zone(dev, &zone)?;
                dev.set_led_wave(&zone, direction)
                    .map_err(|e| e.to_string())?;
                println!("{} LED wave", zone.name);
            }
        }
        LedAction::Brightness { percent, zone } => {
            if is_all_zones(&zone) {
                dev.set_led_brightness_all(percent)
                    .map_err(|e| e.to_string())?;
                println!("All zones brightness set to {percent}%");
            } else {
                let zone = find_zone(dev, &zone)?;
                dev.set_led_brightness(&zone, percent)
                    .map_err(|e| e.to_string())?;
                println!("{} LED brightness set to {percent}%", zone.name);
            }
        }
    }
    Ok(())
}

fn run_matrix(dev: &RazerDevice, action: MatrixAction) -> Result<(), String> {
    match action {
        MatrixAction::Dims => {
            let (rows, cols) = dev
                .matrix_dims()
                .ok_or_else(|| "this device has no addressable LED matrix".to_string())?;
            println!("{rows} row(s) x {cols} column(s)");
        }
        MatrixAction::Fill { color } => {
            let rgb = hex_rgb(&color)?;
            dev.fill_matrix(rgb).map_err(|e| e.to_string())?;
            println!("Matrix filled with #{color}");
        }
        MatrixAction::Off => {
            dev.fill_matrix([0, 0, 0]).map_err(|e| e.to_string())?;
            println!("Matrix off");
        }
        MatrixAction::Row { row, start, colors } => {
            let rgbs = colors
                .split(',')
                .map(hex_rgb)
                .collect::<Result<Vec<_>, _>>()?;
            dev.set_matrix_row(row, start, &rgbs)
                .map_err(|e| e.to_string())?;
            dev.commit_matrix_frame().map_err(|e| e.to_string())?;
            println!(
                "Row {row} columns {start}..{} written",
                start as usize + rgbs.len()
            );
        }
        MatrixAction::Spectrum => {
            dev.set_matrix_spectrum().map_err(|e| e.to_string())?;
            println!("Matrix cycling spectrum");
        }
        MatrixAction::Wave { direction } => {
            dev.set_matrix_wave(direction).map_err(|e| e.to_string())?;
            println!("Matrix wave");
        }
    }
    Ok(())
}

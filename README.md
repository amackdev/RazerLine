<p align="center">
  <img src="assets/razer-inspired-logo.svg" width="96" height="96" alt="RazerLine logo">
</p>

# RazerLine

_(repository: razer-hid-ctl)_

[![CI](https://github.com/amackdev/razer-hid-ctl/actions/workflows/ci.yml/badge.svg)](https://github.com/amackdev/razer-hid-ctl/actions/workflows/ci.yml)

An unofficial, open-source control suite for Razer HID peripherals —
built so you don't need Razer Synapse installed. Started for the Razer
Basilisk V3, but structured as a general framework: any device gets
support by adding one data file describing its capabilities, not by
forking the whole project. `razerline` is the desktop GUI; `razer-ctl`
is the scriptable CLI — both built on the same device-agnostic core
library, so this is meant to scale past one person's one mouse.

**Not affiliated with or endorsed by Razer Inc.** Built from the public
USB HID protocol reverse-engineered by the [OpenRazer](https://github.com/openrazer/openrazer)
community (GPL-3.0). See "Adding support for your device" below for how
to extend or verify the protocol for a new model.

## What works today

**95 mouse device profiles** are included — run `razer-ctl list-devices`
for the full list. Every one of them was built the same way: cross-checked
field-by-field against OpenRazer's kernel driver source
(`driver/razermouse_driver.c`, `razerchromacommon.c`) and daemon metadata
(`daemon/openrazer_daemon/hardware/mouse.py`), not guessed from a
similarly-named model. Each device is its own commit — see `git log` for
exactly what was added for any specific mouse.

- **Razer Basilisk V3** (wired) is `Confidence::Verified` — DPI, polling
  rate, Static/Spectrum/Wave LED effects, brightness, and its 11-LED
  underglow strip are confirmed against real hardware. Breathing and Off
  are confirmed *rejected* by the firmware itself for this exact model —
  not a bug, OpenRazer's own driver doesn't attempt them for this PID
  either.
- **Every other device is `Confidence::Unverified`** — correct as far as
  the source supports, but not yet run against real hardware by anyone
  on this project. If you own one, please try it and report back either
  way (see "Adding support for your device" below).

Deliberately **not** included (117 total mouse PIDs exist in OpenRazer;
22 were excluded, not silently mis-modeled):

- Wireless receivers/dongles (they're not the mouse itself).
- A handful of very old mice (original DeathAdder/Naga/Orochi-era) that
  use a completely different byte-indexed DPI protocol this project
  doesn't model, to avoid shipping a profile that's confidently wrong.

Capabilities modeled so far (per-device, only shown if the device profile
declares them):

- DPI (independent X/Y)
- USB polling rate
- Per-zone RGB lighting: static color, breathing, spectrum, wave, off,
  brightness
- An addressable LED matrix/strip (e.g. underglow) separate from the
  simple zones above, for devices with more than one individually
  controllable LED in a strip
- Firmware version / serial number query

Not yet in scope: per-key keyboard RGB matrices, on-device macros/profile
storage, firmware updates.

## Why Rust

The control logic talks directly to raw USB HID feature reports — exactly
the kind of code where a buffer-overrun or an unchecked index turns into a
real bug, not just a crash. Rust's ownership model catches that class of
mistake at compile time without giving up native performance or a single,
dependency-free binary you can hand to someone else. `hidapi` (via the
`hidapi` crate) gives the same HID transport on Linux, Windows, and macOS,
so one codebase covers all three.

## Project layout

```
crates/
  protocol/  razer-hid-protocol — byte-level report framing/CRC, command
                                   vocabulary. No transport dependency.
  core/      razer-hid-core     — RazerDevice control engine + DeviceProfile
                                   shape, built on razer-hid-protocol. Knows
                                   nothing about any specific device model.
  devices/   razer-hid-devices  — the registry of supported models (one
                                   DeviceProfile per file), built on
                                   razer-hid-core's types.
  cli/       razer-hid-cli      — `razer-ctl` command-line tool
  gui/       razer-hid-gui      — `razerline` desktop control panel (egui)
assets/   razer-inspired-logo.svg — original artwork, not Razer's mark
udev/     Linux udev rule for non-root HID access
.github/workflows/  CI: fmt/clippy/build/test on Linux, Windows, macOS
```

Dependencies only point one way: `protocol ← core ← devices ← {cli, gui}`.
Core never depends on the device registry — `RazerDevice::list_connected`/
`open_first_supported` take a `&'static [DeviceProfile]` slice as an
argument rather than reaching for a baked-in list, so the control engine
stays usable (and testable) without the whole 90+ device database, and
the database can grow independently without touching core.

The CLI and GUI are both written generically against
`razer_hid_core::DeviceProfile` — neither one hardcodes "Basilisk V3"
anywhere. Adding a new device to the registry makes it show up in both
automatically, with only the capabilities that device actually declares.

## Building

Requires the Rust toolchain ([rustup.rs](https://rustup.rs)).

**Linux** also needs `libudev` headers and a GUI toolkit for egui:

```sh
sudo apt install libudev-dev pkg-config libxkbcommon-dev libgtk-3-dev   # Debian/Ubuntu
```

**Windows / macOS**: no extra system packages needed.

```sh
cargo build --release --workspace
```

Binaries land in `target/release/`: `razer-ctl` (CLI) and `razerline`
(GUI).

### Linux device permissions

HID access needs either root or a udev rule granting your user access:

```sh
sudo cp udev/99-razer-hid-ctl.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
sudo usermod -aG plugdev $USER   # log out/in afterwards
```

## Using the CLI

```sh
razer-ctl list-devices                 # known profiles + connection status
razer-ctl debug-usb                    # dump raw HID interfaces if `open` can't find a working one
razer-ctl info                         # firmware/serial of the connected device
razer-ctl list-leds                    # LED zone names this device exposes
razer-ctl dpi 1600                     # DPI X=Y=1600
razer-ctl dpi 1600 800                 # independent X/Y
razer-ctl poll-rate 1000
razer-ctl led static FF0000 --zone "Scroll wheel"
razer-ctl led breathing 00FF00 --zone Logo
razer-ctl led spectrum --zone Logo
razer-ctl led brightness 50 --zone Logo
razer-ctl led off --zone Logo

# Addressable LED matrix/strip (e.g. underglow), where the device has one:
razer-ctl matrix dims                  # e.g. "1 row(s) x 11 column(s)"
razer-ctl matrix fill FF0000           # every LED to one color
razer-ctl matrix off
razer-ctl matrix row --start 0 FF0000,00FF00,0000FF   # per-LED colors

# If more than one supported device is connected:
razer-ctl --device basilisk dpi 1600
```

## Using the GUI (RazerLine)

```sh
razerline
```

RazerLine polls for connected devices roughly once a second and updates
live — plug a supported device in and it appears in the sidebar
automatically, no "Rescan" button, no restart. The sidebar only lists
devices that are *actually connected* right now, not every profile this
project happens to support. The main panel groups controls into cards
(Performance, Lighting, Underglow, Device info) that only appear for
capabilities the connected device's profile actually declares — a device
with no RGB zones simply has no Lighting card. The Lighting card includes
a live animated preview of the selected color/effect (including a
breathing pulse and a cycling spectrum sweep) so you can see roughly what
an effect looks like before sending it.

The GUI code is in `crates/gui/src/{app,theme,widgets}.rs` — `theme.rs`
holds every color constant in one place, `widgets.rs` has the reusable
pieces (status dot, color swatch, card frame, effect preview), and
`app.rs` wires them together. Swap `theme.rs`'s constants to re-skin it
without touching layout code.

## Adding support for your device

1. Copy `crates/devices/src/_template.rs` to a new file named after
   your device (e.g. `deathadder_v3.rs`) — it has inline comments
   explaining each field.
2. Find your device's USB vendor/product id (`lsusb -d 1532:` on Linux,
   Device Manager on Windows, `ioreg -p IOUSB -l -w0 | grep -i razer` on
   macOS).
3. Fill in the `DeviceProfile`: which capabilities it has
   (`dpi`/`poll_rate`/`leds`/`matrix`/`identity`), set the rest to
   `None`/`&[]`. **Check the transaction id per command type, not just
   per device** — some devices (e.g. the Viper Ultimate) use a different
   one for LED commands than for DPI/poll-rate/identity. Cross-reference
   OpenRazer's kernel driver source (`driver/razermouse_driver.c`,
   `razerchromacommon.c`) for your exact `USB_DEVICE_ID_RAZER_*` PID, and
   its daemon metadata (`daemon/openrazer_daemon/hardware/*.py`) for
   facts the driver doesn't state directly (`MATRIX_DIMS`, `DPI_MAX`).
4. Test against real hardware with the CLI (`razer-ctl list-devices`,
   `razer-ctl dpi ...`, etc.). If a command fails or does nothing,
   capture USB traffic with Wireshark (`usbmon` on Linux, USBPcap on
   Windows) and diff it against what this project sends.
5. Set `confidence: Confidence::Verified` only for capabilities you
   personally tested working. Leave the rest `Unverified`.
6. Register it in `crates/devices/src/lib.rs` (`mod` + `pub use` +
   add to `KNOWN_DEVICES`).
7. Run `cargo fmt --all`, `cargo clippy --workspace --all-targets`, and
   `cargo test --workspace` — CI runs the same checks on Linux/Windows/
   macOS.
8. Open a PR describing what you tested and what you couldn't. Even a
   report with no code ("I tested `<device>`, DPI works, breathing does
   nothing") is useful.

If your device needs a capability that doesn't fit `dpi`/`poll_rate`/
`leds`/`matrix` at all (e.g. per-key keyboard RGB), that's a sign the
model in `devices/mod.rs` needs a new type — propose it in your PR.

## License

GPL-3.0-or-later — see `LICENSE`. Chosen to match OpenRazer, whose
reverse-engineering work this protocol implementation builds on.

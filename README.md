# razer-hid-ctl

An unofficial, open-source control utility for Razer HID peripherals —
built so you don't need Razer Synapse installed. Started for the Razer
Basilisk V3, but structured as a general framework: any device gets
support by adding one data file describing its capabilities, not by
forking the whole project.

**Not affiliated with or endorsed by Razer Inc.** Built from the public
USB HID protocol reverse-engineered by the [OpenRazer](https://github.com/openrazer/openrazer)
community (GPL-3.0). See `docs/PROTOCOL.md` for protocol details and how
to verify/extend them, and `CONTRIBUTING.md` for adding your own device.

## What works today

Currently one device profile is included: **Razer Basilisk V3** (wired),
marked `Unverified` — the command bytes follow the pattern most Razer mice
use, but haven't been confirmed against this exact model by a maintainer
yet. If you own one, please try it and open an issue either way.

Capabilities modeled so far (per-device, only shown if the device profile
declares them):

- DPI (independent X/Y)
- USB polling rate
- Per-zone RGB lighting: static color, breathing, spectrum, wave, off,
  brightness
- Firmware version / serial number query

Not yet in scope: per-key keyboard RGB matrices, on-device macros/profile
storage, firmware updates. See "Scope limits" in `docs/PROTOCOL.md`.

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
  core/   razer-hid-core   — protocol framing, command vocabulary, device
                             registry, generic control logic
  cli/    razer-hid-cli    — `razer-ctl` command-line tool
  gui/    razer-hid-gui    — `razer-ctl-gui` desktop control panel (egui)
udev/     Linux udev rule for non-root HID access
docs/     protocol notes
```

The CLI and GUI are both written generically against
`crates/core/src/devices::DeviceProfile` — neither one hardcodes "Basilisk
V3" anywhere. Adding a new device to the registry makes it show up in both
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

Binaries land in `target/release/`: `razer-ctl` (CLI) and `razer-ctl-gui`
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

# If more than one supported device is connected:
razer-ctl --device basilisk dpi 1600
```

## Using the GUI

```sh
razer-ctl-gui
```

It scans for any connected, known device on launch, and shows only the
tabs/controls that device's profile declares (e.g. no lighting section for
a device with no RGB zones).

## Adding support for your device

See `CONTRIBUTING.md` — the short version is: copy
`crates/core/src/devices/_template.rs`, fill in your device's VID/PID and
capabilities, verify against real hardware, register it, open a PR.

## License

GPL-3.0-or-later — see `LICENSE`. Chosen to match OpenRazer, whose
reverse-engineering work this protocol implementation builds on.

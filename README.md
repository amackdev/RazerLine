<p align="center">
  <img src="assets/razer-inspired-logo.svg" width="108" height="108" alt="RazerLine logo">
</p>

<h1 align="center">RazerLine</h1>

<p align="center">
  <strong>The ultra-lightweight, open-source Windows control center for Razer gaming peripherals.</strong><br>
  <em>Engineered in pure Rust — zero bloatware, zero telemetry, zero mandatory cloud logins.</em>
</p>

<p align="center">
  <a href="https://github.com/amackdev/RazerLine/actions/workflows/ci.yml"><img src="https://github.com/amackdev/RazerLine/actions/workflows/ci.yml/badge.svg" alt="CI Status"></a>
  <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20(x64)-0078D4" alt="Platform: Windows">
  <img src="https://img.shields.io/badge/Language-Rust-DEA584" alt="Language: Rust">
  <img src="https://img.shields.io/badge/License-GPL--3.0-00E676" alt="License: GPL-3.0">
  <img src="https://img.shields.io/badge/Hardware-95%2B%20Razer%20Mice-00FF00" alt="Hardware Support">
</p>

---

## Overview

RazerLine is an independent, native Windows desktop application designed to configure, tune, and personalize Razer gaming mice directly via low-level USB HID reports — without requiring Razer Synapse running in the background.

Unlike proprietary software suites that install multiple background services, launch Chromium web renders, track user behavior, and consume hundreds of megabytes of RAM, RazerLine is built from the ground up in high-performance Rust:

* **Instant Startup:** Launches and connects in under 100 milliseconds.
* **Minimal Footprint:** Consumes ~25 MB of RAM and 0.0% background CPU while idle.
* **Offline and Private:** No user accounts, no telemetry, no tracking, and no internet connection required.
* **Zero Disruption In-Game:** Features a transparent, non-intrusive on-screen display (OSD) overlay that reports sensitivity adjustments seamlessly over full-screen games.

> [!NOTE]
> **Disclaimer:** RazerLine is an independent open-source project and is not affiliated with, endorsed by, or sponsored by Razer Inc. Hardware protocol specifications are adapted from public reverse-engineering contributions pioneered by the [OpenRazer](https://github.com/openrazer/openrazer) community.

---

## Comparison: RazerLine vs. Razer Synapse

| Feature | RazerLine | Razer Synapse 3 / 4 |
| :--- | :--- | :--- |
| **Memory Footprint (RAM)** | ~25 MB | 500 MB – 1.2 GB |
| **Background Processes** | 1 unified process | 5 – 12 separate services |
| **Startup Time** | Under 100 ms | 15 – 35 seconds |
| **Account / Login Required** | Never | Mandatory cloud login |
| **Telemetry & Data Collection** | None | Continuous analytics |
| **In-Game DPI OSD Overlay** | Direct Win32 Transparent HUD | Heavy notifications |
| **Physical DPI Button Override** | Enforces Custom Stages | Reverts to factory presets |
| **Binary Architecture** | Native x64 Machine Code | Electron / Chromium / .NET |

---

## Features

### Custom Multi-Stage DPI Sensitivity
* **Configurable Sensitivity Stages:** Define between 1 and 5 custom sensitivity stepping levels with granular DPI values (from 100 up to your sensor's physical ceiling, e.g. 26,000+ DPI).
* **Physical Mouse Button Interception:** When pressing the physical DPI button on your mouse, RazerLine's real-time hardware poller intercepts the mouse microcontroller's factory presets and immediately enforces your active custom profile stage.
* **Independent Axis Tuning:** Full support for asynchronous X-axis and Y-axis sensitivity configurations.
* **Stage Management:** Add, remove, rearrange, and sort stages in ascending order with a single click.

### In-Game Heads-Up Display (DPI OSD)
* **Zero-Latency Game Overlay:** Displays a semi-transparent dark glass HUD badge indicating the active sensitivity (e.g. `1400 DPI`) whenever DPI changes.
* **Click-Through and Non-Activating:** Rendered via a native Win32 layered transparent window (`WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT`). It will never steal keyboard/mouse focus or minimize full-screen gaming sessions.

### Low-Level Button Remapping Engine
* **Native Windows Mouse Hook:** Powered by a high-speed `WH_MOUSE_LL` hook engine running on a dedicated background thread.
* **Interactive Vector Blueprint:** Visual interactive mouse diagram with selectable button pins and real-time mapping state.
* **Configurable Inputs:** Remap Middle Click, Side Back (M4), Side Forward (M5), HyperScroll Tilt Left, Tilt Right, Sensitivity Clutch / Sniper Paddle, and Top buttons.
* **Supported Actions:**
  * **Windows Shortcuts:** Lock Workstation (`Win+L`), Task View (`Win+Tab`), Show Desktop (`Win+D`), Browser Back / Forward.
  * **Media Controls:** Volume Up, Volume Down, Mute, Play/Pause, Next Track, Previous Track.
  * **Sensitivity Controls:** Cycle DPI, DPI Up, DPI Down, Sniper Clutch (temporary DPI shift while holding).
  * **Profile Switching:** Instant profile cycle action mapped to any mouse button.
  * **Default / Disabled:** Native system passthrough or complete button suppression.

### Razer Chroma RGB Lighting Control
* **Multi-Zone Lighting:** Independent control over individual zones (Scroll Wheel, Logo, Underglow Strip).
* **Built-in Hardware Effects:** Static Color, Spectrum Cycling, Breathing, Wave, and Off.
* **Zone Synchronization:** Link all LED zones across your device to display a harmonious lighting aesthetic.
* **Live 60 FPS Preview Canvas:** Real-time animated simulation inside the application demonstrating how your chosen colors and effects animate on physical hardware.
* **Addressable Matrix / Underglow:** Dedicated support for multi-LED linear underglow strips (e.g. the 11-LED bottom underglow on the Razer Basilisk V3).

### Profile Management and Instant Persistence
* **Per-Device Profiles:** Create, duplicate, rename, and delete tailored profiles for individual games or productivity workflows (e.g. `CS2`, `Valorant`, `Office`, `Design`).
* **Auto-Persistence:** All settings, button maps, polling rates, and lighting profiles are automatically saved in human-readable JSON format to `%APPDATA%\RazerLine\config.json`.

### Background Daemon and System Tray Integration
* **System Tray Minimization:** Minimize to the Windows Notification Area on window close with zero visual distraction.
* **Windows Autostart:** One-click toggle in Settings to automatically launch RazerLine minimized on system boot via the Windows Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).
* **Single-Instance Guard:** Protected by a native Windows named Mutex to prevent duplicate processes from launching simultaneously.

---

## Supported Hardware (95+ Razer Mice)

RazerLine contains pre-configured hardware profiles for over 95 Razer gaming mice, cross-referenced against OpenRazer kernel drivers and hardware databases:

| Series | Supported Models |
| :--- | :--- |
| **Basilisk** | Basilisk V3, Basilisk V3 Pro, Basilisk V2, Basilisk Ultimate, Basilisk X HyperSpeed, Basilisk Essential |
| **DeathAdder** | DeathAdder V3, DeathAdder V3 Pro, DeathAdder V2, DeathAdder V2 Pro, DeathAdder V2 Mini, DeathAdder V2 X, DeathAdder Elite, DeathAdder Chroma, DeathAdder Essential |
| **Viper** | Viper V3 Pro, Viper V2 Pro, Viper Ultimate, Viper 8KHz, Viper Mini, Viper Mini Signature Edition, Viper HyperSpeed |
| **Cobra** | Cobra, Cobra Pro |
| **Naga** | Naga V2 Pro, Naga Pro, Naga Trinity, Naga Left-Handed Edition, Naga Chroma, Naga Hex V2 |
| **Mamba** | Mamba Elite, Mamba Wireless, Mamba Tournament Edition, Mamba HyperFlux |
| **Other Series** | Lancehead (TE & Wireless), Abyssus (V2, Essential, Elite), Atheris, Orochi V2, Jugan, Diamondback, Taipan, Krait |

*Notice:* If your specific Razer mouse is not listed, adding it requires only a small declarative `DeviceProfile` descriptor in `crates/devices-mice`.

---

## Quick Start & Building

### Prerequisites
* Windows 10 / 11 (64-bit)
* [Rust toolchain](https://rustup.rs/) (`stable-x86_64-pc-windows-msvc`)

### 1. Clone the Repository
```powershell
git clone https://github.com/amackdev/RazerLine.git
cd RazerLine
```

### 2. Compile the Release Workspace
```powershell
cargo build --release --workspace
```
Compiled binaries and runtime DLLs will be output to `target\release\`:
* `razerline.exe` (Desktop Application)
* `razer_hid_core.dll` (Engine Driver)
* `razer_hid_protocol.dll` (Protocol Engine)
* `razer_hid_devices_mice.dll` (Device Database)

### 3. Launch RazerLine
```powershell
.\target\release\razerline.exe
```
*(To launch directly to the system tray on startup: `.\target\release\razerline.exe --background`)*

---

## Architecture & Technical Design

RazerLine is built as a modular Cargo workspace designed for separation of concerns and crash isolation:

```
RazerLine/
├── crates/
│   ├── abi/            (razer-hid-abi)          Compile-time ABI shapes, command opcodes, and dispatch types
│   ├── protocol/       (razer-hid-protocol)     USB HID report framing, argument packing, and XOR checksum calculation
│   ├── core/           (razer-hid-core)         Hardware communication engine compiled to dynamic library (cdylib)
│   ├── devices-mice/   (razer-hid-devices-mice) 95+ declarative Razer device profiles compiled to dynamic library (cdylib)
│   ├── client/         (razer-hid-client)       Runtime dynamic DLL loader with built-in safe fallback
│   └── desktop/        (razer-hid-desktop)      Tauri v2 + Svelte 5 desktop UI, Win32 low-level mouse hooks, OSD overlay, and tray daemon
├── assets/             (Branding & Icons)       High-res vector logo, application icons, and graphics
└── Cargo.toml          (Workspace Manifest)     Root configuration and dependency specifications
```

### Dynamic Plugin Architecture
`razer-hid-core` and `razer-hid-devices-mice` can be compiled as runtime-loaded dynamic link libraries (`.dll`). The main application executable (`razerline.exe`) loads them dynamically upon startup, with built-in fallbacks if running standalone. This architecture ensures:
1. **Extensibility:** The device catalog DLL can be updated independently without recompiling the main application.
2. **Binary Isolation:** Memory boundaries are cleanly partitioned through a strict C-compatible FFI boundary (`razer_core_call`).

---

## Contributing

To add support or verify capabilities for your Razer device:

1. Copy `crates/devices-mice/src/_template.rs` to a new file named after your model (e.g. `deathadder_v3_wired.rs`).
2. Identify your device's USB Product ID (PID) from Windows Device Manager (Hardware IDs: `VID_1532&PID_XXXX`).
3. Fill in the `DeviceProfile` descriptor (DPI range, supported polling rates, LED zones, matrix dimensions).
4. Register the new profile in `crates/devices-mice/src/lib.rs`.
5. Run the test suite:
   ```powershell
   cargo test --workspace
   ```
6. Submit a Pull Request describing your hardware test results.

---

## License

Distributed under the **GNU General Public License v3.0 or later** (`GPL-3.0-or-later`). See the [`LICENSE`](LICENSE) file for details.

This project honors and builds upon the reverse-engineering documentation established by the [OpenRazer](https://github.com/openrazer/openrazer) project.

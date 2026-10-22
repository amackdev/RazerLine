//! RazerLine desktop app — Tauri Rust backend. The frontend (Svelte) talks
//! to this through the `#[tauri::command]`s in `commands.rs`; the
//! background device scan/connect/battery/DPI-poll loop in `worker.rs`
//! pushes updates the other direction via Tauri events.

mod autostart;
mod buttons;
mod commands;
mod config;
mod profiles;
mod state;
mod worker;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = AppState::new();

            // Sync autostart from saved config
            let autostart_enabled = state.config.lock().unwrap().start_with_windows;
            if autostart_enabled {
                let _ = app.autolaunch().enable();
            }

            app.manage(state.clone());
            worker::spawn(app.handle().clone(), state);

            let show = MenuItem::with_id(app, "show", "Open RazerLine", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide to Tray", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&show, &hide, &separator, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "hide" => hide_main_window(app),
                    "quit" => std::process::exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Close-to-tray: gated by config (Phase 4). Default true matches
            // the old app's behavior; Phase 1 used unconditional true as a
            // placeholder until the config system landed.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let close_to_tray = window
                    .state::<AppState>()
                    .config
                    .lock()
                    .unwrap()
                    .close_to_tray;
                if close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_connected,
            commands::connect_device,
            commands::active_device,
            commands::get_dpi,
            commands::set_dpi,
            commands::get_poll_rate,
            commands::set_poll_rate,
            commands::get_battery_level,
            commands::get_charging_status,
            commands::get_idle_time,
            commands::set_idle_time,
            commands::get_low_battery_threshold,
            commands::set_low_battery_threshold,
            commands::get_firmware_version,
            commands::get_serial,
            commands::set_led_off,
            commands::set_led_static,
            commands::set_led_breathing,
            commands::set_led_spectrum,
            commands::set_led_wave,
            commands::set_led_brightness,
            commands::set_led_off_all,
            commands::set_led_static_all,
            commands::set_led_breathing_all,
            commands::set_led_spectrum_all,
            commands::set_led_wave_all,
            commands::set_led_brightness_all,
            commands::set_matrix_row,
            commands::commit_matrix_frame,
            commands::fill_matrix,
            commands::set_matrix_spectrum,
            commands::set_matrix_wave,
            // Config / Profile commands (Phase 4)
            commands::get_config,
            commands::set_start_with_windows,
            commands::set_close_to_tray,
            commands::save_config,
            commands::list_profiles,
            commands::get_active_profile,
            commands::set_active_profile,
            commands::create_profile,
            commands::duplicate_profile,
            commands::delete_profile,
            commands::rename_profile,
            commands::update_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

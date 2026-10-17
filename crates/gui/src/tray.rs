//! System tray integration for background execution.

use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub struct TrayManager {
    _tray: TrayIcon,
}

impl TrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load icon from bundled PNG
        let image_bytes = include_bytes!("../assets/icon.png");
        let img = image::load_from_memory(image_bytes)?.into_rgba8();
        let (width, height) = img.dimensions();
        let icon = Icon::from_rgba(img.into_raw(), width, height)?;

        let tray_menu = Menu::new();

        let show_item = MenuItem::new("Open RazerLine", true, None);
        let hide_item = MenuItem::new("Hide to Tray", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let show_item_id = show_item.id().clone();
        let hide_item_id = hide_item.id().clone();
        let quit_item_id = quit_item.id().clone();

        tray_menu.append(&show_item)?;
        tray_menu.append(&hide_item)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&quit_item)?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_menu_on_left_click(false)
            .with_tooltip("RazerLine — Peripheral Control")
            .with_icon(icon)
            .build()?;

        // Start background event listener thread so clicks & menus
        // are processed even when the window is hidden/sleeping.
        std::thread::spawn(move || {
            loop {
                // Check TrayIcon clicks
                while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                    match event {
                        TrayIconEvent::Click {
                            button: tray_icon::MouseButton::Left,
                            button_state: tray_icon::MouseButtonState::Up,
                            ..
                        }
                        | TrayIconEvent::DoubleClick {
                            button: tray_icon::MouseButton::Left,
                            ..
                        } => {
                            crate::win_util::toggle_window();
                        }
                        _ => {}
                    }
                }

                // Check context menu actions
                while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
                    if event.id == show_item_id {
                        crate::win_util::show_window();
                    } else if event.id == hide_item_id {
                        crate::win_util::hide_window();
                    } else if event.id == quit_item_id {
                        // Exit immediately and cleanly
                        std::process::exit(0);
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        Ok(Self {
            _tray: tray,
        })
    }
}

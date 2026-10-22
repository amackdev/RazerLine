//! Mouse button definitions and action remapping types.
//! Ported verbatim from `crates/gui/src/buttons.rs` — already
//! `Serialize`/`Deserialize`, no egui dependency.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButtonId {
    LeftClick,
    RightClick,
    MiddleClick,
    TiltLeft,
    TiltRight,
    ScrollUp,
    ScrollDown,
    SideBack,
    SideForward,
    MultiFunctionClutch,
    DpiSwitch,
    ScrollMode,
    ProfileSwitch,
}

impl MouseButtonId {
    #[allow(dead_code)]
    pub const ALL: [MouseButtonId; 13] = [
        MouseButtonId::LeftClick,
        MouseButtonId::RightClick,
        MouseButtonId::MiddleClick,
        MouseButtonId::TiltLeft,
        MouseButtonId::TiltRight,
        MouseButtonId::ScrollUp,
        MouseButtonId::ScrollDown,
        MouseButtonId::SideBack,
        MouseButtonId::SideForward,
        MouseButtonId::MultiFunctionClutch,
        MouseButtonId::DpiSwitch,
        MouseButtonId::ScrollMode,
        MouseButtonId::ProfileSwitch,
    ];

    pub const CONFIGURABLE: [MouseButtonId; 11] = [
        MouseButtonId::MiddleClick,
        MouseButtonId::TiltLeft,
        MouseButtonId::TiltRight,
        MouseButtonId::ScrollUp,
        MouseButtonId::ScrollDown,
        MouseButtonId::SideBack,
        MouseButtonId::SideForward,
        MouseButtonId::MultiFunctionClutch,
        MouseButtonId::DpiSwitch,
        MouseButtonId::ScrollMode,
        MouseButtonId::ProfileSwitch,
    ];

    pub fn is_configurable(&self) -> bool {
        !matches!(self, MouseButtonId::LeftClick | MouseButtonId::RightClick)
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            MouseButtonId::LeftClick => "Left Click",
            MouseButtonId::RightClick => "Right Click",
            MouseButtonId::MiddleClick => "Scroll Click",
            MouseButtonId::TiltLeft => "Repeat Scroll Left",
            MouseButtonId::TiltRight => "Repeat Scroll Right",
            MouseButtonId::ScrollUp => "Scroll Up",
            MouseButtonId::ScrollDown => "Scroll Down",
            MouseButtonId::SideBack => "Mouse Button 4",
            MouseButtonId::SideForward => "Mouse Button 5",
            MouseButtonId::MultiFunctionClutch => "Sensitivity Clutch",
            MouseButtonId::DpiSwitch => "Cycle Up Sensitivity Stages",
            MouseButtonId::ScrollMode => "Scroll Mode Toggle",
            MouseButtonId::ProfileSwitch => "Cycle Up Profile",
        }
    }

    #[allow(dead_code)]
    pub fn short_name(&self) -> &'static str {
        match self {
            MouseButtonId::LeftClick => "Left Click",
            MouseButtonId::RightClick => "Right Click",
            MouseButtonId::MiddleClick => "Scroll Click",
            MouseButtonId::TiltLeft => "Scroll Left",
            MouseButtonId::TiltRight => "Scroll Right",
            MouseButtonId::ScrollUp => "Scroll Up",
            MouseButtonId::ScrollDown => "Scroll Down",
            MouseButtonId::SideBack => "Button 4",
            MouseButtonId::SideForward => "Button 5",
            MouseButtonId::MultiFunctionClutch => "Clutch",
            MouseButtonId::DpiSwitch => "Cycle Sensitivity",
            MouseButtonId::ScrollMode => "Scroll Mode",
            MouseButtonId::ProfileSwitch => "Cycle Profile",
        }
    }

    pub fn default_action(&self) -> ButtonAction {
        match self {
            MouseButtonId::LeftClick => ButtonAction::Default,
            MouseButtonId::RightClick => ButtonAction::Default,
            MouseButtonId::MiddleClick => ButtonAction::Default,
            MouseButtonId::TiltLeft => ButtonAction::Mouse(MouseFunction::ScrollLeft),
            MouseButtonId::TiltRight => ButtonAction::Mouse(MouseFunction::ScrollRight),
            MouseButtonId::ScrollUp => ButtonAction::Default,
            MouseButtonId::ScrollDown => ButtonAction::Default,
            MouseButtonId::SideBack => ButtonAction::Default,
            MouseButtonId::SideForward => ButtonAction::Default,
            MouseButtonId::MultiFunctionClutch => {
                ButtonAction::Sensitivity(SensitivityAction::SniperClutch { clutch_dpi: 400 })
            }
            MouseButtonId::DpiSwitch => ButtonAction::Sensitivity(SensitivityAction::CycleDpi),
            MouseButtonId::ScrollMode => ButtonAction::Default,
            MouseButtonId::ProfileSwitch => ButtonAction::ProfileCycle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonAction {
    Default,
    Mouse(MouseFunction),
    Keyboard { key_name: String },
    Sensitivity(SensitivityAction),
    Multimedia(MediaAction),
    WindowsShortcut(WinShortcut),
    ProfileCycle,
    Disabled,
}

impl ButtonAction {
    #[allow(dead_code)]
    pub fn label(&self) -> String {
        match self {
            ButtonAction::Default => "Default Function".to_string(),
            ButtonAction::Mouse(mf) => format!("Mouse: {}", mf.label()),
            ButtonAction::Keyboard { key_name } => format!("Key: {key_name}"),
            ButtonAction::Sensitivity(sa) => format!("Sensitivity: {}", sa.label()),
            ButtonAction::Multimedia(ma) => format!("Media: {}", ma.label()),
            ButtonAction::WindowsShortcut(ws) => format!("Windows: {}", ws.label()),
            ButtonAction::ProfileCycle => "Profile: Cycle Next Profile".to_string(),
            ButtonAction::Disabled => "Disabled".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseFunction {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollLeft,
    ScrollRight,
    Back,
    Forward,
    DoubleClick,
}

impl MouseFunction {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            MouseFunction::LeftClick => "Left Click",
            MouseFunction::RightClick => "Right Click",
            MouseFunction::MiddleClick => "Middle Click",
            MouseFunction::ScrollLeft => "Scroll Left",
            MouseFunction::ScrollRight => "Scroll Right",
            MouseFunction::Back => "Back (M4)",
            MouseFunction::Forward => "Forward (M5)",
            MouseFunction::DoubleClick => "Double Click",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensitivityAction {
    CycleDpi,
    DpiUp,
    DpiDown,
    SniperClutch { clutch_dpi: u16 },
}

impl SensitivityAction {
    #[allow(dead_code)]
    pub fn label(&self) -> String {
        match self {
            SensitivityAction::CycleDpi => "DPI Cycle".to_string(),
            SensitivityAction::DpiUp => "DPI Stage Up".to_string(),
            SensitivityAction::DpiDown => "DPI Stage Down".to_string(),
            SensitivityAction::SniperClutch { clutch_dpi } => {
                format!("Sniper Clutch ({clutch_dpi} DPI)")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaAction {
    PlayPause,
    VolumeUp,
    VolumeDown,
    Mute,
    NextTrack,
    PrevTrack,
}

impl MediaAction {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            MediaAction::PlayPause => "Play / Pause",
            MediaAction::VolumeUp => "Volume Up",
            MediaAction::VolumeDown => "Volume Down",
            MediaAction::Mute => "Mute Audio",
            MediaAction::NextTrack => "Next Track",
            MediaAction::PrevTrack => "Previous Track",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WinShortcut {
    LockPc,
    TaskView,
    ShowDesktop,
    OpenExplorer,
}

impl WinShortcut {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            WinShortcut::LockPc => "Lock PC (Win+L)",
            WinShortcut::TaskView => "Task View (Win+Tab)",
            WinShortcut::ShowDesktop => "Show Desktop (Win+D)",
            WinShortcut::OpenExplorer => "File Explorer (Win+E)",
        }
    }
}

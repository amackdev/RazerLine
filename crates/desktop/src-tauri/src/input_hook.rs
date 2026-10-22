//! Native Windows Low-Level Mouse Hook engine for real-time button remapping.
//!
//! Intercepts physical mouse clicks (M4, M5, Middle Click, Tilt Wheel, etc.)
//! and executes assigned actions: Multimedia controls, Windows shortcuts,
//! keystrokes, and DPI/profile switching.
//!
//! Ported from `crates/gui/src/input_hook.rs` — the hook logic is unchanged;
//! only the command dispatch destination changes (now emits Tauri events
//! to the worker thread instead of calling GUI methods directly).
//!
//! Tilt-wheel-momentary fix (SniperClutch auto-release on tilt-wheel release)
//! is preserved.
//!
//! On non-Windows platforms, this module provides no-op stubs.

#[cfg(windows)]
#[allow(unused_imports, dead_code)]
mod windows_impl {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::{OnceLock, RwLock};
    use std::thread;
    use std::time::Duration;

    use crate::buttons::{
        ButtonAction, MediaAction, MouseButtonId, MouseFunction, SensitivityAction, WinShortcut,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HookCommand {
        CycleDpi,
        DpiUp,
        DpiDown,
        SetClutchDpi(u16),
        RestoreClutchDpi,
        CycleProfile,
    }

    static CURRENT_MAPPINGS: RwLock<Option<HashMap<MouseButtonId, ButtonAction>>> =
        RwLock::new(None);
    static COMMAND_SENDER: OnceLock<Sender<HookCommand>> = OnceLock::new();
    static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

    #[allow(non_snake_case, non_camel_case_types, clippy::upper_case_acronyms)]
    mod win32 {
        use std::ffi::c_void;

        pub type HHOOK = *mut c_void;
        pub type HINSTANCE = *mut c_void;
        pub type HWND = *mut c_void;
        pub type LPARAM = isize;
        pub type WPARAM = usize;
        pub type LRESULT = isize;
        pub type BOOL = i32;

        pub type HOOKPROC =
            unsafe extern "system" fn(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;

        pub const WH_MOUSE_LL: i32 = 14;

        pub const WM_LBUTTONDOWN: usize = 0x0201;
        pub const WM_LBUTTONUP: usize = 0x0202;
        pub const WM_RBUTTONDOWN: usize = 0x0204;
        pub const WM_RBUTTONUP: usize = 0x0205;
        pub const WM_MBUTTONDOWN: usize = 0x0207;
        pub const WM_MBUTTONUP: usize = 0x0208;
        pub const WM_XBUTTONDOWN: usize = 0x020B;
        pub const WM_XBUTTONUP: usize = 0x020C;
        pub const WM_MOUSEHWHEEL: usize = 0x020E;

        pub const XBUTTON1: u16 = 0x0001; // Back (Button 4)
        pub const XBUTTON2: u16 = 0x0002; // Forward (Button 5)

        pub const KEYEVENTF_KEYUP: u32 = 0x0002;
        pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;

        pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
        pub const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
        pub const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
        pub const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
        pub const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
        pub const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
        pub const MOUSEEVENTF_XDOWN: u32 = 0x0080;
        pub const MOUSEEVENTF_XUP: u32 = 0x0100;
        pub const MOUSEEVENTF_HWHEEL: u32 = 0x1000;

        pub const VK_LWIN: u8 = 0x5B;
        pub const VK_TAB: u8 = 0x09;
        pub const VK_VOLUME_MUTE: u8 = 0xAD;
        pub const VK_VOLUME_DOWN: u8 = 0xAE;
        pub const VK_VOLUME_UP: u8 = 0xAF;
        pub const VK_MEDIA_NEXT_TRACK: u8 = 0xB0;
        pub const VK_MEDIA_PREV_TRACK: u8 = 0xB1;
        pub const VK_MEDIA_PLAY_PAUSE: u8 = 0xB3;

        pub const LLMHF_INJECTED: u32 = 0x00000001;
        pub const WM_QUIT: u32 = 0x0012;

        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct POINT {
            pub x: i32,
            pub y: i32,
        }

        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct MSLLHOOKSTRUCT {
            pub pt: POINT,
            pub mouse_data: u32,
            pub flags: u32,
            pub time: u32,
            pub extra_info: usize,
        }

        #[repr(C)]
        pub struct MSG {
            pub hwnd: HWND,
            pub message: u32,
            pub wparam: WPARAM,
            pub lparam: LPARAM,
            pub time: u32,
            pub pt: POINT,
        }

        #[link(name = "user32")]
        extern "system" {
            pub fn SetWindowsHookExW(
                idHook: i32,
                lpfn: HOOKPROC,
                hmod: HINSTANCE,
                dwThreadId: u32,
            ) -> HHOOK;
            pub fn UnhookWindowsHookEx(hhk: HHOOK) -> BOOL;
            pub fn CallNextHookEx(
                hhk: HHOOK,
                nCode: i32,
                wParam: WPARAM,
                lParam: LPARAM,
            ) -> LRESULT;
            pub fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
            pub fn mouse_event(dwFlags: u32, dx: u32, dy: u32, dwData: u32, dwExtraInfo: usize);
            pub fn LockWorkStation() -> BOOL;
            pub fn GetMessageW(
                lpMsg: *mut MSG,
                hWnd: HWND,
                wMsgFilterMin: u32,
                wMsgFilterMax: u32,
            ) -> BOOL;
            pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
            pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
            pub fn PostThreadMessageW(
                idThread: u32,
                Msg: u32,
                wParam: WPARAM,
                lParam: LPARAM,
            ) -> BOOL;
            pub fn GetCurrentThreadId() -> u32;
        }
    }

    pub fn update_active_mappings(mappings: HashMap<MouseButtonId, ButtonAction>) {
        if let Ok(mut lock) = CURRENT_MAPPINGS.write() {
            *lock = Some(mappings);
        }
    }

    pub struct InputHookManager {
        command_rx: Option<Receiver<HookCommand>>,
    }

    impl InputHookManager {
        pub fn new() -> Self {
            let (tx, rx) = mpsc::channel();
            let _ = COMMAND_SENDER.set(tx);

            thread::Builder::new()
                .name("razerline-mouse-hook".into())
                .spawn(move || {
                    let tid = unsafe { win32::GetCurrentThreadId() };
                    HOOK_THREAD_ID.store(tid, Ordering::SeqCst);

                    let hook = unsafe {
                        win32::SetWindowsHookExW(
                            win32::WH_MOUSE_LL,
                            low_level_mouse_proc,
                            std::ptr::null_mut(),
                            0,
                        )
                    };

                    if hook.is_null() {
                        eprintln!("Failed to register WH_MOUSE_LL hook");
                        return;
                    }

                    let mut msg = unsafe { std::mem::zeroed() };
                    while unsafe { win32::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) } > 0 {
                        unsafe {
                            win32::TranslateMessage(&msg);
                            win32::DispatchMessageW(&msg);
                        }
                    }

                    unsafe {
                        win32::UnhookWindowsHookEx(hook);
                    }
                })
                .expect("failed to spawn mouse hook thread");

            Self {
                command_rx: Some(rx),
            }
        }

        pub fn take_command_rx(&mut self) -> Option<Receiver<HookCommand>> {
            self.command_rx.take()
        }

        pub fn update_mappings(&self, mappings: HashMap<MouseButtonId, ButtonAction>) {
            update_active_mappings(mappings);
        }

        #[allow(dead_code)]
        pub fn try_recv_command(&self) -> Option<HookCommand> {
            self.command_rx.as_ref().and_then(|rx| rx.try_recv().ok())
        }
    }

    impl Drop for InputHookManager {
        fn drop(&mut self) {
            let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
            if tid != 0 {
                unsafe {
                    win32::PostThreadMessageW(tid, win32::WM_QUIT, 0, 0);
                }
            }
        }
    }

    unsafe extern "system" fn low_level_mouse_proc(
        n_code: i32,
        w_param: win32::WPARAM,
        l_param: win32::LPARAM,
    ) -> win32::LRESULT {
        if n_code < 0 {
            return win32::CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        let hook_struct = *(l_param as *const win32::MSLLHOOKSTRUCT);

        // Skip synthetic events injected by ourselves or other tools to avoid feedback loops
        if (hook_struct.flags & win32::LLMHF_INJECTED) != 0 {
            return win32::CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }

        let btn_and_is_down = match w_param {
            win32::WM_XBUTTONDOWN => {
                let xbtn = (hook_struct.mouse_data >> 16) as u16;
                let id = if xbtn == win32::XBUTTON1 {
                    MouseButtonId::SideBack // Button 4
                } else {
                    MouseButtonId::SideForward // Button 5
                };
                Some((id, true))
            }
            win32::WM_XBUTTONUP => {
                let xbtn = (hook_struct.mouse_data >> 16) as u16;
                let id = if xbtn == win32::XBUTTON1 {
                    MouseButtonId::SideBack
                } else {
                    MouseButtonId::SideForward
                };
                Some((id, false))
            }
            win32::WM_MBUTTONDOWN => Some((MouseButtonId::MiddleClick, true)),
            win32::WM_MBUTTONUP => Some((MouseButtonId::MiddleClick, false)),
            win32::WM_MOUSEHWHEEL => {
                let delta = (hook_struct.mouse_data >> 16) as i16;
                let id = if delta < 0 {
                    MouseButtonId::TiltLeft
                } else {
                    MouseButtonId::TiltRight
                };
                Some((id, true))
            }
            // Left and Right clicks are primary system controls and are never intercepted or remapped
            win32::WM_LBUTTONDOWN
            | win32::WM_LBUTTONUP
            | win32::WM_RBUTTONDOWN
            | win32::WM_RBUTTONUP => None,
            _ => None,
        };

        // Tilt-wheel ticks only ever synthesize a "down" event (there's no
        // corresponding WM_MOUSEHWHEEL "up") — treat them as momentary so
        // SniperClutch can't latch a clutch DPI it will never get a release
        // to restore from.
        let is_momentary = w_param == win32::WM_MOUSEHWHEEL;

        if let Some((btn_id, is_down)) = btn_and_is_down {
            if let Ok(lock) = CURRENT_MAPPINGS.read() {
                if let Some(map) = lock.as_ref() {
                    if let Some(action) = map.get(&btn_id) {
                        match action {
                            ButtonAction::Default => {
                                // Let Windows handle normally
                                return win32::CallNextHookEx(
                                    std::ptr::null_mut(),
                                    n_code,
                                    w_param,
                                    l_param,
                                );
                            }
                            ButtonAction::Disabled => {
                                // Eat the event completely
                                return 1;
                            }
                            ButtonAction::Sensitivity(sa) => {
                                if is_down {
                                    match sa {
                                        SensitivityAction::CycleDpi => {
                                            send_hook_command(HookCommand::CycleDpi);
                                        }
                                        SensitivityAction::DpiUp => {
                                            send_hook_command(HookCommand::DpiUp);
                                        }
                                        SensitivityAction::DpiDown => {
                                            send_hook_command(HookCommand::DpiDown);
                                        }
                                        SensitivityAction::SniperClutch { clutch_dpi } => {
                                            send_hook_command(HookCommand::SetClutchDpi(
                                                *clutch_dpi,
                                            ));
                                        }
                                    }
                                }
                                if (!is_down || is_momentary)
                                    && matches!(sa, SensitivityAction::SniperClutch { .. })
                                {
                                    send_hook_command(HookCommand::RestoreClutchDpi);
                                }
                                return 1;
                            }
                            ButtonAction::ProfileCycle => {
                                if is_down {
                                    send_hook_command(HookCommand::CycleProfile);
                                }
                                return 1;
                            }
                            _ => {
                                if is_down {
                                    execute_action(action);
                                }
                                return 1;
                            }
                        }
                    }
                }
            }
        }

        win32::CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
    }

    fn send_hook_command(cmd: HookCommand) {
        if let Some(tx) = COMMAND_SENDER.get() {
            let _ = tx.send(cmd);
        }
    }

    fn execute_action(action: &ButtonAction) {
        match action {
            ButtonAction::Multimedia(media) => {
                let vk = match media {
                    MediaAction::VolumeUp => win32::VK_VOLUME_UP,
                    MediaAction::VolumeDown => win32::VK_VOLUME_DOWN,
                    MediaAction::Mute => win32::VK_VOLUME_MUTE,
                    MediaAction::PlayPause => win32::VK_MEDIA_PLAY_PAUSE,
                    MediaAction::NextTrack => win32::VK_MEDIA_NEXT_TRACK,
                    MediaAction::PrevTrack => win32::VK_MEDIA_PREV_TRACK,
                };
                unsafe {
                    win32::keybd_event(vk, 0, win32::KEYEVENTF_EXTENDEDKEY, 0);
                    win32::keybd_event(
                        vk,
                        0,
                        win32::KEYEVENTF_EXTENDEDKEY | win32::KEYEVENTF_KEYUP,
                        0,
                    );
                }
            }
            ButtonAction::WindowsShortcut(ws) => match ws {
                WinShortcut::LockPc => unsafe {
                    win32::LockWorkStation();
                },
                WinShortcut::TaskView => unsafe {
                    // Win + Tab
                    win32::keybd_event(win32::VK_LWIN, 0, 0, 0);
                    win32::keybd_event(win32::VK_TAB, 0, 0, 0);
                    win32::keybd_event(win32::VK_TAB, 0, win32::KEYEVENTF_KEYUP, 0);
                    win32::keybd_event(win32::VK_LWIN, 0, win32::KEYEVENTF_KEYUP, 0);
                },
                WinShortcut::ShowDesktop => unsafe {
                    // Win + D
                    win32::keybd_event(win32::VK_LWIN, 0, 0, 0);
                    win32::keybd_event(0x44, 0, 0, 0); // 'D'
                    win32::keybd_event(0x44, 0, win32::KEYEVENTF_KEYUP, 0);
                    win32::keybd_event(win32::VK_LWIN, 0, win32::KEYEVENTF_KEYUP, 0);
                },
                WinShortcut::OpenExplorer => unsafe {
                    // Win + E
                    win32::keybd_event(win32::VK_LWIN, 0, 0, 0);
                    win32::keybd_event(0x45, 0, 0, 0); // 'E'
                    win32::keybd_event(0x45, 0, win32::KEYEVENTF_KEYUP, 0);
                    win32::keybd_event(win32::VK_LWIN, 0, win32::KEYEVENTF_KEYUP, 0);
                },
            },
            ButtonAction::Mouse(mf) => match mf {
                MouseFunction::LeftClick => unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                    win32::mouse_event(win32::MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                },
                MouseFunction::RightClick => unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
                    win32::mouse_event(win32::MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
                },
                MouseFunction::MiddleClick => unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0, 0);
                    win32::mouse_event(win32::MOUSEEVENTF_MIDDLEUP, 0, 0, 0, 0);
                },
                MouseFunction::DoubleClick => unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                    win32::mouse_event(win32::MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                    thread::sleep(Duration::from_millis(30));
                    win32::mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                    win32::mouse_event(win32::MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                },
                MouseFunction::Back => unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_XDOWN, 0, 0, win32::XBUTTON1 as u32, 0);
                    win32::mouse_event(win32::MOUSEEVENTF_XUP, 0, 0, win32::XBUTTON1 as u32, 0);
                },
                MouseFunction::Forward => unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_XDOWN, 0, 0, win32::XBUTTON2 as u32, 0);
                    win32::mouse_event(win32::MOUSEEVENTF_XUP, 0, 0, win32::XBUTTON2 as u32, 0);
                },
                MouseFunction::ScrollLeft => unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_HWHEEL, 0, 0, (-120i32) as u32, 0);
                },
                MouseFunction::ScrollRight => unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_HWHEEL, 0, 0, 120u32, 0);
                },
            },
            ButtonAction::Keyboard { key_name } => {
                // Basic single character or letter typing
                if let Some(ch) = key_name.chars().next() {
                    let vk = match ch.to_ascii_uppercase() {
                        'A'..='Z' => ch.to_ascii_uppercase() as u8,
                        '0'..='9' => ch as u8,
                        _ => 0,
                    };
                    if vk != 0 {
                        unsafe {
                            win32::keybd_event(vk, 0, 0, 0);
                            win32::keybd_event(vk, 0, win32::KEYEVENTF_KEYUP, 0);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Re-export for use in worker.rs
    pub use self::HookCommand;
    pub use self::InputHookManager;
}

// Public API available on all platforms
#[cfg(windows)]
pub use windows_impl::{HookCommand, InputHookManager};

#[cfg(not(windows))]
#[allow(dead_code, unused_imports)]
/// No-op stub for non-Windows platforms.
pub mod windows_impl {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HookCommand {
        CycleDpi,
        DpiUp,
        DpiDown,
        SetClutchDpi(u16),
        RestoreClutchDpi,
        CycleProfile,
    }

    pub struct InputHookManager;

    impl InputHookManager {
        pub fn new() -> Self {
            Self
        }

        pub fn take_command_rx(&mut self) -> Option<std::sync::mpsc::Receiver<HookCommand>> {
            None
        }

        pub fn update_mappings(
            &self,
            _mappings: std::collections::HashMap<
                crate::buttons::MouseButtonId,
                crate::buttons::ButtonAction,
            >,
        ) {
        }

        #[allow(dead_code)]
        pub fn try_recv_command(&self) -> Option<HookCommand> {
            None
        }
    }
}

#[cfg(not(windows))]
#[allow(unused_imports)]
pub use windows_impl::{HookCommand, InputHookManager};

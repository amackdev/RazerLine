//! Windows-specific single instance guard and window management utilities.

#[cfg(windows)]
extern "system" {
    fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> *mut std::ffi::c_void;
    fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
    fn SetForegroundWindow(hWnd: *mut std::ffi::c_void) -> i32;
    fn IsWindowVisible(hWnd: *mut std::ffi::c_void) -> i32;
    fn CreateMutexW(lpMutexAttributes: *const std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> *mut std::ffi::c_void;
    fn GetLastError() -> u32;
}

const SW_HIDE: i32 = 0;
const SW_SHOW: i32 = 5;
const SW_RESTORE: i32 = 9;
const ERROR_ALREADY_EXISTS: u32 = 183;

static mut MUTEX_HANDLE: *mut std::ffi::c_void = std::ptr::null_mut();

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Checks if this is the only running instance of RazerLine.
/// If an instance is already running, it brings that instance to the front and returns false.
pub fn acquire_single_instance() -> bool {
    #[cfg(windows)]
    {
        let name = to_wide("Local\\RazerLine_App_Single_Instance_Mutex");
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        let err = unsafe { GetLastError() };

        if !handle.is_null() && err == ERROR_ALREADY_EXISTS {
            // Already running: activate existing window
            show_window();
            return false;
        }

        unsafe {
            MUTEX_HANDLE = handle;
        }
        true
    }

    #[cfg(not(windows))]
    {
        true
    }
}

/// Shows and brings the RazerLine window to the foreground.
pub fn show_window() {
    #[cfg(windows)]
    {
        let title = to_wide("RazerLine");
        let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if !hwnd.is_null() {
            unsafe {
                ShowWindow(hwnd, SW_RESTORE);
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
            }
        }
    }
}

/// Hides the RazerLine window to the tray.
pub fn hide_window() {
    #[cfg(windows)]
    {
        let title = to_wide("RazerLine");
        let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if !hwnd.is_null() {
            unsafe {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}

/// Toggles between showing and hiding the RazerLine window.
pub fn toggle_window() {
    #[cfg(windows)]
    {
        let title = to_wide("RazerLine");
        let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
        if !hwnd.is_null() {
            let visible = unsafe { IsWindowVisible(hwnd) };
            if visible != 0 {
                unsafe { ShowWindow(hwnd, SW_HIDE); }
            } else {
                unsafe {
                    ShowWindow(hwnd, SW_RESTORE);
                    ShowWindow(hwnd, SW_SHOW);
                    SetForegroundWindow(hwnd);
                }
            }
        }
    }
}

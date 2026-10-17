//! On-Screen Display (OSD) notification overlay for DPI changes.
//!
//! Provides a non-intrusive, topmost, click-through HUD overlay that pops up
//! over games and full-screen applications when DPI is changed, without stealing
//! focus or disrupting gameplay.

use std::sync::atomic::{AtomicIsize, AtomicU16, Ordering};

static OSD_HWND: AtomicIsize = AtomicIsize::new(0);
static CURRENT_DPI: AtomicU16 = AtomicU16::new(1600);

/// Initializes the background OSD overlay thread.
pub fn init() {
    #[cfg(windows)]
    {
        let _ = std::thread::Builder::new()
            .name("razerline-dpi-osd".into())
            .spawn(|| unsafe {
                run_osd_thread();
            });
    }
}

/// Triggers the OSD notification overlay displaying "DPI changed to <dpi>".
///
/// Thread-safe, non-blocking, and callable from any thread.
pub fn show_dpi(dpi: u16) {
    CURRENT_DPI.store(dpi, Ordering::SeqCst);
    #[cfg(windows)]
    {
        let hwnd = OSD_HWND.load(Ordering::SeqCst);
        if hwnd != 0 {
            unsafe {
                win32::PostMessageW(hwnd as win32::HWND, win32::WM_SHOW_DPI, dpi as win32::WPARAM, 0);
            }
        }
    }
}

#[cfg(windows)]
unsafe fn run_osd_thread() {
    let class_name = win32::to_wide("RazerLine_Dpi_OSD");
    let title = win32::to_wide("RazerLine DPI OSD");

    let wc = win32::WNDCLASSEXW {
        cbSize: std::mem::size_of::<win32::WNDCLASSEXW>() as u32,
        style: win32::CS_HREDRAW | win32::CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: std::ptr::null_mut(),
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };

    win32::RegisterClassExW(&wc);

    let ex_style = win32::WS_EX_TOPMOST
        | win32::WS_EX_TOOLWINDOW
        | win32::WS_EX_LAYERED
        | win32::WS_EX_NOACTIVATE
        | win32::WS_EX_TRANSPARENT;
    let style = win32::WS_POPUP;

    let win_w = 320;
    let win_h = 54;

    let hwnd = win32::CreateWindowExW(
        ex_style,
        class_name.as_ptr(),
        title.as_ptr(),
        style,
        0,
        0,
        win_w,
        win_h,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null(),
    );

    if !hwnd.is_null() {
        let rgn = win32::CreateRoundRectRgn(0, 0, win_w + 1, win_h + 1, 14, 14);
        if !rgn.is_null() {
            win32::SetWindowRgn(hwnd, rgn, 1);
        }

        // 94% opacity
        win32::SetLayeredWindowAttributes(hwnd, 0, 240, win32::LWA_ALPHA);
        OSD_HWND.store(hwnd as isize, Ordering::SeqCst);

        let mut msg = std::mem::zeroed::<win32::MSG>();
        while win32::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            win32::TranslateMessage(&msg);
            win32::DispatchMessageW(&msg);
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn wnd_proc(
    hwnd: win32::HWND,
    msg: u32,
    wparam: win32::WPARAM,
    lparam: win32::LPARAM,
) -> win32::LRESULT {
    match msg {
        win32::WM_SHOW_DPI => {
            let dpi = wparam as u16;
            CURRENT_DPI.store(dpi, Ordering::SeqCst);

            let screen_w = win32::GetSystemMetrics(win32::SM_CXSCREEN);
            let screen_h = win32::GetSystemMetrics(win32::SM_CYSCREEN);
            let win_w = 320;
            let win_h = 54;
            let x = (screen_w - win_w) / 2;
            let y = screen_h - win_h - 100; // 100px above bottom edge

            win32::SetWindowPos(
                hwnd,
                win32::HWND_TOPMOST,
                x,
                y,
                win_w,
                win_h,
                win32::SWP_NOACTIVATE | win32::SWP_SHOWWINDOW,
            );
            win32::ShowWindow(hwnd, win32::SW_SHOWNOACTIVATE);
            win32::InvalidateRect(hwnd, std::ptr::null(), 1);

            // Reset 1.8-second auto-hide timer
            win32::SetTimer(hwnd, 1, 1800, std::ptr::null());
            0
        }
        win32::WM_TIMER => {
            win32::KillTimer(hwnd, 1);
            win32::ShowWindow(hwnd, win32::SW_HIDE);
            0
        }
        win32::WM_PAINT => {
            paint_osd(hwnd);
            0
        }
        win32::WM_DESTROY => {
            win32::PostQuitMessage(0);
            0
        }
        _ => win32::DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(windows)]
unsafe fn paint_osd(hwnd: win32::HWND) {
    let mut ps = std::mem::zeroed::<win32::PAINTSTRUCT>();
    let hdc = win32::BeginPaint(hwnd, &mut ps);
    let mut rect = std::mem::zeroed::<win32::RECT>();
    win32::GetClientRect(hwnd, &mut rect);

    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    if width > 0 && height > 0 {
        let mem_dc = win32::CreateCompatibleDC(hdc);
        let mem_bmp = win32::CreateCompatibleBitmap(hdc, width, height);
        let old_bmp = win32::SelectObject(mem_dc, mem_bmp);

        // Stealth dark background with Razer green accent border
        // ColorRef is 0x00BBGGRR
        let bg_brush = win32::CreateSolidBrush(0x00181412); // RGB(18, 20, 24)
        let border_pen = win32::CreatePen(0, 2, 0x0066FF00); // RGB(0, 255, 102)

        let old_brush = win32::SelectObject(mem_dc, bg_brush);
        let old_pen = win32::SelectObject(mem_dc, border_pen);

        win32::RoundRect(mem_dc, 1, 1, width - 1, height - 1, 14, 14);

        win32::SelectObject(mem_dc, old_brush);
        win32::SelectObject(mem_dc, old_pen);
        win32::DeleteObject(bg_brush);
        win32::DeleteObject(border_pen);

        win32::SetBkMode(mem_dc, win32::TRANSPARENT);

        // Razer green glowing indicator dot
        let dot_brush = win32::CreateSolidBrush(0x0066FF00);
        let null_pen = win32::CreatePen(5, 0, 0); // PS_NULL = 5
        let old_b = win32::SelectObject(mem_dc, dot_brush);
        let old_p = win32::SelectObject(mem_dc, null_pen);
        win32::RoundRect(mem_dc, 20, (height - 10) / 2, 30, (height + 10) / 2, 8, 8);
        win32::SelectObject(mem_dc, old_b);
        win32::SelectObject(mem_dc, old_p);
        win32::DeleteObject(dot_brush);
        win32::DeleteObject(null_pen);

        // Display string: "DPI changed to <dpi>"
        let dpi = CURRENT_DPI.load(Ordering::SeqCst);
        let text = format!("DPI changed to {dpi}");
        let text_wide = win32::to_wide(&text);

        let font_name = win32::to_wide("Segoe UI");
        let font = win32::CreateFontW(
            22,
            0,
            0,
            0,
            win32::FW_BOLD,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            font_name.as_ptr(),
        );
        let old_font = win32::SelectObject(mem_dc, font);

        win32::SetTextColor(mem_dc, 0x00FFFFFF); // Pure white

        let mut text_rect = win32::RECT {
            left: 42,
            top: 0,
            right: width - 16,
            bottom: height,
        };
        win32::DrawTextW(
            mem_dc,
            text_wide.as_ptr(),
            text_wide.len() as i32 - 1, // exclude null terminator in count
            &mut text_rect,
            win32::DT_LEFT | win32::DT_VCENTER | win32::DT_SINGLELINE,
        );

        win32::SelectObject(mem_dc, old_font);
        win32::DeleteObject(font);

        win32::BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, win32::SRCCOPY);

        win32::SelectObject(mem_dc, old_bmp);
        win32::DeleteObject(mem_bmp);
        win32::DeleteDC(mem_dc);
    }

    win32::EndPaint(hwnd, &ps);
}

#[cfg(not(windows))]
pub fn init() {}

#[cfg(not(windows))]
pub fn show_dpi(_dpi: u16) {}

#[cfg(windows)]
#[allow(non_snake_case)]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type HDC = *mut c_void;
    pub type HGDIOBJ = *mut c_void;
    pub type HRGN = *mut c_void;
    pub type HINSTANCE = *mut c_void;
    pub type HICON = *mut c_void;
    pub type HCURSOR = *mut c_void;
    pub type HBRUSH = *mut c_void;
    pub type HPEN = *mut c_void;
    pub type HBITMAP = *mut c_void;
    pub type HFONT = *mut c_void;
    pub type HMENU = *mut c_void;
    pub type BOOL = i32;
    pub type WPARAM = usize;
    pub type LPARAM = isize;
    pub type LRESULT = isize;

    pub const CS_VREDRAW: u32 = 0x0001;
    pub const CS_HREDRAW: u32 = 0x0002;

    pub const WS_POPUP: u32 = 0x80000000;
    pub const WS_EX_TOPMOST: u32 = 0x00000008;
    pub const WS_EX_TRANSPARENT: u32 = 0x00000020;
    pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    pub const WS_EX_LAYERED: u32 = 0x00080000;
    pub const WS_EX_NOACTIVATE: u32 = 0x08000000;

    pub const SW_HIDE: i32 = 0;
    pub const SW_SHOWNOACTIVATE: i32 = 4;

    pub const HWND_TOPMOST: HWND = -1isize as HWND;
    pub const SWP_NOACTIVATE: u32 = 0x0010;
    pub const SWP_SHOWWINDOW: u32 = 0x0040;

    pub const LWA_ALPHA: u32 = 0x00000002;

    pub const WM_DESTROY: u32 = 0x0002;
    pub const WM_PAINT: u32 = 0x000F;
    pub const WM_TIMER: u32 = 0x0113;
    pub const WM_USER: u32 = 0x0400;
    pub const WM_SHOW_DPI: u32 = WM_USER + 42;

    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;

    pub const DT_LEFT: u32 = 0x00000000;
    pub const DT_VCENTER: u32 = 0x00000004;
    pub const DT_SINGLELINE: u32 = 0x00000020;

    pub const TRANSPARENT: i32 = 1;
    pub const FW_BOLD: i32 = 700;
    pub const SRCCOPY: u32 = 0x00CC0020;

    #[repr(C)]
    pub struct WNDCLASSEXW {
        pub cbSize: u32,
        pub style: u32,
        pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
        pub cbClsExtra: i32,
        pub cbWndExtra: i32,
        pub hInstance: HINSTANCE,
        pub hIcon: HICON,
        pub hCursor: HCURSOR,
        pub hbrBackground: HBRUSH,
        pub lpszMenuName: *const u16,
        pub lpszClassName: *const u16,
        pub hIconSm: HICON,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
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

    #[repr(C)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    pub struct PAINTSTRUCT {
        pub hdc: HDC,
        pub fErase: BOOL,
        pub rcPaint: RECT,
        pub fRestore: BOOL,
        pub fIncUpdate: BOOL,
        pub rgbReserved: [u8; 32],
    }

    #[link(name = "user32")]
    #[link(name = "gdi32")]
    extern "system" {
        pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
        pub fn CreateWindowExW(
            dwExStyle: u32,
            lpClassName: *const u16,
            lpWindowName: *const u16,
            dwStyle: u32,
            X: i32,
            Y: i32,
            nWidth: i32,
            nHeight: i32,
            hWndParent: HWND,
            hMenu: HMENU,
            hInstance: HINSTANCE,
            lpParam: *const c_void,
        ) -> HWND;
        pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
        pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
        pub fn SetWindowPos(
            hWnd: HWND,
            hWndInsertAfter: HWND,
            X: i32,
            Y: i32,
            cx: i32,
            cy: i32,
            uFlags: u32,
        ) -> BOOL;
        pub fn SetLayeredWindowAttributes(hwnd: HWND, crKey: u32, bAlpha: u8, dwFlags: u32) -> BOOL;
        pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
        pub fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
        pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
        pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
        pub fn CreateSolidBrush(color: u32) -> HBRUSH;
        pub fn CreatePen(iStyle: i32, cWidth: i32, color: u32) -> HPEN;
        pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
        pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
        pub fn RoundRect(
            hdc: HDC,
            left: i32,
            top: i32,
            right: i32,
            bottom: i32,
            width: i32,
            height: i32,
        ) -> BOOL;
        pub fn CreateRoundRectRgn(
            x1: i32,
            y1: i32,
            x2: i32,
            y2: i32,
            w: i32,
            h: i32,
        ) -> HRGN;
        pub fn SetWindowRgn(hWnd: HWND, hRgn: HRGN, bRedraw: BOOL) -> i32;
        pub fn SetTextColor(hdc: HDC, color: u32) -> u32;
        pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
        pub fn DrawTextW(
            hdc: HDC,
            lpchText: *const u16,
            cchText: i32,
            lprc: *mut RECT,
            format: u32,
        ) -> i32;
        pub fn CreateFontW(
            cHeight: i32,
            cWidth: i32,
            cEscapement: i32,
            cOrientation: i32,
            cWeight: i32,
            bItalic: u32,
            bUnderline: u32,
            bStrikeOut: u32,
            iCharSet: u32,
            iOutPrecision: u32,
            iClipPrecision: u32,
            iQuality: u32,
            iPitchAndFamily: u32,
            pszFaceName: *const u16,
        ) -> HFONT;
        pub fn SetTimer(
            hWnd: HWND,
            nIDEvent: usize,
            uElapse: u32,
            lpTimerFunc: *const c_void,
        ) -> usize;
        pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> BOOL;
        pub fn GetSystemMetrics(nIndex: i32) -> i32;
        pub fn GetMessageW(
            lpMsg: *mut MSG,
            hWnd: HWND,
            wMsgFilterMin: u32,
            wMsgFilterMax: u32,
        ) -> BOOL;
        pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
        pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
        pub fn PostMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> BOOL;
        pub fn PostQuitMessage(nExitCode: i32);
        pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
        pub fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP;
        pub fn DeleteDC(hdc: HDC) -> BOOL;
        pub fn BitBlt(
            hdc: HDC,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            hdcSrc: HDC,
            x1: i32,
            y1: i32,
            rop: u32,
        ) -> BOOL;
    }

    pub fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

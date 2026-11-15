use std::ffi::c_void;
use tauri::WebviewWindow;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, POINT, RECT,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
    MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetCursorPos, GetForegroundWindow, GetWindowLongW, GetWindowRect,
    SetWindowLongW, SetWindowPos, ShowWindow, GWL_EXSTYLE, HWND_TOPMOST, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub const WIDGET_WIDTH: f64 = 256.0;
pub const WIDGET_HEIGHT: f64 = 476.0;
pub const MINI_WIDTH: f64 = 212.0;
pub const MINI_HEIGHT: f64 = 48.0;
pub const NOTIFY_WIDTH: f64 = 400.0;
pub const NOTIFY_HEIGHT: f64 = 176.0;

/// Pengawal satu instance menggunakan Windows named mutex.
pub struct SingleInstanceGuard(HANDLE);

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn acquire_single_instance() -> Option<SingleInstanceGuard> {
    let name: Vec<u16> = "Local\\WSolatWidgetSingleInstanceV2\0"
        .encode_utf16()
        .collect();
    unsafe {
        let handle = CreateMutexW(None, false, PCWSTR(name.as_ptr())).ok()?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(handle);
            return None;
        }
        Some(SingleInstanceGuard(handle))
    }
}

pub fn to_hwnd(window: &WebviewWindow) -> Option<HWND> {
    window
        .hwnd()
        .ok()
        .map(|h| HWND(h.0 as *mut c_void))
}

fn primary_monitor_rects() -> Option<(RECT, RECT)> {
    unsafe {
        let hmon = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmon, &mut mi).as_bool() {
            Some((mi.rcMonitor, mi.rcWork))
        } else {
            None
        }
    }
}

pub fn widget_bounds_physical(scale: f64) -> Option<(i32, i32, i32, i32)> {
    let (mon, _) = primary_monitor_rects()?;
    let scale = if scale > 0.0 { scale } else { 1.0 };

    // Bersebelahan widget cuaca Windows 11 di sebelah kiri taskbar
    let logical_x = (155.0 / scale).round() + 90.0;
    let x = mon.left + (logical_x * scale).round() as i32;
    let w = (WIDGET_WIDTH * scale).round() as i32;
    let h = (WIDGET_HEIGHT * scale).round() as i32;
    let y = mon.bottom - h;

    Some((x, y, w, h))
}

pub fn position_widget(window: &WebviewWindow) {
    let Some(hwnd) = to_hwnd(window) else { return };
    let scale = window.scale_factor().unwrap_or(1.0);
    let Some((x, y, w, h)) = widget_bounds_physical(scale) else { return };

    unsafe {
        let mut rect = RECT::default();
        let needs_move = if GetWindowRect(hwnd, &mut rect).is_ok() {
            rect.left != x
                || rect.top != y
                || (rect.right - rect.left) != w
                || (rect.bottom - rect.top) != h
        } else {
            true
        };

        if needs_move {
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                x,
                y,
                w,
                h,
                SWP_NOACTIVATE,
            );
        } else {
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            );
        }
    }
}

pub fn position_notify(window: &WebviewWindow, position_mode: &str) {
    let Some(hwnd) = to_hwnd(window) else { return };
    let Some((mon, work)) = primary_monitor_rects() else { return };
    let scale = window.scale_factor().unwrap_or(1.0);
    let nw = (NOTIFY_WIDTH * scale).round() as i32;
    let nh = (NOTIFY_HEIGHT * scale).round() as i32;

    // Jika taskbar auto-hide (work.bottom == mon.bottom), anggaran jarak taskbar 48dp
    let anchor_bottom = if work.bottom < mon.bottom {
        work.bottom
    } else {
        mon.bottom - (48.0 * scale).round() as i32
    };

    let (x, y) = if position_mode == "corner" {
        let margin_r = (12.0 * scale).round() as i32;
        let margin_b = (10.0 * scale).round() as i32;
        (work.right - nw - margin_r, anchor_bottom - nh - margin_b)
    } else {
        let wx = widget_bounds_physical(scale).map(|b| b.0).unwrap_or(work.left + 16);
        let margin_b = (8.0 * scale).round() as i32;
        let x = (wx - (4.0 * scale).round() as i32).max(work.left + 8);
        (x, anchor_bottom - nh - margin_b)
    };

    unsafe {
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, x, y, nw, nh, SWP_NOACTIVATE);
    }
}

pub fn configure_overlay_style(window: &WebviewWindow, no_activate: bool) {
    let Some(hwnd) = to_hwnd(window) else { return };
    unsafe {
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let mut next = ex | (WS_EX_TOOLWINDOW.0 as i32);
        if no_activate {
            next |= WS_EX_NOACTIVATE.0 as i32;
        } else {
            next &= !(WS_EX_NOACTIVATE.0 as i32);
        }
        if next != ex {
            SetWindowLongW(hwnd, GWL_EXSTYLE, next);
            let _ = SetWindowPos(
                hwnd,
                HWND::default(),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
        }
    }
}

pub fn show_inactive(window: &WebviewWindow) {
    let Some(hwnd) = to_hwnd(window) else { return };
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
        );
    }
}

pub fn hide_window(window: &WebviewWindow) {
    let Some(hwnd) = to_hwnd(window) else { return };
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn is_cursor_over_minibar(window: &WebviewWindow) -> bool {
    let Some(hwnd) = to_hwnd(window) else { return false };
    let scale = window.scale_factor().unwrap_or(1.0);
    let pad_x = (((WIDGET_WIDTH - MINI_WIDTH) / 2.0) * scale).round() as i32;
    let mini_w = (MINI_WIDTH * scale).round() as i32;
    let mini_h = (MINI_HEIGHT * scale).round() as i32;

    unsafe {
        let mut pt = POINT::default();
        let mut rect = RECT::default();
        if GetCursorPos(&mut pt).is_err() || GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        pt.x >= rect.left + pad_x
            && pt.x <= rect.left + pad_x + mini_w
            && pt.y >= rect.bottom - mini_h
            && pt.y <= rect.bottom
    }
}

pub fn is_fullscreen_active(own_hwnds: &[HWND]) -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() || own_hwnds.iter().any(|h| h.0 == fg.0) {
            return false;
        }

        let mut class_buf = [0u16; 256];
        let len = GetClassNameW(fg, &mut class_buf);
        if len == 0 {
            return false;
        }
        let class_name = String::from_utf16_lossy(&class_buf[..len as usize]);
        if matches!(
            class_name.as_str(),
            "WorkerW" | "Progman" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
        ) {
            return false;
        }

        let mut fg_rect = RECT::default();
        if GetWindowRect(fg, &mut fg_rect).is_err() {
            return false;
        }

        let hmon = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(hmon, &mut mi).as_bool() {
            return false;
        }

        let mon_w = mi.rcMonitor.right - mi.rcMonitor.left;
        let mon_h = mi.rcMonitor.bottom - mi.rcMonitor.top;
        let fg_w = fg_rect.right - fg_rect.left;
        let fg_h = fg_rect.bottom - fg_rect.top;

        fg_w >= mon_w && fg_h >= mon_h
    }
}

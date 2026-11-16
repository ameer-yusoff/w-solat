use std::ffi::c_void;
use std::sync::Mutex;
use tauri::{Emitter, WebviewWindow};
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;
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
    SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub const WIDGET_WIDTH: f64 = 320.0;
pub const WIDGET_HEIGHT: f64 = 476.0;
pub const MINI_WIDTH: f64 = 246.0;
pub const MINI_HEIGHT: f64 = 48.0;
pub const NOTIFY_WIDTH: f64 = 400.0;
pub const NOTIFY_HEIGHT: f64 = 216.0;

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

const POSITION_KEY: &str = r"Software\W-Solat";
const POSITION_VALUE: &str = "WidgetPosition";

#[derive(Default)]
struct WidgetPlacement {
    // Position of the mini bar relative to the primary display, in logical pixels.
    anchor: Option<(i32, i32)>,
    editing: bool,
    drag_offset: Option<(i32, i32)>,
    bar_at_top: bool,
}

static PLACEMENT: Mutex<WidgetPlacement> = Mutex::new(WidgetPlacement {
    anchor: None,
    editing: false,
    drag_offset: None,
    bar_at_top: false,
});

pub fn restore_widget_position() {
    let saved = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(POSITION_KEY)
        .and_then(|key| key.get_value::<String, _>(POSITION_VALUE))
        .ok()
        .and_then(|value| serde_json::from_str(&value).ok());
    PLACEMENT.lock().unwrap().anchor = saved;
}

fn widget_geometry(
    window: &WebviewWindow,
    placement: &WidgetPlacement,
) -> Option<(RECT, f64, i32, i32, i32, i32, i32)> {
    let (mon, _) = primary_monitor_rects()?;
    let scale = window.scale_factor().unwrap_or(1.0);
    let w = (WIDGET_WIDTH * scale).round() as i32;
    let h = (WIDGET_HEIGHT * scale).round() as i32;
    let mini_h = (MINI_HEIGHT * scale).round() as i32;
    let (x, bar_y) = placement.anchor.map_or_else(
        || (mon.left + 155 + (58.0 * scale).round() as i32,
            mon.bottom - mini_h),
        |(x, y)| (mon.left + (f64::from(x) * scale).round() as i32,
                   mon.top + (f64::from(y) * scale).round() as i32),
    );
    let x = x.clamp(mon.left, (mon.right - w).max(mon.left));
    let bar_y = bar_y.clamp(mon.top, (mon.bottom - mini_h).max(mon.top));
    Some((mon, scale, x, bar_y, w, h, mini_h))
}

fn apply_widget_position(window: &WebviewWindow, placement: &mut WidgetPlacement) {
    let Some(hwnd) = to_hwnd(window) else { return };
    let Some((mon, _, x, bar_y, w, h, mini_h)) = widget_geometry(window, placement) else { return };
    let above = bar_y - mon.top;
    let below = mon.bottom - bar_y - mini_h;
    let at_top = above < h - mini_h && below > above;
    let y = if at_top { bar_y } else { bar_y + mini_h - h };

    unsafe {
        let mut rect = RECT::default();
        let unchanged = GetWindowRect(hwnd, &mut rect).is_ok()
            && rect.left == x && rect.top == y
            && rect.right - rect.left == w && rect.bottom - rect.top == h;
        let mut flags = SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS;
        if unchanged {
            flags |= SWP_NOMOVE | SWP_NOSIZE;
        }
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, x, y, w, h, flags);
    }
    if placement.bar_at_top != at_top {
        placement.bar_at_top = at_top;
        let _ = window.emit("widget-edge", at_top);
    }
}

pub fn position_widget(window: &WebviewWindow) {
    apply_widget_position(window, &mut PLACEMENT.lock().unwrap());
}

pub fn widget_at_top() -> bool {
    PLACEMENT.lock().unwrap().bar_at_top
}

pub fn edit_widget_position() {
    PLACEMENT.lock().unwrap().editing = true;
}

pub fn drag_widget(window: &WebviewWindow, phase: &str) {
    let mut placement = PLACEMENT.lock().unwrap();
    if !placement.editing { return; }
    if phase == "cancel" {
        placement.drag_offset = None;
        return;
    }
    let Some((mon, scale, x, bar_y, w, _, mini_h)) = widget_geometry(window, &placement) else { return };
    let mut cursor = POINT::default();
    if unsafe { GetCursorPos(&mut cursor) }.is_err() { return; }
    match phase {
        "start" => placement.drag_offset = Some((cursor.x - x, cursor.y - bar_y)),
        "move" | "end" => {
            let Some((dx, dy)) = placement.drag_offset else { return };
            let x = (cursor.x - dx).clamp(mon.left, (mon.right - w).max(mon.left));
            let y = (cursor.y - dy).clamp(mon.top, (mon.bottom - mini_h).max(mon.top));
            placement.anchor = Some((((x - mon.left) as f64 / scale).round() as i32,
                                     ((y - mon.top) as f64 / scale).round() as i32));
            if phase == "end" { placement.drag_offset = None; }
            apply_widget_position(window, &mut placement);
        }
        _ => {}
    }
}

pub fn save_widget_position() -> Result<(), String> {
    let mut placement = PLACEMENT.lock().unwrap();
    if let Some(anchor) = placement.anchor {
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(POSITION_KEY)
            .map_err(|err| err.to_string())?;
        let value = serde_json::to_string(&anchor).map_err(|err| err.to_string())?;
        key.set_value(POSITION_VALUE, &value).map_err(|err| err.to_string())?;
    }
    placement.drag_offset = None;
    placement.editing = false;
    Ok(())
}

pub fn position_notify(window: &WebviewWindow, widget: Option<&WebviewWindow>, position_mode: &str) {
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
        let placement = PLACEMENT.lock().unwrap();
        let margin = (8.0 * scale).round() as i32;
        let Some((_, _, wx, bar_y, _, _, mini_h)) = widget
            .and_then(|widget| widget_geometry(widget, &placement))
        else { return };
        let x = wx.clamp(work.left, (work.right - nw).max(work.left));
        let y = if bar_y - nh - margin >= work.top {
            bar_y - nh - margin
        } else {
            (bar_y + mini_h + margin).min(work.bottom - nh).max(work.top)
        };
        (x, y)
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
    let placement = PLACEMENT.lock().unwrap();
    if placement.drag_offset.is_some() { return true; }
    let width = if placement.editing { WIDGET_WIDTH } else { MINI_WIDTH };
    let pad_x = (((WIDGET_WIDTH - width) / 2.0) * scale).round() as i32;
    let mini_w = (width * scale).round() as i32;
    let mini_h = (MINI_HEIGHT * scale).round() as i32;

    unsafe {
        let mut pt = POINT::default();
        let mut rect = RECT::default();
        if GetCursorPos(&mut pt).is_err() || GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }
        let bar_y = if placement.bar_at_top { rect.top } else { rect.bottom - mini_h };
        pt.x >= rect.left + pad_x
            && pt.x <= rect.left + pad_x + mini_w
            && pt.y >= bar_y && pt.y <= bar_y + mini_h
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

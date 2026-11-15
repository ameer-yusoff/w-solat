mod win32;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const AUTOSTART_NAME: &str = "W-Solat Widget";

#[derive(Default)]
struct AppState {
    expanded: Arc<AtomicBool>,
    ignore_cursor: Arc<AtomicBool>,
    pending_notif: Mutex<Option<NotificationPayload>>,
    tray: Mutex<Option<TrayIcon>>,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPayload {
    pub kind: String,
    pub key: String,
    pub name: String,
    pub ar: String,
    pub time: String,
    pub location: String,
    pub minutes: u32,
    pub sound: String,
    pub volume: f64,
    pub duration: u32,
    pub position: String,
    pub over_fullscreen: bool,
    pub hijri: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoInfo {
    pub country_code: String,
    pub region: String,
    pub city: String,
}

#[tauri::command]
async fn detect_location(state: State<'_, AppState>) -> Result<GeoInfo, String> {
    if let Ok(resp) = state
        .http
        .get("https://ipwho.is/")
        .timeout(Duration::from_secs(6))
        .send()
        .await
    {
        if let Ok(v) = resp.json::<serde_json::Value>().await {
            if v.get("success").and_then(|b| b.as_bool()).unwrap_or(false) {
                return Ok(GeoInfo {
                    country_code: v["country_code"].as_str().unwrap_or("").to_string(),
                    region: v["region"].as_str().unwrap_or("").to_string(),
                    city: v["city"].as_str().unwrap_or("").to_string(),
                });
            }
        }
    }

    let resp = state
        .http
        .get("https://ipapi.co/json/")
        .header("User-Agent", "W-Solat/2.0")
        .timeout(Duration::from_secs(6))
        .send()
        .await
        .map_err(|e| format!("Ralat lokasi: {e}"))?;
    let v = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Ralat JSON lokasi: {e}"))?;

    Ok(GeoInfo {
        country_code: v["country_code"].as_str().unwrap_or("").to_string(),
        region: v["region"].as_str().unwrap_or("").to_string(),
        city: v["city"].as_str().unwrap_or("").to_string(),
    })
}

#[tauri::command]
async fn fetch_prayer_times(
    state: State<'_, AppState>,
    zone: String,
    period: String,
) -> Result<serde_json::Value, String> {
    let clean_zone: String = zone
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let clean_period = match period.as_str() {
        "month" => "month",
        "week" => "week",
        _ => "today",
    };
    let url = format!(
        "https://www.e-solat.gov.my/index.php?r=esolatApi/takwimsolat&zone={}&period={}",
        clean_zone, clean_period
    );
    let resp = state
        .http
        .get(&url)
        .timeout(Duration::from_secs(12))
        .send()
        .await
        .map_err(|e| format!("Ralat rangkaian: {e}"))?;
    let json = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Ralat JSON: {e}"))?;
    Ok(json)
}

#[tauri::command]
fn show_notification(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: NotificationPayload,
) -> Result<(), String> {
    let own: Vec<_> = ["main", "notify"]
        .iter()
        .filter_map(|l| app.get_webview_window(l).and_then(|w| win32::to_hwnd(&w)))
        .collect();

    if !payload.over_fullscreen && win32::is_fullscreen_active(&own) {
        return Ok(());
    }

    if let Ok(mut guard) = state.pending_notif.lock() {
        *guard = Some(payload.clone());
    }

    let notify = app
        .get_webview_window("notify")
        .ok_or_else(|| "Tetingkap notifikasi tidak dijumpai".to_string())?;

    win32::configure_overlay_style(&notify, true);
    win32::position_notify(&notify, &payload.position);
    win32::show_inactive(&notify);
    let _ = notify.emit("prayer-alert", &payload);

    Ok(())
}

#[tauri::command]
fn hide_notification(app: AppHandle, state: State<'_, AppState>) {
    if let Ok(mut guard) = state.pending_notif.lock() {
        *guard = None;
    }
    if let Some(notify) = app.get_webview_window("notify") {
        win32::hide_window(&notify);
    }
}

#[tauri::command]
fn get_pending_notification(state: State<'_, AppState>) -> Option<NotificationPayload> {
    state.pending_notif.lock().ok().and_then(|g| g.clone())
}

#[tauri::command]
fn set_expanded(app: AppHandle, state: State<'_, AppState>, expanded: bool) {
    state.expanded.store(expanded, Ordering::SeqCst);
    if let Some(main) = app.get_webview_window("main") {
        win32::configure_overlay_style(&main, !expanded);
        if expanded {
            state.ignore_cursor.store(false, Ordering::SeqCst);
            let _ = main.set_ignore_cursor_events(false);
            let _ = main.set_focus();
        }
    }
}

#[tauri::command]
fn set_tray_tooltip(state: State<'_, AppState>, text: String) {
    if let Ok(guard) = state.tray.lock() {
        if let Some(tray) = guard.as_ref() {
            let _ = tray.set_tooltip(Some(text));
        }
    }
}

#[tauri::command]
fn get_autostart() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey_with_flags(RUN_KEY_PATH, KEY_READ) else {
        return false;
    };
    key.get_value::<String, _>(AUTOSTART_NAME).is_ok()
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey_with_flags(RUN_KEY_PATH, KEY_WRITE)
        .map_err(|e| e.to_string())?;
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let quoted = format!("\"{}\"", exe.display());
        key.set_value(AUTOSTART_NAME, &quoted)
            .map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        let _ = key.delete_value(AUTOSTART_NAME);
        Ok(false)
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let toggle_item = MenuItem::with_id(app, "toggle", "Buka / Tutup Jadual", true, None::<&str>)?;
    let test_item = MenuItem::with_id(app, "test", "Uji Notifikasi", true, None::<&str>)?;
    let refresh_item = MenuItem::with_id(app, "refresh", "Muat Semula Data", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle_item, &test_item, &refresh_item, &quit_item])?;

    let mut builder = TrayIconBuilder::with_id("w-solat-tray")
        .menu(&menu)
        .tooltip("W-Solat")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, ev| match ev.id.as_ref() {
            "toggle" => {
                if let Some(main) = app.get_webview_window("main") {
                    win32::show_inactive(&main);
                    win32::position_widget(&main);
                }
                let _ = app.emit_to("main", "toggle-panel", ());
            }
            "test" => {
                let _ = app.emit_to("main", "test-notif", ());
            }
            "refresh" => {
                let _ = app.emit_to("main", "refresh", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = ev
            {
                let app = tray.app_handle();
                if let Some(main) = app.get_webview_window("main") {
                    win32::show_inactive(&main);
                    win32::position_widget(&main);
                }
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)
}

fn spawn_win32_watchers(app: AppHandle) {
    let state = app.state::<AppState>();
    let expanded = Arc::clone(&state.expanded);
    let ignore_cursor = Arc::clone(&state.ignore_cursor);

    // Watcher 1: Pengesan kursor klik-tembus (50ms) — tiada polling PowerShell lagi
    let app_cursor = app.clone();
    let exp_cursor = Arc::clone(&expanded);
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(50));
        let Some(main) = app_cursor.get_webview_window("main") else {
            continue;
        };
        if !main.is_visible().unwrap_or(false) {
            continue;
        }
        let want_ignore = if exp_cursor.load(Ordering::SeqCst) {
            false
        } else {
            !win32::is_cursor_over_minibar(&main)
        };
        if ignore_cursor.swap(want_ignore, Ordering::SeqCst) != want_ignore {
            let _ = main.set_ignore_cursor_events(want_ignore);
        }
    });

    // Watcher 2: Pengesan skrin penuh + penyelarasan taskbar (250ms & 3000ms)
    std::thread::spawn(move || {
        // Jujukan penyelarasan awal (atasi kelewatan Explorer semasa Windows startup)
        for ms in [300_u64, 1200, 2500, 5000] {
            std::thread::sleep(Duration::from_millis(ms));
            if let Some(main) = app.get_webview_window("main") {
                if main.is_visible().unwrap_or(false) {
                    win32::position_widget(&main);
                }
            }
        }

        let mut hidden_by_fs = false;
        let mut tick: u32 = 0;

        loop {
            std::thread::sleep(Duration::from_millis(250));
            tick = tick.wrapping_add(1);

            let Some(main) = app.get_webview_window("main") else {
                continue;
            };
            let own: Vec<_> = ["main", "notify"]
                .iter()
                .filter_map(|l| app.get_webview_window(l).and_then(|w| win32::to_hwnd(&w)))
                .collect();

            let fs = win32::is_fullscreen_active(&own);
            if fs && !hidden_by_fs {
                hidden_by_fs = true;
                if expanded.load(Ordering::SeqCst) {
                    let _ = app.emit_to("main", "close-panel", ());
                }
                win32::hide_window(&main);
            } else if !fs && hidden_by_fs {
                hidden_by_fs = false;
                win32::position_widget(&main);
                win32::show_inactive(&main);
            } else if !fs && tick % 12 == 0 {
                // Failsafe setiap ~3s supaya kekal di atas taskbar
                win32::position_widget(&main);
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _instance_guard = match win32::acquire_single_instance() {
        Some(g) => g,
        None => return,
    };

    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            detect_location,
            fetch_prayer_times,
            show_notification,
            hide_notification,
            get_pending_notification,
            set_expanded,
            set_tray_tooltip,
            get_autostart,
            set_autostart,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            if let Some(main) = app.get_webview_window("main") {
                win32::configure_overlay_style(&main, true);
                win32::position_widget(&main);
                win32::show_inactive(&main);

                let h_blur = handle.clone();
                main.on_window_event(move |ev| {
                    if let WindowEvent::Focused(false) = ev {
                        let st = h_blur.state::<AppState>();
                        if st.expanded.load(Ordering::SeqCst) {
                            let _ = h_blur.emit_to("main", "close-panel", ());
                        }
                    }
                });
            }

            if let Some(notify) = app.get_webview_window("notify") {
                win32::configure_overlay_style(&notify, true);
                win32::hide_window(&notify);
            }

            if let Ok(tray) = setup_tray(&handle) {
                if let Ok(mut guard) = app.state::<AppState>().tray.lock() {
                    *guard = Some(tray);
                }
            }

            spawn_win32_watchers(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Gagal menjalankan aplikasi W-Solat");
}

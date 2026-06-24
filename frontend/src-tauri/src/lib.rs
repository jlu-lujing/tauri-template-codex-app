//! Tauri Template — Desktop application with borderless window, status bar, and sidebar layout
//!
//! This crate provides the IPC layer between the frontend and Tauri backend.

pub mod commands;

#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

use serde::{Deserialize, Serialize};
use tauri::{Manager, PhysicalPosition, PhysicalSize, Position, Size, WindowEvent};

/// Window size stored for persistence.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct StoredSize {
    w: u32,
    h: u32,
}

/// Minimum window dimensions (must match tauri.conf.json).
const MIN_WIDTH: u32 = 1024;
const MIN_HEIGHT: u32 = 680;

/// Target aspect ratio: 3:2 = 1.5.
const ASPECT_RATIO: f64 = 1.5;

/// Fraction of monitor to use for default window size.
const MONITOR_FRACTION: f64 = 0.85;

/// Compute a default window size for the given monitor using a 3:2 aspect ratio.
///
/// The window will not exceed 85% of the monitor's physical size.
/// Priority: fit width first (w = max_w, h = w / 1.5), then fit height (h = max_h, w = h * 1.5).
/// Result is clamped to [MIN_WIDTH, MIN_HEIGHT].
fn compute_default_size(monitor: &tauri::Monitor) -> (u32, u32) {
    let physical = monitor.size();
    let max_w = (physical.width as f64 * MONITOR_FRACTION).round() as u32;
    let max_h = (physical.height as f64 * MONITOR_FRACTION).round() as u32;

    // Try width-first: w = max_w, h = w / 1.5
    let (w, h) = {
        let w = max_w;
        let h = (w as f64 / ASPECT_RATIO).round() as u32;
        if h <= max_h {
            (w, h)
        } else {
            // Fall back to height-first: h = max_h, w = h * 1.5
            let h = max_h;
            let w = (h as f64 * ASPECT_RATIO).round() as u32;
            (w, h)
        }
    };

    // Clamp to minimums
    let w = w.max(MIN_WIDTH);
    let h = h.max(MIN_HEIGHT);

    (w, h)
}

/// Read persisted window size from disk.
fn load_window_size(app: &tauri::AppHandle) -> Option<(u32, u32)> {
    let path = match app.path().app_data_dir() {
        Ok(dir) => dir.join("window_size.json"),
        Err(_) => return None,
    };

    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return None,
    };

    let parsed: StoredSize = match serde_json::from_str(&data) {
        Ok(p) => p,
        Err(_) => return None,
    };

    Some((parsed.w, parsed.h))
}

/// Write window size to disk.
fn save_window_size(app: &tauri::AppHandle, w: u32, h: u32) {
    let path = match app.path().app_data_dir() {
        Ok(dir) => dir.join("window_size.json"),
        Err(_) => return,
    };

    let stored = StoredSize { w, h };
    let data = match serde_json::to_string_pretty(&stored) {
        Ok(d) => d,
        Err(_) => return,
    };

    let _ = std::fs::write(&path, data);
}

/// Resize the window to the given (width, height).
fn resize_window(window: &tauri::WebviewWindow, w: u32, h: u32) {
    let _ = window.set_size(Size::Physical(PhysicalSize::<u32>::new(w, h)));
}

/// Center the window on the given monitor using **physical** coordinates.
fn center_window_on_monitor(
    window: &tauri::WebviewWindow,
    monitor: &tauri::Monitor,
    window_size: PhysicalSize<u32>,
) {
    let monitor_size = monitor.size();
    let monitor_pos = monitor.position();

    let center_x = monitor_pos.x
        + ((monitor_size.width as i32 - window_size.width as i32) / 2);
    let center_y = monitor_pos.y
        + ((monitor_size.height as i32 - window_size.height as i32) / 2);

    let _ = window.set_position(Position::Physical(PhysicalPosition::<i32>::new(center_x, center_y)));
}

/// Application entry point — called from main.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::window::win_minimize,
            commands::window::win_maximize,
            commands::window::win_close,
            commands::window::win_start_drag,
        ])
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .expect("main window not found");

            // macOS: enable window shadow for borderless transparent window
            #[cfg(target_os = "macos")]
            {
                let _ = window.set_shadow(true);

                // Apply native macOS vibrancy (NSVisualEffectView) — blurs the
                // desktop wallpaper through the window. The webview stays
                // transparent so the vibrancy shows through.
                apply_vibrancy(
                    &window,
                    NSVisualEffectMaterial::HudWindow,
                    Some(NSVisualEffectState::Active),
                    None,
                )
                .expect("failed to apply vibrancy");

                // macOS: fix focus loss on frameless window when switching back
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(true) = event {
                        let _ = w.set_focus();
                    }
                });

                // macOS: delayed activation on first launch
                let w0 = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let _ = w0.set_focus();
                });
            }

            // ── Load persisted window size ───────────────────────────────
            let stored_size = load_window_size(app.handle());

            // ── Window positioning strategy ──────────────────────────────
            // 1. Find the monitor where the mouse cursor currently is
            // 2. Apply stored size (or compute 3:2 default based on monitor)
            // 3. Center using physical coordinates
            // ─────────────────────────────────────────────────────────────

            // Try to find the monitor under the cursor
            let target_monitor = match window.cursor_position() {
                Ok(cursor_logical) => {
                    let monitors = app.available_monitors().unwrap_or_default();
                    monitors
                        .into_iter()
                        .find(|m| {
                            let pos = m.position();
                            let size = m.size();
                            cursor_logical.x >= pos.x as f64
                                && cursor_logical.x
                                    < (pos.x as f64 + size.width as f64)
                                && cursor_logical.y >= pos.y as f64
                                && cursor_logical.y
                                    < (pos.y as f64 + size.height as f64)
                        })
                }
                Err(_) => None,
            };

            // Fall back to primary monitor
            let monitor = match target_monitor {
                Some(m) => m,
                None => app
                    .primary_monitor()
                    .expect("failed to get primary monitor")
                    .expect("no primary monitor"),
            };

            // Apply stored size or compute default based on monitor
            let (w, h) = match stored_size {
                Some(s) => s,
                None => compute_default_size(&monitor),
            };

            resize_window(&window, w, h);
            std::thread::sleep(std::time::Duration::from_millis(50));

            if let Ok(size) = window.outer_size() {
                center_window_on_monitor(&window, &monitor, size);
            }

            // ── Save window size on close ────────────────────────────────
            let app_handle = app.handle().clone();
            let close_window = window.clone();
            close_window.clone().on_window_event(move |event| {
                if let WindowEvent::CloseRequested { .. } = event {
                    let app = app_handle.clone();
                    let win = close_window.clone();

                    // Save on main thread to ensure size is up-to-date
                    let _ = app.run_on_main_thread({
                        let app = app.clone();
                        move || {
                            if let Ok(size) = win.outer_size() {
                                save_window_size(&app, size.width, size.height);
                            }
                        }
                    });
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

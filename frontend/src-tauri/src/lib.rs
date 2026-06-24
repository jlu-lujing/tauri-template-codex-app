//! Tauri Template — Desktop application with borderless window, status bar, and sidebar layout
//!
//! This crate provides the IPC layer between the frontend and Tauri backend.

pub mod commands;

#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Listener, Manager, PhysicalPosition, PhysicalSize, Position, Size, WindowEvent};

/// Window size stored per monitor for persistence.
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

/// Tracks per-monitor window sizes and the current monitor state.
#[derive(Debug)]
struct MonitorState {
    /// The current monitor's name (for identity comparison).
    last_monitor_name: Mutex<Option<String>>,
    /// Per-monitor stored sizes: monitor_name -> (width, height).
    per_monitor_size: Mutex<HashMap<String, (u32, u32)>>,
    /// If Some(old_name), a monitor change was detected and we're waiting
    /// for the frontend "drag-ended" event before restoring the new monitor's size.
    pending_restore: Mutex<Option<String>>,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            last_monitor_name: Mutex::new(None),
            per_monitor_size: Mutex::new(HashMap::new()),
            pending_restore: Mutex::new(None),
        }
    }
}

/// Get the monitor name as a String (or empty string if unavailable).
fn monitor_name(monitor: &tauri::Monitor) -> String {
    monitor.name().map(|n| n.to_string()).unwrap_or_default()
}

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

/// Read persisted window sizes from disk.
fn load_window_sizes(app: &tauri::AppHandle) -> HashMap<String, (u32, u32)> {
    let path = match app.path().app_data_dir() {
        Ok(dir) => dir.join("window_sizes.json"),
        Err(_) => return HashMap::new(),
    };

    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return HashMap::new(),
    };

    let parsed: HashMap<String, StoredSize> = match serde_json::from_str(&data) {
        Ok(p) => p,
        Err(_) => return HashMap::new(),
    };

    parsed.into_iter().map(|(k, v)| (k, (v.w, v.h))).collect()
}

/// Write per-monitor window sizes to disk.
fn save_window_sizes(app: &tauri::AppHandle, sizes: &HashMap<String, (u32, u32)>) {
    let path = match app.path().app_data_dir() {
        Ok(dir) => dir.join("window_sizes.json"),
        Err(_) => return,
    };

    let stored: HashMap<String, StoredSize> = sizes
        .iter()
        .map(|(k, (w, h))| (k.clone(), StoredSize { w: *w, h: *h }))
        .collect();

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
///
/// We do NOT use `window.center()` because for transparent borderless
/// windows on macOS the NSWindow frame isn't fully initialized at that
/// point, causing incorrect centering (especially with multiple monitors
/// arranged in non-standard layouts).
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

/// Apply the stored (or default) size for the given monitor and center the window.
fn apply_monitor_size(
    window: &tauri::WebviewWindow,
    monitor: &tauri::Monitor,
    monitor_state: &MonitorState,
) {
    let name = monitor_name(monitor);
    let (w, h) = {
        let mut sizes = monitor_state.per_monitor_size.lock().expect("lock poisoned");
        if let Some(&size) = sizes.get(&name) {
            size
        } else {
            let default = compute_default_size(monitor);
            sizes.insert(name.clone(), default);
            default
        }
    };

    resize_window(window, w, h);

    // Small delay to let the resize settle before centering
    std::thread::sleep(std::time::Duration::from_millis(50));

    if let Ok(size) = window.outer_size() {
        center_window_on_monitor(window, monitor, size);
    }
}

/// Check if the window is on a different monitor than before.
/// Returns Some(old_name) if the monitor changed, None otherwise.
///
/// When a change is detected, `last_monitor_name` is updated to the new monitor.
fn check_monitor_changed(
    window: &tauri::WebviewWindow,
    monitor_state: &MonitorState,
) -> Option<String> {
    let current_monitor = match window.current_monitor() {
        Ok(Some(m)) => m,
        _ => return None,
    };

    let current_name = monitor_name(&current_monitor);

    let mut last_name = monitor_state
        .last_monitor_name
        .lock()
        .expect("lock poisoned");

    match last_name.as_ref() {
        None => {
            // First time — record this monitor
            *last_name = Some(current_name);
            None
        }
        Some(prev) if prev.as_str() != current_name => {
            // Monitor changed!
            let old_name = prev.clone();
            *last_name = Some(current_name);
            Some(old_name)
        }
        _ => None,
    }
}

/// Collect data needed for restore (called on main thread, no side effects).
/// Returns Some((old_name, new_monitor, current_size, target_w, target_h))
/// if a resize is needed, None otherwise.
fn collect_restore_data(
    app: &tauri::AppHandle,
    monitor_state: &MonitorState,
    old_name: String,
) -> Option<(String, tauri::Monitor, PhysicalSize<u32>, u32, u32)> {
    let window = app.get_webview_window("main")?;
    let new_monitor = window.current_monitor().ok().flatten()?;
    let new_name = monitor_name(&new_monitor);
    let current_size = window.outer_size().ok()?;

    let (target_w, target_h) = {
        let mut sizes = monitor_state.per_monitor_size.lock().expect("lock poisoned");
        sizes.insert(old_name.clone(), (current_size.width, current_size.height));
        match sizes.get(&new_name) {
            Some(&s) => s,
            None => {
                let default = compute_default_size(&new_monitor);
                sizes.insert(new_name.clone(), default);
                default
            }
        }
    };

    if current_size.width == target_w && current_size.height == target_h {
        // No resize needed, but still persist
        let sizes = monitor_state.per_monitor_size.lock().expect("lock poisoned");
        save_window_sizes(app, &sizes);
        return None;
    }

    Some((old_name, new_monitor, current_size, target_w, target_h))
}

/// Perform the restore: save old monitor size, restore new monitor size, resize + center.
/// MUST be called on the main thread.
fn perform_restore(app: &tauri::AppHandle, monitor_state: &MonitorState, old_name: String) {
    let Some((_old_name, new_monitor, _current_size, target_w, target_h)) =
        collect_restore_data(app, monitor_state, old_name)
    else {
        return;
    };

    if let Some(window) = app.get_webview_window("main") {
        resize_window(&window, target_w, target_h);
    }

    // Small delay to let the resize settle before centering
    std::thread::sleep(std::time::Duration::from_millis(50));

    if let Some(window) = app.get_webview_window("main") {
        if let Ok(new_size) = window.outer_size() {
            center_window_on_monitor(&window, &new_monitor, new_size);
        }
    }

    // Persist to disk (on a background thread to avoid blocking)
    let sizes = monitor_state.per_monitor_size.lock().expect("lock poisoned").clone();
    let app_clone = app.clone();
    let _ = std::thread::spawn(move || {
        save_window_sizes(&app_clone, &sizes);
    });
}

/// Application entry point — called from main.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(MonitorState::default())
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

            // ── Load persisted per-monitor sizes ─────────────────────────
            let monitor_state = app.state::<MonitorState>();
            let stored_sizes = load_window_sizes(app.handle());
            {
                let mut sizes = monitor_state.per_monitor_size.lock().expect("lock poisoned");
                *sizes = stored_sizes;
            }

            // ── Window positioning strategy ──────────────────────────────
            // 1. Find the monitor where the mouse cursor currently is
            // 2. Apply stored size for that monitor (or compute 3:2 default)
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

            // Apply stored or default size and center
            apply_monitor_size(&window, &monitor, &monitor_state);

            // Record the current monitor
            {
                let mut name = monitor_state.last_monitor_name.lock().expect("lock poisoned");
                *name = Some(monitor_name(&monitor));
            }

            // ── Cross-monitor resize via polling + frontend event ────────
            //
            // Transparent borderless windows on macOS do NOT reliably fire
            // WindowEvent::Moved when dragged between monitors. We use a
            // two-part approach:
            //
            // 1. **Polling (100ms):** Detect monitor changes while the user
            //    is dragging. When a change is detected, set
            //    `pending_restore = Some(old_name)`. Do NOT resize yet.
            //
            // 2. **Frontend "drag-ended" event:** The frontend emits this
            //    event on `document` `mouseup`. When received, if
            //    `pending_restore` is Some, perform the restore:
            //    - Save old monitor's current window size
            //    - Restore new monitor's stored size (or compute default)
            //    - Resize and center
            //    - Persist to disk
            //
            // This approach is reliable because the frontend can detect
            // mouse-up, which is the actual end of a drag. Polling alone
            // can't distinguish "paused during drag" from "released".
            // ─────────────────────────────────────────────────────────────

            // Part 1: Polling — detect monitor changes during drag
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // Wait a bit before starting to poll (let the window settle)
                std::thread::sleep(std::time::Duration::from_secs(2));

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(100));

                    let window = match app_handle.get_webview_window("main") {
                        Some(w) => w,
                        None => break, // window closed
                    };

                    let monitor_state = app_handle.state::<MonitorState>();

                    // Check for monitor change
                    if let Some(old_name) = check_monitor_changed(&window, &monitor_state) {
                        // A monitor change was detected. Mark it but do NOT
                        // resize yet — the user is still dragging.
                        let mut pending = monitor_state.pending_restore.lock().expect("lock poisoned");
                        *pending = Some(old_name);
                    }
                }
            });

            // Part 2: Frontend event — restore when drag ends
            let app_handle = app.handle().clone();
            app_handle.clone().listen("drag-ended", move |_event| {
                let app = app_handle.clone();

                // Take the pending restore (if any)
                let old_name = {
                    let monitor_state = app.state::<MonitorState>();
                    let mut pending = monitor_state.pending_restore.lock().expect("lock poisoned");
                    pending.take()
                };

                if let Some(old_name) = old_name {
                    // Schedule restore after a short delay to let the window
                    // settle after mouse-up. Use a background thread for the
                    // delay, then dispatch to main thread for the actual work.
                    let app_clone = app.clone();
                    let _ = std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        let app2 = app_clone.clone();
                        let _ = app_clone.run_on_main_thread(move || {
                            let monitor_state = app2.state::<MonitorState>();
                            perform_restore(&app2, &monitor_state, old_name);
                        });
                    });
                }
            });

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
                            let monitor_state = app.state::<MonitorState>();
                            if let Ok(Some(monitor)) = win.current_monitor() {
                                let name = monitor_name(&monitor);
                                if let Ok(size) = win.outer_size() {
                                    let mut sizes = monitor_state.per_monitor_size.lock().expect("lock poisoned");
                                    sizes.insert(name.clone(), (size.width, size.height));
                                    // Persist to disk
                                    save_window_sizes(&app, &sizes);
                                }
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

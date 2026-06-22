//! Tauri Template — Desktop application with borderless window, status bar, and sidebar layout
//!
//! This crate provides the IPC layer between the frontend and Tauri backend.

pub mod commands;

use std::sync::Mutex;
use tauri::{Manager, PhysicalPosition, PhysicalSize, Position, Size, WindowEvent};

/// Tracks the current monitor state for cross-monitor resize.
#[derive(Debug)]
struct MonitorState {
    /// The current monitor's name (for identity comparison).
    last_monitor_name: Mutex<Option<String>>,
    /// Flag: was a monitor change detected during drag?
    /// If true, we should center after the drag settles.
    pending_center: Mutex<bool>,
    /// Target window size from the last resize (for accurate centering).
    pending_center_size: Mutex<Option<PhysicalSize<u32>>>,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            last_monitor_name: Mutex::new(None),
            pending_center: Mutex::new(false),
            pending_center_size: Mutex::new(None),
        }
    }
}

/// Get the monitor name as a String (or empty string if unavailable).
fn monitor_name(monitor: &tauri::Monitor) -> String {
    monitor.name().map(|n| n.to_string()).unwrap_or_default()
}

/// Compute the target window size (85% of monitor) and apply it.
///
/// Returns the target PhysicalSize so callers can use it for centering
/// without relying on the (possibly stale) outer_size() after set_size.
fn resize_to_monitor(window: &tauri::WebviewWindow, monitor: &tauri::Monitor) -> PhysicalSize<u32> {
    let physical = monitor.size();
    let target_w = (physical.width as f64 * 0.85).round() as u32;
    let target_h = (physical.height as f64 * 0.85).round() as u32;

    let target = PhysicalSize::<u32>::new(target_w, target_h);
    let _ = window.set_size(Size::Physical(target));
    target
}

/// Center the window on the given monitor using **physical** coordinates.
///
/// We do NOT use `window.center()` because for transparent borderless
/// windows on macOS the NSWindow frame isn't fully initialized at that
/// point, causing incorrect centering (especially with multiple monitors
/// arranged in non-standard layouts).
///
/// The `window_size` parameter is the **target** size (from resize_to_monitor),
/// NOT `outer_size()`, because `set_size` is asynchronous and `outer_size()`
/// may return a stale value immediately after `set_size`.
fn center_window_on_monitor(
    window: &tauri::WebviewWindow,
    monitor: &tauri::Monitor,
    window_size: PhysicalSize<u32>,
) {
    let monitor_size = monitor.size();   // PhysicalSize<u32>
    let monitor_pos = monitor.position(); // PhysicalPosition<i32>

    let center_x = monitor_pos.x
        + ((monitor_size.width as i32 - window_size.width as i32) / 2);
    let center_y = monitor_pos.y
        + ((monitor_size.height as i32 - window_size.height as i32) / 2);

    let _ = window.set_position(Position::Physical(PhysicalPosition::<i32>::new(center_x, center_y)));
}

/// Check if the window is on a different monitor than before.
/// Returns Some(new_monitor) if the monitor changed, None otherwise.
fn check_monitor_changed(
    window: &tauri::WebviewWindow,
    monitor_state: &MonitorState,
) -> Option<tauri::Monitor> {
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
            *last_name = Some(current_name);
            Some(current_monitor)
        }
        _ => None,
    }
}

/// Handle a monitor change during drag: **resize only**, do NOT center.
/// Sets a flag so the centering logic can center after the drag settles.
fn handle_monitor_change_during_drag(
    window: &tauri::WebviewWindow,
    monitor: &tauri::Monitor,
    monitor_state: &MonitorState,
) {
    // Resize to the new monitor's size (user sees the window grow/shrink)
    let target_size = resize_to_monitor(window, monitor);

    // Store the target size for later centering
    {
        let mut size = monitor_state.pending_center_size.lock().expect("lock poisoned");
        *size = Some(target_size);
    }

    // Mark that we need to center after drag settles
    let mut pending = monitor_state.pending_center.lock().expect("lock poisoned");
    *pending = true;
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

            // ── Window positioning strategy ──────────────────────────────
            // 1. Get the primary monitor for sizing (reliable at startup)
            // 2. Resize to 85% of that monitor's physical size
            // 3. Center using physical coordinates
            //
            // This avoids the bug where current_monitor() returns None or
            // the wrong monitor when the window is at (0,0) on startup.
            // ─────────────────────────────────────────────────────────────

            let primary = app
                .primary_monitor()
                .expect("failed to get primary monitor")
                .expect("no primary monitor");

            // Resize to 85% of primary monitor, get target size for centering
            let target_size = resize_to_monitor(&window, &primary);

            // Center on the primary monitor using physical coordinates
            center_window_on_monitor(&window, &primary, target_size);

            // Record the current monitor
            {
                let monitor_state = app.state::<MonitorState>();
                let mut name = monitor_state.last_monitor_name.lock().expect("lock poisoned");
                *name = Some(monitor_name(&primary));
            }

            // ── Cross-monitor resize via polling ─────────────────────────
            // Transparent borderless windows on macOS do NOT reliably fire
            // WindowEvent::Moved when dragged between monitors. Instead we
            // poll current_monitor() every 500ms.
            //
            // When a monitor change is detected:
            //   - During drag: resize to new monitor (NO centering)
            //   - After drag settles: center on new monitor
            //
            // We detect "drag settled" by observing that the window stops
            // moving for 800ms, then checking if pending_center is true.
            // ─────────────────────────────────────────────────────────────

            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // Wait a bit before starting to poll (let the window settle)
                std::thread::sleep(std::time::Duration::from_secs(2));

                // Track the last known window position to detect when dragging stops
                let mut last_position: Option<(i32, i32)> = None;

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(100));

                    let window = match app_handle.get_webview_window("main") {
                        Some(w) => w,
                        None => break, // window closed
                    };

                    let monitor_state = app_handle.state::<MonitorState>();

                    // Get current window position
                    let current_pos = match window.outer_position() {
                        Ok(pos) => (pos.x, pos.y),
                        Err(_) => continue,
                    };

                    // Check if window is still moving (dragging)
                    let is_moving = match last_position {
                        Some(prev) => prev != current_pos,
                        None => false,
                    };
                    last_position = Some(current_pos);

                    // ── Step 1: Check for monitor change (resize only) ──
                    if let Some(new_monitor) = check_monitor_changed(&window, &monitor_state) {
                        handle_monitor_change_during_drag(&window, &new_monitor, &monitor_state);
                    }

                    // ── Step 2: If drag settled and center pending, center ──
                    if !is_moving {
                        let mut pending = monitor_state.pending_center.lock().expect("lock poisoned");
                        if *pending {
                            *pending = false;

                            // Grab the stored target size
                            let mut stored_size = monitor_state.pending_center_size.lock().expect("lock poisoned");
                            let target_size = stored_size.take();

                            // Center on the current monitor immediately
                            if let Ok(Some(monitor)) = window.current_monitor() {
                                if let Some(size) = target_size {
                                    let _ = app_handle.run_on_main_thread(move || {
                                        center_window_on_monitor(&window, &monitor, size);
                                    });
                                }
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

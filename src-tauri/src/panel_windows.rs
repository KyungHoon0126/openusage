use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Manager, Position, Size};

static LAST_SHOW_TIME: OnceLock<Mutex<std::time::Instant>> = OnceLock::new();

fn touch_last_show() {
    if let Some(lock) = LAST_SHOW_TIME.get() {
        if let Ok(mut ts) = lock.lock() {
            *ts = std::time::Instant::now();
        }
    }
}

/// Macro to get the main window, similar to macOS get_or_init_panel.
macro_rules! get_or_init_panel {
    ($app_handle:expr) => {
        match $app_handle.get_webview_window("main") {
            Some(window) => Some(window),
            None => {
                log::error!("Main window not found");
                None
            }
        }
    };
}

pub(crate) use get_or_init_panel;

/// Show the panel window.
pub fn show_panel(app_handle: &AppHandle) {
    if let Some(window) = get_or_init_panel!(app_handle) {
        touch_last_show();
        let _ = window.set_always_on_top(true);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Toggle panel visibility.
pub fn toggle_panel(app_handle: &AppHandle) {
    let Some(window) = get_or_init_panel!(app_handle) else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        log::debug!("toggle_panel: hiding panel");
        let _ = window.hide();
    } else {
        log::debug!("toggle_panel: showing panel");
        touch_last_show();
        let _ = window.set_always_on_top(true);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Initialize the panel window with Windows-appropriate settings.
pub fn init(app_handle: &AppHandle) -> tauri::Result<()> {
    let Some(window) = app_handle.get_webview_window("main") else {
        log::error!("Main window not found during init");
        return Ok(());
    };

    // Set up blur event to hide the panel when it loses focus.
    // Use a shared timestamp to prevent hide-after-show race with tray click.
    LAST_SHOW_TIME.get_or_init(|| Mutex::new(std::time::Instant::now()));

    let handle = app_handle.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let handle_inner = handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                // Skip hide if a show was requested recently (tray click race)
                if let Some(lock) = LAST_SHOW_TIME.get() {
                    if let Ok(ts) = lock.lock() {
                        if ts.elapsed() < std::time::Duration::from_millis(300) {
                            return;
                        }
                    }
                }
                if let Some(w) = handle_inner.get_webview_window("main") {
                    if !w.is_focused().unwrap_or(true) {
                        let _ = w.set_always_on_top(false);
                        let _ = w.hide();
                    }
                }
            });
        }
    });

    Ok(())
}

/// Position the panel near the tray icon on Windows.
pub fn position_panel_at_tray_icon(
    app_handle: &AppHandle,
    icon_position: Position,
    icon_size: Size,
) {
    let Some(window) = app_handle.get_webview_window("main") else {
        return;
    };

    let (icon_x, icon_y) = match &icon_position {
        Position::Physical(pos) => (pos.x as f64, pos.y as f64),
        Position::Logical(pos) => (pos.x, pos.y),
    };
    let (icon_w, _icon_h) = match &icon_size {
        Size::Physical(s) => (s.width as f64, s.height as f64),
        Size::Logical(s) => (s.width, s.height),
    };

    let scale = window.scale_factor().unwrap_or(1.0);

    let (panel_width, panel_height) = match window.outer_size() {
        Ok(s) => (s.width as f64 / scale, s.height as f64 / scale),
        _ => (400.0, 500.0),
    };

    // On Windows, the tray is typically at bottom-right.
    // Position panel above the tray icon, centered horizontally.
    let icon_logical_x = icon_x / scale;
    let icon_logical_y = icon_y / scale;
    let icon_logical_w = icon_w / scale;

    let icon_center_x = icon_logical_x + (icon_logical_w / 2.0);
    let panel_x = icon_center_x - (panel_width / 2.0);
    // Position above the tray icon with a small gap
    let panel_y = icon_logical_y - panel_height - 8.0;

    let _ = window.set_position(tauri::LogicalPosition::new(panel_x, panel_y));
}

/// Hide the panel.
#[allow(dead_code)]
pub fn hide_panel(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// Check if the panel is visible.
#[allow(dead_code)]
pub fn is_visible(app_handle: &AppHandle) -> bool {
    app_handle
        .get_webview_window("main")
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false)
}

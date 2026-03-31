// Platform-specific panel implementations
#[cfg(target_os = "macos")]
#[path = "panel_macos.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "panel_windows.rs"]
mod platform;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "panel_windows.rs"]
mod platform; // Linux fallback uses same window-based approach

pub(crate) use platform::get_or_init_panel;
pub(crate) use platform::init;
pub(crate) use platform::position_panel_at_tray_icon;
pub(crate) use platform::show_panel;
pub(crate) use platform::toggle_panel;

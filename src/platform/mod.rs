#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "macos")]
pub const HOTKEY_LABEL: &str = "Right Option";
#[cfg(target_os = "windows")]
pub const HOTKEY_LABEL: &str = "Right Alt";

#[cfg(target_os = "macos")]
pub const PASTE_SHORTCUT_LABEL: &str = "Cmd+V";
#[cfg(target_os = "windows")]
pub const PASTE_SHORTCUT_LABEL: &str = "Ctrl+V";

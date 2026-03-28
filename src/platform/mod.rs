#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

pub enum PasteModifier {
    Super, // Cmd on macOS
    Control, // Ctrl on Windows
}

pub trait PlatformProvider {
    fn paste_modifier() -> PasteModifier;
}

use enigo::Key;
use std::process::Command;

unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn paste_modifier() -> Key {
    Key::Meta
}

pub fn play_start_sound() {
    let _ = Command::new("afplay")
        .arg("/System/Library/Sounds/Tink.aiff")
        .spawn();
}

pub fn play_stop_sound() {
    let _ = Command::new("afplay")
        .arg("/System/Library/Sounds/Pop.aiff")
        .spawn();
}

pub fn check_microphone() -> bool {
    use cpal::traits::{DeviceTrait, HostTrait};
    let Some(device) = cpal::default_host().default_input_device() else {
        return false;
    };
    let Ok(config) = device.default_input_config() else {
        return false;
    };
    device
        .build_input_stream(
            &config.into(),
            |_: &[f32], _: &cpal::InputCallbackInfo| {},
            |_| {},
            None,
        )
        .is_ok()
}

pub fn check_accessibility() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn prompt_microphone() {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    if let Some(device) = host.default_input_device()
        && let Ok(config) = device.default_input_config()
    {
        let _ = device.build_input_stream(
            &config.into(),
            |_: &[f32], _: &cpal::InputCallbackInfo| {},
            |_| {},
            None,
        );
    }
}

pub fn prompt_accessibility() {
    eprintln!("[murmur] accessibility permission required — opening System Settings...");
    let _ = Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

pub fn prompt_input_monitoring() {
    eprintln!("[murmur] input monitoring permission required — opening System Settings...");
    let _ = Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
        .spawn();
}

#[cfg(feature = "fluid_audio")]
pub fn is_apple_silicon() -> bool {
    std::env::consts::ARCH == "aarch64"
}

pub fn self_restart() -> ! {
    use std::os::unix::process::CommandExt;
    let exe = std::env::current_exe().expect("failed to get current exe path");
    eprintln!("[murmur] restarting to apply permissions...");
    let err = Command::new(exe).exec();
    panic!("failed to restart: {err}");
}

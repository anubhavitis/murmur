use enigo::Key;
use std::process::Command;

pub fn paste_modifier() -> Key {
    Key::Control
}

pub fn play_start_sound() {
    let _ = Command::new("powershell")
        .args(["-Command", "[System.Media.SystemSounds]::Exclamation.Play()"])
        .spawn();
}

pub fn play_stop_sound() {
    let _ = Command::new("powershell")
        .args(["-Command", "[System.Media.SystemSounds]::Asterisk.Play()"])
        .spawn();
}

pub fn check_accessibility() -> bool {
    true
}

pub fn prompt_accessibility() {}
pub fn prompt_input_monitoring() {}
pub fn self_restart() -> ! {
    let exe = std::env::current_exe().expect("failed to get current exe path");
    let _ = Command::new(exe).spawn();
    std::process::exit(0);
}

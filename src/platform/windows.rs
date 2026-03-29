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

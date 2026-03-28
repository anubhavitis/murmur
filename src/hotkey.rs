use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tao::event_loop::EventLoopProxy;

use crate::app::AppEvent;
use crate::config::HotkeyChoice;

fn target_key(choice: &HotkeyChoice) -> Key {
    match choice {
        HotkeyChoice::RightAlt => Key::AltGr,
        HotkeyChoice::CapsLock => Key::CapsLock,
    }
}

pub fn spawn_listener(
    proxy: EventLoopProxy<AppEvent>,
    hotkey_choice: HotkeyChoice,
    is_pressed: Arc<AtomicBool>,
) {
    let key = target_key(&hotkey_choice);

    thread::spawn(move || {
        listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(k) if k == key => {
                    if !is_pressed.load(Ordering::SeqCst) {
                        is_pressed.store(true, Ordering::SeqCst);
                        let _ = proxy.send_event(AppEvent::HotkeyPressed);
                    }
                }
                EventType::KeyRelease(k) if k == key => {
                    if is_pressed.load(Ordering::SeqCst) {
                        is_pressed.store(false, Ordering::SeqCst);
                        let _ = proxy.send_event(AppEvent::HotkeyReleased);
                    }
                }
                _ => {}
            }
        })
        .expect("failed to listen for global key events");
    });
}

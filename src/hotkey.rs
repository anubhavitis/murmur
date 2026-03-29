use rdev::{Event, EventType, Key, listen};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tao::event_loop::EventLoopProxy;

use crate::app::AppEvent;
use crate::config::HotkeyChoice;

const STALE_THRESHOLD_MS: u64 = 500;

fn target_key(choice: &HotkeyChoice) -> Key {
    match choice {
        HotkeyChoice::RightAlt => Key::AltGr,
        HotkeyChoice::CapsLock => Key::CapsLock,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn spawn_listener(proxy: EventLoopProxy<AppEvent>, hotkey_choice: HotkeyChoice) {
    let key = target_key(&hotkey_choice);
    let press_time = Arc::new(AtomicU64::new(0));

    thread::spawn(move || {
        let result = listen(move |event: Event| match event.event_type {
            EventType::KeyPress(k) if k == key => {
                let last = press_time.load(Ordering::SeqCst);
                let now = now_ms();
                let stale = last > 0 && (now - last) > STALE_THRESHOLD_MS;

                if last == 0 || stale {
                    press_time.store(now, Ordering::SeqCst);
                    let _ = proxy.send_event(AppEvent::HotkeyPressed);
                }
            }
            EventType::KeyRelease(k) if k == key => {
                if press_time.load(Ordering::SeqCst) > 0 {
                    press_time.store(0, Ordering::SeqCst);
                    let _ = proxy.send_event(AppEvent::HotkeyReleased);
                }
            }
            _ => {}
        });
        if let Err(e) = result {
            eprintln!("[murmur] hotkey listener failed: {e:?}");
            let retries: u32 = std::env::var("MURMUR_HOTKEY_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            if retries >= 3 {
                eprintln!("[murmur] hotkey listener failed after 3 attempts, giving up");
                eprintln!("[murmur] grant Input Monitoring in System Settings and restart");
                return;
            }
            crate::platform::prompt_input_monitoring();
            eprintln!(
                "[murmur] waiting for input monitoring (attempt {})...",
                retries + 1
            );
            std::thread::sleep(std::time::Duration::from_secs(10));
            // SAFETY: only this thread runs at this point, self_restart replaces process immediately
            unsafe {
                std::env::set_var("MURMUR_HOTKEY_RETRIES", (retries + 1).to_string());
            }
            crate::platform::self_restart();
        }
    });
}

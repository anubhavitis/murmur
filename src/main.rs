mod app;
mod audio;
mod config;
mod downloader;
mod hotkey;
mod languages;
mod platform;
mod transcriber;
mod tray;

use std::time::{Duration, Instant};

use app::{AppEvent, AppState, MenuCommand, RecordingState};
use arboard::Clipboard;
use audio::AudioCapture;
use config::{Config, OutputMode};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use transcriber::Transcriber;
use tray::Tray;
use tray_icon::menu::MenuEvent;

const ANIMATION_INTERVAL: Duration = Duration::from_millis(33);
const PERMISSION_CHECK_INTERVAL: Duration = Duration::from_secs(2);

fn rotate_log() {
    let log_path = Config::base_dir().join("murmur.log");
    let prev_path = Config::base_dir().join("murmur.log.1");
    let _ = std::fs::rename(&log_path, &prev_path);
    let _ = std::fs::File::create(&log_path);
}

fn check_permissions(state: &mut AppState) -> bool {
    let mut changed = false;

    let mic = platform::check_microphone();
    if mic != state.permissions.microphone {
        state.permissions.microphone = mic;
        changed = true;
        if mic {
            eprintln!("[murmur] microphone permission granted");
        }
    }

    let acc = platform::check_accessibility();
    if acc != state.permissions.accessibility {
        state.permissions.accessibility = acc;
        changed = true;
        if acc {
            eprintln!("[murmur] accessibility permission granted");
        }
    }

    // Prompt sequentially: mic first, then accessibility
    if !state.permissions.microphone && !state.permissions.prompted_mic {
        platform::prompt_microphone();
        state.permissions.prompted_mic = true;
    } else if state.permissions.microphone && !state.permissions.accessibility && !state.permissions.prompted_accessibility {
        platform::prompt_accessibility();
        state.permissions.prompted_accessibility = true;
    }

    changed
}

fn all_permissions_granted(state: &AppState) -> bool {
    state.permissions.microphone && state.permissions.accessibility
}

const BOOTSTRAP_MODEL: &str = "tiny.en";

fn main() {
    rotate_log();
    eprintln!("[murmur] starting...");

    let config = Config::load();
    let mut state = AppState::new(config);
    let mut audio = AudioCapture::new();

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Always bootstrap with tiny.en for instant startup
    let mut transcriber = Transcriber::new(proxy.clone(), BOOTSTRAP_MODEL);

    let download_proxy = proxy.clone();

    hotkey::spawn_listener(proxy, state.config.hotkey.clone());

    check_permissions(&mut state);

    let menu_channel = MenuEvent::receiver();
    let mut tray = Tray::new(&state);
    let mut clipboard = Clipboard::new().expect("failed to init clipboard");
    let mut last_perm_check = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        if let Event::NewEvents(StartCause::ResumeTimeReached { .. }) = &event {
            if state.recording_state != RecordingState::Idle {
                tray.advance_frame();
            }

            if !all_permissions_granted(&state)
                && last_perm_check.elapsed() >= PERMISSION_CHECK_INTERVAL
            {
                last_perm_check = Instant::now();
                if check_permissions(&mut state) {
                    tray.rebuild(&state);
                }
            }
        }

        if let Ok(menu_event) = menu_channel.try_recv()
            && let Some(app_event) = tray.handle_menu_event(&menu_event, &state)
        {
            handle_event(
                app_event,
                &mut state,
                &mut tray,
                &mut audio,
                &mut transcriber,
                &mut clipboard,
                &download_proxy,
            );
        }

        if let Event::UserEvent(app_event) = event {
            handle_event(
                app_event,
                &mut state,
                &mut tray,
                &mut audio,
                &mut transcriber,
                &mut clipboard,
                &download_proxy,
            );
        }

        *control_flow = if state.recording_state != RecordingState::Idle
            || !all_permissions_granted(&state)
        {
            ControlFlow::WaitUntil(Instant::now() + ANIMATION_INTERVAL)
        } else {
            ControlFlow::Wait
        };
    });
}

fn handle_event(
    event: AppEvent,
    state: &mut AppState,
    tray: &mut Tray,
    audio: &mut AudioCapture,
    transcriber: &mut Transcriber,
    clipboard: &mut Clipboard,
    download_proxy: &tao::event_loop::EventLoopProxy<AppEvent>,
) {
    match event {
        AppEvent::HotkeyPressed => {
            if state.recording_state != RecordingState::Idle {
                eprintln!(
                    "[murmur] hotkey pressed but busy ({:?})",
                    state.recording_state
                );
                return;
            }
            if !state.transcriber_ready {
                eprintln!("[murmur] model still loading...");
                return;
            }
            if !state.permissions.microphone {
                platform::play_stop_sound();
                eprintln!("[murmur] microphone permission not granted");
                return;
            }
            match audio.start() {
                Ok(()) => {
                    platform::play_start_sound();
                    eprintln!("[murmur] recording...");
                    state.recording_state = RecordingState::Recording;
                    tray.rebuild(state);
                }
                Err(e) => {
                    platform::play_stop_sound();
                    eprintln!("[murmur] error: failed to start recording: {e}");
                }
            }
        }
        AppEvent::HotkeyReleased => {
            if state.recording_state != RecordingState::Recording {
                return;
            }
            platform::play_stop_sound();
            let samples = audio.stop();
            eprintln!("[murmur] transcribing...");
            state.recording_state = RecordingState::Transcribing;
            tray.rebuild(state);
            transcriber.transcribe(samples, state.config.languages.clone());
        }
        AppEvent::TranscriptionComplete(text) => {
            eprintln!("[murmur] \"{text}\"");
            if let Err(e) = clipboard.set_text(&text) {
                eprintln!("[murmur] error: clipboard: {e}");
            } else if state.config.output_mode == OutputMode::PasteAtCursor {
                if state.permissions.accessibility {
                    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        let modifier = platform::paste_modifier();
                        let _ = enigo.key(modifier, Direction::Press);
                        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                        let _ = enigo.key(modifier, Direction::Release);
                    }
                } else {
                    eprintln!("[murmur] paste skipped: accessibility not granted (text is in clipboard)");
                }
            }
            state.recording_state = RecordingState::Idle;
            tray.rebuild(state);
        }
        AppEvent::TranscriberReady => {
            eprintln!("[murmur] model loaded, ready");
            state.transcriber_ready = true;
            tray.rebuild(state);

            // Check if we need to upgrade from bootstrap model
            let target = state.config.selected_tier.whisper_model();
            if target != BOOTSTRAP_MODEL && !state.upgrading_backend {
                if downloader::model_path(target).exists() {
                    let _ = download_proxy.send_event(AppEvent::BackendUpgradeReady);
                } else {
                    eprintln!("[murmur] background download: {target}");
                    state.upgrading_backend = true;
                    downloader::spawn_upgrade(download_proxy.clone(), target.to_string());
                }
            }
        }
        AppEvent::TranscriptionError(err) => {
            eprintln!("[murmur] error: {err}");
            state.recording_state = RecordingState::Idle;
            tray.reset_icon();
            tray.rebuild(state);
        }
        AppEvent::ModelDownloadProgress(model, pct) => {
            let first = state.download_progress.is_none();
            state.download_progress = Some((model.clone(), pct));
            if first {
                tray.rebuild(state);
            } else {
                tray.update_progress(pct, state.config.selected_tier.display_name());
            }
        }
        AppEvent::ModelDownloadComplete(_model) => {
            state.download_progress = None;
            if state.pending_restart {
                platform::self_restart();
            }
            tray.rebuild(state);
        }
        AppEvent::BackendUpgradeReady => {
            let target = state.config.selected_tier.whisper_model();
            eprintln!("[murmur] upgrading to {target}");
            state.transcriber_ready = false;
            state.upgrading_backend = false;
            state.download_progress = None;
            tray.rebuild(state);
            *transcriber = Transcriber::new(download_proxy.clone(), target);
        }
        AppEvent::Menu(cmd) => handle_menu_command(cmd, state, tray, download_proxy),
        AppEvent::Quit => {
            std::process::exit(0);
        }
    }
}

fn handle_menu_command(
    cmd: MenuCommand,
    state: &mut AppState,
    tray: &mut Tray,
    proxy: &tao::event_loop::EventLoopProxy<AppEvent>,
) {
    match cmd {
        MenuCommand::SetOutputMode(mode) => {
            state.config.output_mode = mode;
            state.config.save();
            tray.rebuild(state);
        }
        MenuCommand::SetHotkey(hotkey) => {
            state.config.hotkey = hotkey;
            state.config.save();
            platform::self_restart();
        }
        MenuCommand::SetTier(tier) => {
            if state.download_progress.is_some() {
                return;
            }
            state.config.selected_tier = tier;
            state.config.save();
            let model = state.config.selected_tier.whisper_model();
            if downloader::model_path(model).exists() {
                platform::self_restart();
            } else {
                state.pending_restart = true;
                state.download_progress = Some((model.to_string(), 0));
                tray.rebuild(state);
                downloader::spawn_download(proxy.clone(), model.to_string());
            }
        }
        MenuCommand::ToggleLanguage(code) => {
            if code == "en" {
                return;
            }
            if let Some(pos) = state.config.languages.iter().position(|l| l == &code) {
                state.config.languages.remove(pos);
            } else {
                state.config.languages.push(code);
            }
            if state.config.languages.is_empty() {
                state.config.languages.push("en".to_string());
            }
            state.config.save();
            tray.rebuild(state);
        }
    }
}

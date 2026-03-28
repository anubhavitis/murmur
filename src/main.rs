mod app;
mod audio;
mod config;
mod downloader;
mod hotkey;
mod platform;
mod transcriber;
mod tray;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
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

fn main() {
    let config = Config::load();
    let mut state = AppState::new(config);
    let mut audio = AudioCapture::new();

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let transcriber = match Transcriber::new(proxy.clone(), &state.config.selected_model) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[murmur] failed to init transcriber: {e}");
            std::process::exit(1);
        }
    };

    let download_proxy = proxy.clone();

    let hotkey_pressed = Arc::new(AtomicBool::new(false));
    hotkey::spawn_listener(proxy, state.config.hotkey.clone(), hotkey_pressed);

    let menu_channel = MenuEvent::receiver();
    let mut tray = Tray::new(&state);
    let mut clipboard = Clipboard::new().expect("failed to init clipboard");

    event_loop.run(move |event, _, control_flow| {
        if let Event::NewEvents(StartCause::ResumeTimeReached { .. }) = &event {
            if state.recording_state == RecordingState::Recording {
                tray.advance_frame();
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
                &transcriber,
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
                &transcriber,
                &mut clipboard,
                &download_proxy,
            );
        }

        *control_flow = if state.recording_state == RecordingState::Recording {
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
    transcriber: &Transcriber,
    clipboard: &mut Clipboard,
    download_proxy: &tao::event_loop::EventLoopProxy<AppEvent>,
) {
    match event {
        AppEvent::HotkeyPressed => {
            if state.recording_state != RecordingState::Idle {
                return;
            }
            match audio.start() {
                Ok(()) => {
                    eprintln!("[murmur] recording...");
                    state.recording_state = RecordingState::Recording;
                    tray.rebuild(state);
                }
                Err(e) => eprintln!("[murmur] error: failed to start recording: {e}"),
            }
        }
        AppEvent::HotkeyReleased => {
            if state.recording_state != RecordingState::Recording {
                return;
            }
            let samples = audio.stop();
            eprintln!("[murmur] transcribing...");
            state.recording_state = RecordingState::Transcribing;
            tray.reset_icon();
            tray.rebuild(state);
            transcriber.transcribe(samples);
        }
        AppEvent::TranscriptionComplete(text) => {
            eprintln!("[murmur] \"{text}\"");
            if let Err(e) = clipboard.set_text(&text) {
                eprintln!("[murmur] error: clipboard: {e}");
            } else if state.config.output_mode == OutputMode::PasteAtCursor
                && let Ok(mut enigo) = Enigo::new(&Settings::default())
            {
                std::thread::sleep(std::time::Duration::from_millis(50));
                let modifier = platform::paste_modifier();
                let _ = enigo.key(modifier, Direction::Press);
                let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                let _ = enigo.key(modifier, Direction::Release);
            }
            state.recording_state = RecordingState::Idle;
            tray.rebuild(state);
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
                tray.update_progress(&model, pct);
            }
        }
        AppEvent::ModelDownloadComplete(model) => {
            state.download_progress = None;
            state.refresh_models();
            state.config.selected_model = model;
            state.config.save();
            tray.rebuild(state);
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
            tray.rebuild(state);
        }
        MenuCommand::SelectModel(model) => {
            state.config.selected_model = model;
            state.config.save();
            tray.rebuild(state);
        }
        MenuCommand::DownloadModel(model) => {
            if state.download_progress.is_some() {
                return;
            }
            downloader::spawn_download(proxy.clone(), model);
        }
        MenuCommand::Quit => {}
    }
}

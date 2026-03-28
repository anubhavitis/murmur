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

use app::{AppEvent, AppState, MenuCommand, RecordingState};
use arboard::Clipboard;
use audio::AudioCapture;
use config::{Config, OutputMode};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tao::event::Event;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use transcriber::Transcriber;
use tray::Tray;
use tray_icon::menu::MenuEvent;

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

    let hotkey_pressed = Arc::new(AtomicBool::new(false));
    hotkey::spawn_listener(proxy, state.config.hotkey.clone(), hotkey_pressed);

    let menu_channel = MenuEvent::receiver();
    let mut tray = Tray::new(&state);
    let mut clipboard = Clipboard::new().expect("failed to init clipboard");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

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
                control_flow,
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
                control_flow,
            );
        }
    });
}

fn handle_event(
    event: AppEvent,
    state: &mut AppState,
    tray: &mut Tray,
    audio: &mut AudioCapture,
    transcriber: &Transcriber,
    clipboard: &mut Clipboard,
    control_flow: &mut ControlFlow,
) {
    match event {
        AppEvent::HotkeyPressed => {
            if state.recording_state != RecordingState::Idle {
                return;
            }
            match audio.start() {
                Ok(()) => {
                    eprintln!("[murmur] recording started");
                    state.recording_state = RecordingState::Recording;
                    tray.rebuild(state);
                }
                Err(e) => eprintln!("[murmur] failed to start recording: {e}"),
            }
        }
        AppEvent::HotkeyReleased => {
            if state.recording_state != RecordingState::Recording {
                return;
            }
            let samples = audio.stop();
            let duration_secs = samples.len() as f32 / 16_000.0;
            eprintln!(
                "[murmur] recording stopped: {} samples ({:.1}s), transcribing...",
                samples.len(),
                duration_secs
            );
            state.recording_state = RecordingState::Transcribing;
            tray.rebuild(state);
            transcriber.transcribe(samples);
        }
        AppEvent::TranscriptionComplete(text) => {
            eprintln!("[murmur] transcription: {text}");
            if let Err(e) = clipboard.set_text(&text) {
                eprintln!("[murmur] clipboard error: {e}");
            } else {
                eprintln!("[murmur] copied to clipboard");
                if state.config.output_mode == OutputMode::PasteAtCursor
                    && let Ok(mut enigo) = Enigo::new(&Settings::default())
                {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let _ = enigo.key(Key::Meta, Direction::Press);
                    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                    let _ = enigo.key(Key::Meta, Direction::Release);
                    eprintln!("[murmur] pasted at cursor");
                }
            }
            state.recording_state = RecordingState::Idle;
            tray.rebuild(state);
        }
        AppEvent::TranscriptionError(err) => {
            eprintln!("[murmur] transcription error: {err}");
            state.recording_state = RecordingState::Idle;
            tray.rebuild(state);
        }
        AppEvent::Menu(cmd) => handle_menu_command(cmd, state, tray),
        AppEvent::Quit => {
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    }
}

fn handle_menu_command(cmd: MenuCommand, state: &mut AppState, tray: &mut Tray) {
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
        MenuCommand::DownloadModel(_model) => {
            // phase 5: model download via menu
        }
        MenuCommand::Quit => {}
    }
}

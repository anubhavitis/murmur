use crate::config::{Config, HotkeyChoice, OutputMode, Tier};

#[derive(Debug, Clone)]
pub enum MenuCommand {
    SetTier(Tier),
    SetOutputMode(OutputMode),
    SetHotkey(HotkeyChoice),
    ToggleLanguage(String),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    HotkeyPressed,
    HotkeyReleased,
    TranscriptionComplete(String),
    TranscriptionError(String),
    TranscriberReady,
    ModelDownloadProgress(String, u8),
    ModelDownloadComplete(String),
    BackendUpgradeReady,
    Menu(MenuCommand),
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
}

#[derive(Debug, Default)]
pub struct Permissions {
    pub microphone: bool,
    pub accessibility: bool,
    pub prompted_mic: bool,
    pub prompted_accessibility: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub recording_state: RecordingState,
    pub download_progress: Option<(String, u8)>,
    pub pending_restart: bool,
    pub upgrading_backend: bool,
    pub permissions: Permissions,
    pub transcriber_ready: bool,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            recording_state: RecordingState::Idle,
            download_progress: None,
            pending_restart: false,
            upgrading_backend: false,
            permissions: Permissions::default(),
            transcriber_ready: false,
        }
    }
}

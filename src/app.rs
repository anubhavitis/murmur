use crate::config::{Config, HotkeyChoice, OutputMode};

#[derive(Debug, Clone)]
pub enum MenuCommand {
    SelectModel(String),
    DownloadModel(String),
    SetOutputMode(OutputMode),
    SetHotkey(HotkeyChoice),
    Quit,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    HotkeyPressed,
    HotkeyReleased,
    TranscriptionComplete(String),
    TranscriptionError(String),
    ModelDownloadProgress(String, u8),
    ModelDownloadComplete(String),
    Menu(MenuCommand),
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Transcribing,
}

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub recording_state: RecordingState,
    pub download_progress: Option<(String, u8)>,
    pub installed_models: Vec<String>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let installed_models = Self::scan_installed_models();
        Self {
            config,
            recording_state: RecordingState::Idle,
            download_progress: None,
            installed_models,
        }
    }

    fn scan_installed_models() -> Vec<String> {
        let models_dir = Config::models_dir();
        let mut models = Vec::new();
        if let Ok(entries) = std::fs::read_dir(models_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(model) = name
                    .strip_prefix("ggml-")
                    .and_then(|s| s.strip_suffix(".bin"))
                {
                    models.push(model.to_string());
                }
            }
        }
        models
    }

    pub fn refresh_models(&mut self) {
        self.installed_models = Self::scan_installed_models();
    }
}

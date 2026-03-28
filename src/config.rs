use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    Clipboard,
    PasteAtCursor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyChoice {
    RightAlt,
    CapsLock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub selected_model: String,
    pub output_mode: OutputMode,
    pub hotkey: HotkeyChoice,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            selected_model: "tiny.en".to_string(),
            output_mode: OutputMode::Clipboard,
            hotkey: HotkeyChoice::RightAlt,
        }
    }
}

impl Config {
    pub fn base_dir() -> PathBuf {
        dirs::home_dir()
            .expect("could not resolve home directory")
            .join(".murmur")
    }

    pub fn config_path() -> PathBuf {
        Self::base_dir().join("config.json")
    }

    pub fn models_dir() -> PathBuf {
        Self::base_dir().join("models")
    }

    pub fn ensure_dirs() {
        let base = Self::base_dir();
        fs::create_dir_all(&base).expect("failed to create ~/.murmur");
        fs::create_dir_all(Self::models_dir()).expect("failed to create ~/.murmur/models");
    }

    pub fn load() -> Self {
        Self::ensure_dirs();
        let path = Self::config_path();
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            let config = Self::default();
            config.save();
            config
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        let data = serde_json::to_string_pretty(self).expect("failed to serialize config");
        fs::write(path, data).expect("failed to write config");
    }
}

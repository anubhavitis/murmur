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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Fast,
    Standard,
    Accurate,
}

impl Tier {
    pub fn whisper_model(&self) -> &str {
        match self {
            Tier::Fast => "small",
            Tier::Standard => "medium",
            Tier::Accurate => "large-v3",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Tier::Fast => "Fast",
            Tier::Standard => "Standard",
            Tier::Accurate => "Accurate",
        }
    }

    pub fn download_size(&self) -> &str {
        match self {
            Tier::Fast => "465 MB",
            Tier::Standard => "1.4 GB",
            Tier::Accurate => "2.9 GB",
        }
    }

    pub fn label_for_model(model: &str) -> &'static str {
        match model {
            "small.en" | "small" => "Fast",
            "medium.en" | "medium" => "Standard",
            "large-v3" => "Accurate",
            _ => "Fast",
        }
    }

    fn from_legacy_model(model: &str) -> Self {
        match model {
            "tiny.en" | "tiny" | "base.en" | "base" | "small.en" | "small" => Tier::Fast,
            "medium.en" | "medium" => Tier::Standard,
            _ => Tier::Accurate,
        }
    }
}

fn default_tier() -> Tier {
    Tier::Fast
}

fn default_languages() -> Vec<String> {
    vec!["en".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_tier")]
    pub selected_tier: Tier,
    #[serde(default, skip_serializing)]
    selected_model: Option<String>,
    pub output_mode: OutputMode,
    pub hotkey: HotkeyChoice,
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            selected_tier: Tier::Fast,
            selected_model: None,
            output_mode: OutputMode::Clipboard,
            hotkey: HotkeyChoice::RightAlt,
            languages: default_languages(),
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
        let mut config = if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            let config = Self::default();
            config.save();
            config
        };
        if config.languages.is_empty() {
            config.languages.push("en".to_string());
        }
        if let Some(model) = config.selected_model.take() {
            config.selected_tier = Tier::from_legacy_model(&model);
            config.save();
        }
        config
    }

    pub fn save(&self) {
        let path = Self::config_path();
        let data = serde_json::to_string_pretty(self).expect("failed to serialize config");
        fs::write(path, data).expect("failed to write config");
    }
}

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::config::Config;

const HF_BASE: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

pub fn model_path(model: &str) -> PathBuf {
    Config::models_dir().join(format!("ggml-{model}.bin"))
}

pub fn model_exists(model: &str) -> bool {
    model_path(model).exists()
}

pub fn ensure_model(model: &str) -> Result<PathBuf, String> {
    let path = model_path(model);
    if path.exists() {
        return Ok(path);
    }

    eprintln!("[murmur] model '{model}' not found, downloading...");
    download_model(model, &path)?;
    eprintln!("[murmur] model '{model}' downloaded successfully");
    Ok(path)
}

fn download_model(model: &str, dest: &PathBuf) -> Result<(), String> {
    let url = format!("{HF_BASE}/ggml-{model}.bin");

    let response = reqwest::blocking::get(&url)
        .map_err(|e| format!("download request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("download failed: HTTP {}", response.status()));
    }

    let bytes = response.bytes().map_err(|e| format!("failed to read response: {e}"))?;

    let tmp_path = dest.with_extension("bin.tmp");
    let mut file = fs::File::create(&tmp_path)
        .map_err(|e| format!("failed to create temp file: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("failed to write model: {e}"))?;

    fs::rename(&tmp_path, dest)
        .map_err(|e| format!("failed to rename temp file: {e}"))?;

    Ok(())
}

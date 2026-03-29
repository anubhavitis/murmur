use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::thread;

use tao::event_loop::EventLoopProxy;

use crate::app::AppEvent;
use crate::config::Config;

const HF_BASE: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

pub fn model_path(model: &str) -> PathBuf {
    Config::models_dir().join(format!("ggml-{model}.bin"))
}

pub fn ensure_model(model: &str) -> Result<PathBuf, String> {
    let path = model_path(model);
    if path.exists() {
        return Ok(path);
    }

    download_model_blocking(model, &path)?;
    Ok(path)
}

pub fn spawn_download(proxy: EventLoopProxy<AppEvent>, model: String) {
    thread::spawn(move || {
        let path = model_path(&model);
        let _ = proxy.send_event(AppEvent::ModelDownloadProgress(model.clone(), 0));

        match download_model_streaming(&model, &path, &proxy) {
            Ok(()) => {
                let _ = proxy.send_event(AppEvent::ModelDownloadComplete(model));
            }
            Err(e) => {
                let _ = proxy.send_event(AppEvent::TranscriptionError(format!(
                    "download failed: {e}"
                )));
            }
        }
    });
}

pub fn spawn_upgrade(proxy: EventLoopProxy<AppEvent>, model: String) {
    thread::spawn(move || {
        let path = model_path(&model);
        let _ = proxy.send_event(AppEvent::ModelDownloadProgress(model.clone(), 0));

        match download_model_streaming(&model, &path, &proxy) {
            Ok(()) => {
                let _ = proxy.send_event(AppEvent::BackendUpgradeReady);
            }
            Err(e) => {
                eprintln!("[murmur] background upgrade failed: {e}");
            }
        }
    });
}

fn download_model_blocking(model: &str, dest: &PathBuf) -> Result<(), String> {
    let url = format!("{HF_BASE}/ggml-{model}.bin");

    let response =
        reqwest::blocking::get(&url).map_err(|e| format!("download request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("download failed: HTTP {}", response.status()));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("failed to read response: {e}"))?;

    let tmp_path = dest.with_extension("bin.tmp");
    let mut file =
        fs::File::create(&tmp_path).map_err(|e| format!("failed to create temp file: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("failed to write model: {e}"))?;

    let _ = fs::remove_file(dest);
    fs::rename(&tmp_path, dest).map_err(|e| format!("failed to rename temp file: {e}"))?;

    Ok(())
}

fn download_model_streaming(
    model: &str,
    dest: &PathBuf,
    proxy: &EventLoopProxy<AppEvent>,
) -> Result<(), String> {
    let url = format!("{HF_BASE}/ggml-{model}.bin");

    let response =
        reqwest::blocking::get(&url).map_err(|e| format!("download request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let total = response.content_length().unwrap_or(0);
    let mut reader = response;
    let tmp_path = dest.with_extension("bin.tmp");
    let mut file =
        fs::File::create(&tmp_path).map_err(|e| format!("failed to create temp file: {e}"))?;

    let mut downloaded: u64 = 0;
    let mut last_pct: u8 = 0;
    let mut buf = [0u8; 65536];

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("read error: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("write error: {e}"))?;
        downloaded += n as u64;

        if total > 0 {
            let pct = ((downloaded * 100) / total).min(99) as u8;
            if pct != last_pct {
                last_pct = pct;
                let _ = proxy.send_event(AppEvent::ModelDownloadProgress(model.to_string(), pct));
            }
        }
    }

    let _ = fs::remove_file(dest);
    fs::rename(&tmp_path, dest).map_err(|e| format!("failed to rename temp file: {e}"))?;

    Ok(())
}

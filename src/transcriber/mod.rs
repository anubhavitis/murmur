mod whisper_backend;

use std::sync::mpsc;
use std::thread;

use tao::event_loop::EventLoopProxy;

use crate::app::AppEvent;
use crate::downloader;

pub trait TranscriptionBackend: Send {
    fn transcribe(&mut self, samples: &[f32], languages: &[String]) -> Result<String, String>;
}

fn suppress_whisper_logs() {
    unsafe {
        whisper_rs_sys::whisper_log_set(None, std::ptr::null_mut());
    }
}

struct TranscribeRequest {
    samples: Vec<f32>,
    languages: Vec<String>,
}

pub struct Transcriber {
    sender: mpsc::Sender<TranscribeRequest>,
    proxy: EventLoopProxy<AppEvent>,
}

impl Transcriber {
    pub fn new(proxy: EventLoopProxy<AppEvent>, model: &str) -> Self {
        suppress_whisper_logs();

        let is_english_model = model.ends_with(".en");
        let model_name = model.to_string();
        let (sender, receiver) = mpsc::channel::<TranscribeRequest>();
        let thread_proxy = proxy.clone();

        thread::spawn(move || {
            let model_path = match downloader::ensure_model(&model_name) {
                Ok(p) => p,
                Err(e) => {
                    let _ = thread_proxy.send_event(AppEvent::TranscriptionError(
                        format!("model load failed: {e}"),
                    ));
                    return;
                }
            };

            let model_path_str = match model_path.to_str() {
                Some(s) => s.to_string(),
                None => {
                    let _ = thread_proxy.send_event(AppEvent::TranscriptionError(
                        "invalid model path".to_string(),
                    ));
                    return;
                }
            };

            let mut backend: Box<dyn TranscriptionBackend> =
                match whisper_backend::WhisperBackend::new(&model_path_str, is_english_model) {
                    Ok(b) => Box::new(b),
                    Err(e) => {
                        let _ = thread_proxy.send_event(AppEvent::TranscriptionError(e));
                        return;
                    }
                };

            let _ = thread_proxy.send_event(AppEvent::TranscriberReady);

            while let Ok(req) = receiver.recv() {
                match backend.transcribe(&req.samples, &req.languages) {
                    Ok(text) => {
                        let _ = thread_proxy.send_event(AppEvent::TranscriptionComplete(text));
                    }
                    Err(e) => {
                        let _ = thread_proxy.send_event(AppEvent::TranscriptionError(e));
                    }
                }
            }
        });

        Self { sender, proxy }
    }

    pub fn transcribe(&self, samples: Vec<f32>, languages: Vec<String>) {
        if self
            .sender
            .send(TranscribeRequest { samples, languages })
            .is_err()
        {
            let _ = self
                .proxy
                .send_event(AppEvent::TranscriptionError(
                    "transcriber unavailable".to_string(),
                ));
        }
    }
}

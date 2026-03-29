mod whisper_backend;

#[cfg(feature = "fluid_audio")]
mod fluid_backend;

use std::sync::mpsc;
use std::thread;

use tao::event_loop::EventLoopProxy;

use crate::app::AppEvent;
use crate::config::Tier;
use crate::downloader;

pub trait TranscriptionBackend: Send {
    fn transcribe(&mut self, samples: &[f32], languages: &[String]) -> Result<String, String>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendChoice {
    Whisper(String),
    #[cfg(feature = "fluid_audio")]
    FluidAudio,
}

pub fn resolve_backend(tier: &Tier, _languages: &[String]) -> BackendChoice {
    match tier {
        Tier::Fast => {
            #[cfg(feature = "fluid_audio")]
            if crate::platform::is_apple_silicon() {
                return BackendChoice::FluidAudio;
            }
            BackendChoice::Whisper(tier.whisper_model().to_string())
        }
        Tier::Standard | Tier::Accurate => BackendChoice::Whisper(tier.whisper_model().to_string()),
    }
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
    pub fn new(
        proxy: EventLoopProxy<AppEvent>,
        choice: BackendChoice,
        on_ready: AppEvent,
        on_fail: fn(String) -> AppEvent,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<TranscribeRequest>();
        let thread_proxy = proxy.clone();

        thread::spawn(move || {
            let backend_label = format!("{choice:?}");
            let mut backend: Box<dyn TranscriptionBackend> =
                match Self::create_backend(&choice, &thread_proxy, on_fail) {
                    Some(b) => b,
                    None => return,
                };

            let _ = thread_proxy.send_event(on_ready);

            while let Ok(req) = receiver.recv() {
                eprintln!("[murmur] transcribing with {backend_label}");
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

    fn create_backend(
        choice: &BackendChoice,
        proxy: &EventLoopProxy<AppEvent>,
        on_fail: fn(String) -> AppEvent,
    ) -> Option<Box<dyn TranscriptionBackend>> {
        match choice {
            BackendChoice::Whisper(model) => {
                suppress_whisper_logs();
                let is_english = model.ends_with(".en");
                let model_path = match downloader::ensure_model(model) {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = proxy.send_event(on_fail(format!("model load failed: {e}")));
                        return None;
                    }
                };

                let path_str = match model_path.to_str() {
                    Some(s) => s.to_string(),
                    None => {
                        let _ = proxy.send_event(on_fail("invalid model path".to_string()));
                        return None;
                    }
                };

                match whisper_backend::WhisperBackend::new(&path_str, is_english) {
                    Ok(b) => Some(Box::new(b)),
                    Err(e) => {
                        let _ = proxy.send_event(on_fail(e));
                        None
                    }
                }
            }
            #[cfg(feature = "fluid_audio")]
            BackendChoice::FluidAudio => {
                let result = std::panic::catch_unwind(fluid_backend::FluidAudioBackend::new);
                match result {
                    Ok(Ok(b)) => {
                        eprintln!("[murmur] FluidAudio (Parakeet) backend loaded");
                        Some(Box::new(b))
                    }
                    Ok(Err(e)) => {
                        eprintln!("[murmur] FluidAudio failed: {e}, falling back to Whisper");
                        Self::create_backend(
                            &BackendChoice::Whisper("small".to_string()),
                            proxy,
                            on_fail,
                        )
                    }
                    Err(_) => {
                        eprintln!("[murmur] FluidAudio panicked, falling back to Whisper");
                        Self::create_backend(
                            &BackendChoice::Whisper("small".to_string()),
                            proxy,
                            on_fail,
                        )
                    }
                }
            }
        }
    }

    pub fn transcribe(&self, samples: Vec<f32>, languages: Vec<String>) {
        if self
            .sender
            .send(TranscribeRequest { samples, languages })
            .is_err()
        {
            let _ = self.proxy.send_event(AppEvent::TranscriptionError(
                "transcriber unavailable".to_string(),
            ));
        }
    }
}

use std::sync::mpsc;
use std::thread;

use tao::event_loop::EventLoopProxy;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::app::AppEvent;
use crate::downloader;

fn suppress_whisper_logs() {
    unsafe {
        whisper_rs_sys::whisper_log_set(None, std::ptr::null_mut());
    }
}

pub struct Transcriber {
    sender: mpsc::Sender<Vec<f32>>,
}

impl Transcriber {
    pub fn new(proxy: EventLoopProxy<AppEvent>, model: &str) -> Result<Self, String> {
        suppress_whisper_logs();

        let model_path = downloader::ensure_model(model)?;
        let model_path_str = model_path
            .to_str()
            .ok_or("invalid model path")?
            .to_string();

        let (sender, receiver) = mpsc::channel::<Vec<f32>>();

        thread::spawn(move || {
            let ctx = WhisperContext::new_with_params(&model_path_str, WhisperContextParameters::default())
                .expect("failed to load whisper model");

            while let Ok(samples) = receiver.recv() {
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                params.set_language(Some("en"));
                params.set_print_special(false);
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);
                params.set_n_threads(4);

                let mut state = ctx.create_state().expect("failed to create whisper state");

                match state.full(params, &samples) {
                    Ok(_) => {
                        let n_segments = state.full_n_segments().unwrap_or(0);
                        let mut text = String::new();
                        for i in 0..n_segments {
                            if let Ok(segment) = state.full_get_segment_text(i) {
                                text.push_str(&segment);
                            }
                        }
                        let text = text.trim().to_string();
                        if text.is_empty() {
                            let _ = proxy.send_event(AppEvent::TranscriptionError(
                                "no speech detected".to_string(),
                            ));
                        } else {
                            let _ = proxy.send_event(AppEvent::TranscriptionComplete(text));
                        }
                    }
                    Err(e) => {
                        let _ = proxy.send_event(AppEvent::TranscriptionError(format!(
                            "transcription failed: {e}"
                        )));
                    }
                }
            }
        });

        Ok(Self { sender })
    }

    pub fn transcribe(&self, samples: Vec<f32>) {
        let _ = self.sender.send(samples);
    }
}

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

struct TranscribeRequest {
    samples: Vec<f32>,
    languages: Vec<String>,
}

pub struct Transcriber {
    sender: mpsc::Sender<TranscribeRequest>,
}

impl Transcriber {
    pub fn new(proxy: EventLoopProxy<AppEvent>, model: &str) -> Result<Self, String> {
        suppress_whisper_logs();

        let model_path = downloader::ensure_model(model)?;
        let model_path_str = model_path
            .to_str()
            .ok_or("invalid model path")?
            .to_string();

        let is_english_model = model.ends_with(".en");
        let (sender, receiver) = mpsc::channel::<TranscribeRequest>();

        thread::spawn(move || {
            let ctx = WhisperContext::new_with_params(
                &model_path_str,
                WhisperContextParameters::default(),
            )
            .expect("failed to load whisper model");

            while let Ok(req) = receiver.recv() {
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                params.set_translate(false);
                params.set_print_special(false);
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);
                params.set_n_threads(4);

                let mut state = ctx.create_state().expect("failed to create whisper state");

                let lang = if is_english_model {
                    "en".to_string()
                } else {
                    detect_language(&mut state, &req.samples, &req.languages)
                };
                params.set_language(Some(&lang));

                match state.full(params, &req.samples) {
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

    pub fn transcribe(&self, samples: Vec<f32>, languages: Vec<String>) {
        let _ = self.sender.send(TranscribeRequest { samples, languages });
    }
}

fn detect_language(
    state: &mut whisper_rs::WhisperState,
    samples: &[f32],
    preferred: &[String],
) -> String {
    if preferred.len() == 1 {
        return preferred[0].clone();
    }

    if state.pcm_to_mel(samples, 4).is_err() {
        return preferred.first().cloned().unwrap_or("en".to_string());
    }

    let probs = match state.lang_detect(0, 4) {
        Ok((_, probs)) => probs,
        Err(_) => return preferred.first().cloned().unwrap_or("en".to_string()),
    };

    let mut best_code = "en".to_string();
    let mut best_prob: f32 = -1.0;

    for lang in preferred {
        if let Some(id) = whisper_rs::get_lang_id(lang) {
            let id = id as usize;
            if id < probs.len() && probs[id] > best_prob {
                best_prob = probs[id];
                best_code = lang.clone();
            }
        }
    }

    best_code
}

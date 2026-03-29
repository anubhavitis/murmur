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

            let ctx = match WhisperContext::new_with_params(
                &model_path_str,
                WhisperContextParameters::default(),
            ) {
                Ok(c) => c,
                Err(e) => {
                    let _ = thread_proxy.send_event(AppEvent::TranscriptionError(
                        format!("failed to load whisper model: {e}"),
                    ));
                    return;
                }
            };

            let mut state = match ctx.create_state() {
                Ok(s) => s,
                Err(e) => {
                    let _ = thread_proxy.send_event(AppEvent::TranscriptionError(
                        format!("failed to create whisper state: {e}"),
                    ));
                    return;
                }
            };

            let _ = thread_proxy.send_event(AppEvent::TranscriberReady);

            while let Ok(req) = receiver.recv() {
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                params.set_translate(false);
                params.set_print_special(false);
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);
                params.set_n_threads(4);

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
                            let _ = thread_proxy.send_event(AppEvent::TranscriptionError(
                                "no speech detected".to_string(),
                            ));
                        } else {
                            let _ =
                                thread_proxy.send_event(AppEvent::TranscriptionComplete(text));
                        }
                    }
                    Err(e) => {
                        let _ = thread_proxy.send_event(AppEvent::TranscriptionError(format!(
                            "transcription failed: {e}"
                        )));
                    }
                }
            }
        });

        Self { sender, proxy }
    }

    pub fn transcribe(&self, samples: Vec<f32>, languages: Vec<String>) {
        if self.sender.send(TranscribeRequest { samples, languages }).is_err() {
            let _ = self.proxy.send_event(AppEvent::TranscriptionError(
                "transcriber unavailable".to_string(),
            ));
        }
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

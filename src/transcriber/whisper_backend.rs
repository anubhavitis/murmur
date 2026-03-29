use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::TranscriptionBackend;

pub struct WhisperBackend {
    state: whisper_rs::WhisperState,
    is_english_model: bool,
}

impl WhisperBackend {
    pub fn new(model_path: &str, is_english_model: bool) -> Result<Self, String> {
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|e| format!("failed to load whisper model: {e}"))?;

        let state = ctx
            .create_state()
            .map_err(|e| format!("failed to create whisper state: {e}"))?;

        Ok(Self {
            state,
            is_english_model,
        })
    }
}

impl TranscriptionBackend for WhisperBackend {
    fn transcribe(&mut self, samples: &[f32], languages: &[String]) -> Result<String, String> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_n_threads(4);

        let lang = if self.is_english_model {
            "en".to_string()
        } else {
            detect_language(&mut self.state, samples, languages)
        };
        params.set_language(Some(&lang));

        self.state
            .full(params, samples)
            .map_err(|e| format!("transcription failed: {e}"))?;

        let n_segments = self.state.full_n_segments().unwrap_or(0);
        let mut text = String::new();
        for i in 0..n_segments {
            if let Ok(segment) = self.state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }
        let text = text.trim().to_string();

        if text.is_empty() {
            Err("no speech detected".to_string())
        } else {
            Ok(text)
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

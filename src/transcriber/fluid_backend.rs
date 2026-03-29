use super::TranscriptionBackend;

pub struct FluidAudioBackend;

impl FluidAudioBackend {
    pub fn new() -> Result<Self, String> {
        Err("FluidAudio backend not yet implemented".to_string())
    }
}

impl TranscriptionBackend for FluidAudioBackend {
    fn transcribe(&mut self, _samples: &[f32], _languages: &[String]) -> Result<String, String> {
        Err("FluidAudio backend not yet implemented".to_string())
    }
}

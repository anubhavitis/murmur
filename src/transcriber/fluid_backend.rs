use std::io::Write;

use fluidaudio_rs::FluidAudio;

use super::TranscriptionBackend;

pub struct FluidAudioBackend {
    audio: FluidAudio,
}

impl FluidAudioBackend {
    pub fn new() -> Result<Self, String> {
        let audio = FluidAudio::new().map_err(|e| format!("FluidAudio init failed: {e}"))?;
        audio
            .init_asr()
            .map_err(|e| format!("FluidAudio ASR init failed: {e}"))?;
        Ok(Self { audio })
    }
}

impl TranscriptionBackend for FluidAudioBackend {
    fn transcribe(&mut self, samples: &[f32], _languages: &[String]) -> Result<String, String> {
        let tmp = write_wav_temp(samples).map_err(|e| format!("failed to write temp WAV: {e}"))?;
        let path_str = tmp.to_string_lossy().to_string();

        let result = self
            .audio
            .transcribe_file(&path_str)
            .map_err(|e| format!("FluidAudio transcription failed: {e}"));
        let _ = std::fs::remove_file(&tmp);
        let result = result?;

        let text = result.text.trim().to_string();
        if text.is_empty() {
            Err("no speech detected".to_string())
        } else {
            Ok(text)
        }
    }
}

fn write_wav_temp(samples: &[f32]) -> Result<std::path::PathBuf, std::io::Error> {
    let path = std::env::temp_dir().join(format!("murmur_fluid_{}.wav", std::process::id()));
    let mut f = std::fs::File::create(&path)?;

    let sample_rate: u32 = 16000;
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample) / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = (samples.len() * 2) as u32;

    // RIFF header
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_size).to_le_bytes())?;
    f.write_all(b"WAVE")?;

    // fmt chunk
    f.write_all(b"fmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&num_channels.to_le_bytes())?;
    f.write_all(&sample_rate.to_le_bytes())?;
    f.write_all(&byte_rate.to_le_bytes())?;
    f.write_all(&block_align.to_le_bytes())?;
    f.write_all(&bits_per_sample.to_le_bytes())?;

    // data chunk
    f.write_all(b"data")?;
    f.write_all(&data_size.to_le_bytes())?;
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let pcm = (clamped * 32767.0) as i16;
        f.write_all(&pcm.to_le_bytes())?;
    }

    Ok(path)
}

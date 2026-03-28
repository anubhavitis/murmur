use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const TARGET_SAMPLE_RATE: u32 = 16_000;

pub struct AudioCapture {
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<Stream>,
    recording: Arc<AtomicBool>,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            recording: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("no input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("no default input config: {e}"))?;

        let device_sample_rate = config.sample_rate().0;
        let device_channels = config.channels() as usize;

        self.buffer.lock().unwrap().clear();
        self.recording.store(true, Ordering::SeqCst);

        let buffer = Arc::clone(&self.buffer);
        let recording = Arc::clone(&self.recording);

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !recording.load(Ordering::SeqCst) {
                        return;
                    }
                    // downmix to mono
                    let mono: Vec<f32> = data
                        .chunks(device_channels)
                        .map(|frame| frame.iter().sum::<f32>() / device_channels as f32)
                        .collect();

                    // resample if needed (linear interpolation)
                    let samples = if device_sample_rate != TARGET_SAMPLE_RATE {
                        resample(&mono, device_sample_rate, TARGET_SAMPLE_RATE)
                    } else {
                        mono
                    };

                    buffer.lock().unwrap().extend_from_slice(&samples);
                },
                |err| eprintln!("audio stream error: {err}"),
                None,
            )
            .map_err(|e| format!("failed to build input stream: {e}"))?;

        stream.play().map_err(|e| format!("failed to start stream: {e}"))?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> Vec<f32> {
        self.recording.store(false, Ordering::SeqCst);
        self.stream = None; // drops the stream, stopping capture
        let mut buf = self.buffer.lock().unwrap();
        std::mem::take(&mut *buf)
    }
}

fn resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (input.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let sample = if idx + 1 < input.len() {
            input[idx] as f64 * (1.0 - frac) + input[idx + 1] as f64 * frac
        } else {
            input.get(idx).copied().unwrap_or(0.0) as f64
        };
        output.push(sample as f32);
    }
    output
}

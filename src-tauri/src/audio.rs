use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
}

// SAFETY: cpal::Stream on Windows (WASAPI) is thread-safe,
// but cpal marks it as !Send for cross-platform safety.
// We only target Windows, so this is safe.
unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
        }
    }

    pub fn start(&mut self, app_handle: AppHandle) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device found")?;

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Default,
        };

        let samples = self.samples.clone();
        let app = app_handle.clone();
        let fft_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let fft_buf_clone = fft_buffer.clone();

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Store PCM samples for transcription
                    // PCM-Samples für Transkription speichern
                    if let Ok(mut s) = samples.lock() {
                        s.extend_from_slice(data);
                    }
                    // Accumulate for FFT / Für FFT akkumulieren
                    if let Ok(mut fb) = fft_buf_clone.lock() {
                        fb.extend_from_slice(data);
                        // Process FFT every 512 samples (~30fps at 16kHz)
                        while fb.len() >= 512 {
                            let chunk: Vec<f32> = fb.drain(..512).collect();
                            let bins = compute_fft(&chunk, 16);
                            let _ = app.emit("fft-data", &bins);
                        }
                    }
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> Vec<f32> {
        self.stream = None; // Drop stops the stream
        let mut samples = self.samples.lock().unwrap();
        std::mem::take(&mut *samples)
    }
}

fn compute_fft(samples: &[f32], num_bins: usize) -> Vec<f32> {
    let len = samples.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(len);

    let mut buffer: Vec<Complex<f32>> = samples
        .iter()
        .map(|&s| Complex { re: s, im: 0.0 })
        .collect();

    fft.process(&mut buffer);

    // Take first half (positive frequencies), group into bins
    // Erste Hälfte (positive Frequenzen), in Bins gruppieren
    let half = len / 2;
    let bin_size = half / num_bins;
    (0..num_bins)
        .map(|i| {
            let start = i * bin_size;
            let end = start + bin_size;
            let magnitude: f32 = buffer[start..end]
                .iter()
                .map(|c| (c.re * c.re + c.im * c.im).sqrt())
                .sum::<f32>()
                / bin_size as f32;
            // Normalize to 0.0-1.0 range
            (magnitude * 10.0).min(1.0)
        })
        .collect()
}

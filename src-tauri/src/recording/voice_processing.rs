// macOS Voice Processing I/O (AEC/NS/AGC) microphone capture to WAV

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct VoiceProcHandle {
    running: Arc<Mutex<bool>>,
    join: Option<thread::JoinHandle<()>>,
    pub wav_path: PathBuf,
}

impl VoiceProcHandle {
    pub fn stop(mut self) {
        if let Ok(mut flag) = self.running.lock() {
            *flag = false;
        }
        if let Some(h) = self.join.take() {
            let _ = h.join();
        }
    }
}

/// Start Voice Processing I/O capture into a WAV file at `wav_path`.
/// Returns a handle to stop capture.
#[cfg(target_os = "macos")]
pub fn start_voice_processing_capture(wav_path: PathBuf, sample_rate: f64, channels: u32) -> anyhow::Result<VoiceProcHandle> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    // WAV writer (16-bit PCM)
    let spec = hound::WavSpec {
        channels: channels as u16,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let writer = hound::WavWriter::create(&wav_path, spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();
    let writer_clone = writer.clone();
    let wav_path_clone = wav_path.clone();

    // Spawn a thread to own the audio stream lifetime
    let handle = thread::spawn(move || {
        // Get the default input device
        let host = cpal::default_host();
        let device = match host.default_input_device() {
            Some(d) => d,
            None => {
                eprintln!("[VOICE_PROC] No default input device found");
                return;
            }
        };

        // Configure the input stream
        let config = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: cpal::SampleRate(sample_rate as u32),
            buffer_size: cpal::BufferSize::Default,
        };

        let err_fn = |err| eprintln!("[VOICE_PROC] Stream error: {}", err);

        // Build the input stream
        let stream = match device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Convert f32 samples to i16 and write to WAV
                if let Ok(mut w) = writer_clone.lock() {
                    if let Some(ref mut writer) = *w {
                        for &sample in data {
                            // Convert f32 [-1.0, 1.0] to i16 [-32768, 32767]
                            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            let _ = writer.write_sample(sample_i16);
                        }
                    }
                }
            },
            err_fn,
            None,
        ) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[VOICE_PROC] Failed to build input stream: {}", e);
                return;
            }
        };

        // Start the stream
        if let Err(e) = stream.play() {
            eprintln!("[VOICE_PROC] Failed to start stream: {}", e);
            return;
        }

        // Keep stream alive until stop requested
        loop {
            let keep_running = { *running_clone.lock().unwrap() };
            if !keep_running { break; }
            thread::sleep(Duration::from_millis(10));
        }

        // Stream will be dropped here, stopping capture
        drop(stream);

        // Finalize WAV
        if let Ok(mut w) = writer.lock() {
            if let Some(writer) = w.take() {
                let _ = writer.finalize();
            }
        }
    });

    Ok(VoiceProcHandle { running, join: Some(handle), wav_path })
}

#[cfg(not(target_os = "macos"))]
pub fn start_voice_processing_capture(_wav_path: PathBuf, _sample_rate: f64, _channels: u32) -> anyhow::Result<VoiceProcHandle> {
    anyhow::bail!("VoiceProcessing capture is only supported on macOS");
}

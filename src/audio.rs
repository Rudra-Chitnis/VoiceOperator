
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SILENCE_DURATION: Duration = Duration::from_millis(1800);
const MAX_RECORDING_DURATION: Duration = Duration::from_secs(30);
const STARTUP_GRACE: Duration = Duration::from_millis(800);
const MONITOR_INTERVAL: Duration = Duration::from_millis(100);
const SILENCE_THRESHOLD: f32 = 0.015;

pub fn list_input_devices() {
    let host = cpal::default_host();

    println!("\nAvailable microphone/audio input devices:");

    match host.input_devices() {
        Ok(devices) => {
            let mut found = false;

            for (index, device) in devices.enumerate() {
                found = true;
                println!("  {}. {}", index + 1, device);
            }

            if !found {
                println!("No input devices found.");
            }
        }
        Err(error) => {
            println!("Failed to enumerate input devices: {error}");
        }
    }

    match host.default_input_device() {
        Some(device) => println!("\nDefault input device: {device}"),
        None => println!("\nNo default input device found."),
    }
}

pub fn capture_audio_until_silence() {
    capture_audio_internal();
}

fn capture_audio_internal() {
    let host = cpal::default_host();

    let Some(device) = host.default_input_device() else {
        println!("No default microphone found.");
        return;
    };

    println!("\nUsing microphone: {device}");

    let supported_config = match device.default_input_config() {
        Ok(config) => config,
        Err(error) => {
            println!("Failed to get microphone configuration: {error}");
            return;
        }
    };

    let sample_format = supported_config.sample_format();
    let stream_config = supported_config.config();

    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_samples = Arc::clone(&samples);

    let error_callback = |error| {
        eprintln!("Audio stream error: {error}");
    };

    let stream_result: Result<Stream, cpal::Error> = match sample_format {
        SampleFormat::F32 => {
            device.build_input_stream(
                stream_config,
                move |data: &[f32], _| {
                    if let Ok(mut buffer) = captured_samples.lock() {
                        buffer.extend_from_slice(data);
                    }
                },
                error_callback,
                None,
            )
        }

        SampleFormat::I16 => {
            device.build_input_stream(
                stream_config,
                move |data: &[i16], _| {
                    if let Ok(mut buffer) = captured_samples.lock() {
                        buffer.extend(
                            data.iter().map(|&sample| {
                                sample as f32 / 32768.0
                            }),
                        );
                    }
                },
                error_callback,
                None,
            )
        }

        SampleFormat::U16 => {
            device.build_input_stream(
                stream_config,
                move |data: &[u16], _| {
                    if let Ok(mut buffer) = captured_samples.lock() {
                        buffer.extend(
                            data.iter().map(|&sample| {
                                (sample as f32 / 65535.0) * 2.0 - 1.0
                            }),
                        );
                    }
                },
                error_callback,
                None,
            )
        }

        other => {
            println!("Unsupported microphone sample format: {other:?}");
            return;
        }
    };

    let stream = match stream_result {
        Ok(stream) => stream,
        Err(error) => {
            println!("Failed to build audio stream: {error}");
            return;
        }
    };

    if let Err(error) = stream.play() {
        println!("Failed to start microphone: {error}");
        return;
    }

    println!("\nRecording started! Speak now.");
    println!("Recording will stop automatically when silence is detected.");

    let recording_start = Instant::now();
    let mut silence_start: Option<Instant> = None;
    let mut last_checked_samples = 0usize;

    let samples_per_check =
        (stream_config.sample_rate as usize
            * stream_config.channels as usize
            * MONITOR_INTERVAL.as_millis() as usize)
            / 1000;

    loop {
        std::thread::sleep(MONITOR_INTERVAL);

        let current_samples = match samples.lock() {
            Ok(buffer) => buffer.len(),
            Err(_) => {
                println!("Audio buffer lock failed.");
                return;
            }
        };

        if recording_start.elapsed() >= MAX_RECORDING_DURATION {
            println!("\nMaximum recording duration reached.");
            break;
        }

        if recording_start.elapsed() < STARTUP_GRACE {
            last_checked_samples = current_samples;
            continue;
        }

        if current_samples <= last_checked_samples {
            continue;
        }

        let new_samples = match samples.lock() {
            Ok(buffer) => {
                let start = last_checked_samples.min(buffer.len());
                let end = buffer.len();

                buffer[start..end].to_vec()
            }
            Err(_) => {
                println!("Audio buffer lock failed.");
                return;
            }
        };

        last_checked_samples = current_samples;

        if new_samples.is_empty() {
            continue;
        }

        let rms = (new_samples
            .iter()
            .map(|sample| sample * sample)
            .sum::<f32>()
            / new_samples.len() as f32)
            .sqrt();

        if rms < SILENCE_THRESHOLD {
            match silence_start {
                Some(start) => {
                    if start.elapsed() >= SILENCE_DURATION {
                        println!("\nSilence detected. Recording stopped.");
                        break;
                    }
                }
                None => {
                    silence_start = Some(Instant::now());
                }
            }
        } else {
            silence_start = None;
        }

        let elapsed = recording_start.elapsed().as_secs();
        println!(
            "Recording... {} seconds | Audio level: {:.4}",
            elapsed, rms
        );

        let _ = samples_per_check;
    }

    drop(stream);

    let buffer = match samples.lock() {
        Ok(buffer) => buffer,
        Err(_) => {
            println!("Audio buffer lock failed.");
            return;
        }
    };

    println!("\nAudio capture finished!");
    println!("Sample rate: {} Hz", stream_config.sample_rate);
    println!("Channels: {}", stream_config.channels);
    println!("Captured samples: {}", buffer.len());

    if buffer.is_empty() {
        println!("No audio samples were captured.");
        return;
    }

    let peak = buffer
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0_f32, f32::max);

    let rms = (buffer
        .iter()
        .map(|sample| sample * sample)
        .sum::<f32>()
        / buffer.len() as f32)
        .sqrt();

    println!("Peak level: {:.4}", peak);
    println!("Average RMS level: {:.4}", rms);

    if rms < 0.005 {
        println!("Audio level is very low. Check your microphone.");
    } else {
        println!("Microphone audio appears to be captured!");
    }

    save_wav(
        &buffer,
        stream_config.sample_rate,
        stream_config.channels,
    );

    let whisper_samples = prepare_whisper_audio(
        &buffer,
        stream_config.sample_rate,
        stream_config.channels,
    );

    if !whisper_samples.is_empty() {
        save_whisper_wav(&whisper_samples);
    }
}

fn save_wav(samples: &[f32], sample_rate: u32, channels: u16) {
    if let Err(error) = fs::create_dir_all("data") {
        println!("Failed to create data directory: {error}");
        return;
    }

    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = match hound::WavWriter::create(
        "data/test_recording.wav",
        spec,
    ) {
        Ok(writer) => writer,
        Err(error) => {
            println!("Failed to create WAV file: {error}");
            return;
        }
    };

    for sample in samples {
        let converted = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;

        if let Err(error) = writer.write_sample(converted) {
            println!("Failed to write audio sample: {error}");
            return;
        }
    }

    match writer.finalize() {
        Ok(()) => println!("WAV file saved: data/test_recording.wav"),
        Err(error) => println!("Failed to finalize WAV file: {error}"),
    }
}

fn prepare_whisper_audio(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Vec<f32> {
    if sample_rate != 48_000 || channels != 2 {
        println!("This conversion currently expects 48 kHz stereo audio.");
        return Vec::new();
    }

    let mono: Vec<f32> = samples
        .chunks_exact(2)
        .map(|frame| (frame[0] + frame[1]) / 2.0)
        .collect();

    mono.iter().step_by(3).copied().collect()
}

fn save_whisper_wav(samples: &[f32]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = match hound::WavWriter::create(
        "data/whisper_input.wav",
        spec,
    ) {
        Ok(writer) => writer,
        Err(error) => {
            println!("Failed to create Whisper WAV: {error}");
            return;
        }
    };

    for sample in samples {
        let converted = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;

        if let Err(error) = writer.write_sample(converted) {
            println!("Failed to write Whisper sample: {error}");
            return;
        }
    }

    match writer.finalize() {
        Ok(()) => println!("Whisper-ready WAV saved: data/whisper_input.wav"),
        Err(error) => println!("Failed to finalize Whisper WAV: {error}"),
    }
}

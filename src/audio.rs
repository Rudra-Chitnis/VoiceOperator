
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

pub fn capture_audio(duration_seconds: u64) {
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

    let stream_result: Result<Stream, cpal::Error> =
        match sample_format {
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

    println!(
        "\nRecording audio for {} seconds... Speak now!",
        duration_seconds
    );

    thread::sleep(Duration::from_secs(duration_seconds));

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
}


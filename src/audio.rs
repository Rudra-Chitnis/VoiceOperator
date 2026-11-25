
use cpal::traits::HostTrait;

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
        Some(device) => {
            println!("\nDefault input device: {device}");
        }
        None => println!("\nNo default input device found."),
    }
}

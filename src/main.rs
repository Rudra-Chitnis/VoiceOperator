
mod commands;
mod context;
mod executor;
mod parser;
mod registry;
mod audio;
mod speech;
mod intent_bridge;
mod discovery;

use context::SessionContext;
use intent_bridge::IntentBridge;
use std::io::{self, Write};

fn main() {
    println!("================================");
    println!("       VoiceOperator v0.8");
    println!("================================");

    let mut context = SessionContext::default();

    // Discover applications.
    let app_catalog = match discovery::AppCatalog::discover() {
        Ok(catalog) => catalog,
        Err(error) => {
            println!("Application discovery failed: {error}");
            discovery::AppCatalog::from_apps(Vec::new())
        }
    };

    println!(
        "Application discovery complete: {} entries loaded.",
        app_catalog.apps.len()
    );

    executor::print_help();

    // Start the semantic intent bridge.
    let mut bridge = match IntentBridge::start() {
        Ok(bridge) => {
            println!("Semantic router connected.");
            Some(bridge)
        }
        Err(error) => {
            println!("Semantic router unavailable: {error}");
            None
        }
    };

    // Existing voice command pipeline.
    if bridge.is_some() {
        audio::list_input_devices();
        audio::capture_audio(5);

        match speech::transcribe_audio("data/whisper_input.wav") {
            Ok(transcript) => {
                println!("\nRecognized speech: {transcript}");

                if let Some(bridge) = bridge.as_mut() {
                    match bridge.classify(&transcript) {
                        Ok(Some(intent)) => {
                            println!("Executing approved intent...");

                            if !executor::execute_intent(intent, &mut context) {
                                return;
                            }
                        }

                        Ok(None) => {
                            println!(
                                "Command rejected or no matching intent found."
                            );
                        }

                        Err(error) => {
                            println!("Intent routing error: {error}");
                        }
                    }
                }
            }

            Err(error) => {
                println!("\nTranscription error: {error}");
            }
        }
    }

    // Typed command loop with dynamic application resolution.
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input.");
            continue;
        }

        match parser::parse_command(&input, &app_catalog) {
            Some(intent) => {
                if !executor::execute_intent(intent, &mut context) {
                    break;
                }
            }

            None => {
                println!(
                    "I don't understand that command, or the app match is ambiguous. Type 'help' for commands."
                );
            }
        }
    }
}

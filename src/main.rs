mod commands;
mod context;
mod executor;
mod parser;
mod registry;
mod audio;

use context::SessionContext;
use std::io::{self, Write};

fn main() {
    println!("================================");
    println!("       VoiceOperator v0.6");
    println!("================================");

    let mut context = SessionContext::default();

    executor::print_help();
    audio::list_input_devices();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input.");
            continue;
        }

        match parser::parse_command(&input) {
            Some(intent) => {
                if !executor::execute_intent(intent, &mut context) {
                    break;
                }
            }

            None => {
                println!("I don't understand that command. Type 'help' for commands.");
            }
        }
    }
}
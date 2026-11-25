use crate::commands::{Application, Intent};
use crate::context::SessionContext;
use crate::registry::ACTION_REGISTRY;

use std::path::Path;
use std::process::Command;

pub fn print_help() {
    println!();
    println!("Available commands:");

    for spec in ACTION_REGISTRY {
        println!("  {:<22} {}", spec.aliases[0], spec.description);
    }

    println!("  {:<22} Open a folder by path", "go to <path>");
    println!("  {:<22} Open a folder by path", "open <path>");
    println!("  {:<22} Open a folder by path", "open folder <path>");
    println!();
}

fn launch_application(application: Application) -> Result<(), String> {
    let executable = match application {
        Application::Notepad => "notepad.exe",
        Application::Explorer => "explorer.exe",
        Application::Calculator => "calc.exe",
    };

    Command::new(executable)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to launch {executable}: {e}"))
}

pub fn execute_intent(
    intent: Intent,
    context: &mut SessionContext,
) -> bool {
    match intent {
        Intent::LaunchApplication { application } => {
            match launch_application(application) {
                Ok(()) => {
                    let name = match application {
                        Application::Notepad => "Notepad",
                        Application::Explorer => "File Explorer",
                        Application::Calculator => "Calculator",
                    };

                    context.remember_application(name.to_string());
                    println!("Launched {name}.");
                }

                Err(error) => println!("{error}"),
            }
        }

        Intent::OpenPath { path } => {
            if !Path::new(&path).exists() {
                println!("Path does not exist: {path}");
                return true;
            }

            match Command::new("explorer.exe").arg(&path).spawn() {
                Ok(_) => {
                    context.remember_folder(path.clone());
                    context.remember_application("File Explorer".to_string());
                    println!("Opened: {path}");
                }

                Err(e) => println!("Failed to open path: {e}"),
            }
        }

        Intent::ReopenLastPath => {
            match &context.last_folder {
                Some(path) => {
                    match Command::new("explorer.exe").arg(path).spawn() {
                        Ok(_) => println!("Reopened: {path}"),
                        Err(e) => println!("Failed to open folder: {e}"),
                    }
                }

                None => println!("No previous folder is stored in this session."),
            }
        }

        Intent::ShowHelp => {
            print_help();
        }

        Intent::Shutdown => {
            println!("VoiceOperator shutting down.");
            return false;
        }
    }

    true
}
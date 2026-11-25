use crate::commands::{Application, Intent};
use crate::registry::find_action;

pub fn parse_command(input: &str) -> Option<Intent> {
    let trimmed = input.trim();
    let normalized = trimmed.to_lowercase();

    if let Some(spec) = find_action(trimmed) {
        match spec.id {
            "open_notepad" => {
                return Some(Intent::LaunchApplication {
                    application: Application::Notepad,
                });
            }

            "open_explorer" => {
                return Some(Intent::LaunchApplication {
                    application: Application::Explorer,
                });
            }

            "open_calculator" => {
                return Some(Intent::LaunchApplication {
                    application: Application::Calculator,
                });
            }

            "open_last_folder" => {
                return Some(Intent::ReopenLastPath);
            }

            "help" => return Some(Intent::ShowHelp),

            "exit" => return Some(Intent::Shutdown),

            _ => {}
        }
    }

    let prefixes = ["open folder ", "go to ", "open "];

    for prefix in prefixes {
        if normalized.starts_with(prefix) {
            let path = trimmed[prefix.len()..].trim();

            if !path.is_empty() {
                return Some(Intent::OpenPath {
                    path: path.to_string(),
                });
            }
        }
    }

    None
}
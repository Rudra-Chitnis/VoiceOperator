
use crate::commands::{Application, Intent};
use crate::registry::find_action;

pub fn parse_command(input: &str) -> Option<Intent> {
    let trimmed = input.trim();
    let normalized = trimmed.to_lowercase();

    // Remove sentence-ending punctuation for registered command matching.
    let command_candidate = trimmed.trim_end_matches(['.', ',', '!', '?']);

    // First, check registered commands and aliases.
    if let Some(spec) = find_action(command_candidate) {
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

    // Check folder/path commands.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Application, Intent};

    #[test]
    fn parses_notepad_aliases() {
        for command in [
            "open notepad",
            "start notepad",
            "launch notepad",
            "OPEN NOTEPAD",
            "  open notepad  ",
        ] {
            assert!(matches!(
                parse_command(command),
                Some(Intent::LaunchApplication {
                    application: Application::Notepad
                })
            ));
        }
    }

    #[test]
    fn parses_calculator() {
        assert!(matches!(
            parse_command("open calculator"),
            Some(Intent::LaunchApplication {
                application: Application::Calculator
            })
        ));
    }

    #[test]
    fn parses_explorer() {
        assert!(matches!(
            parse_command("open explorer"),
            Some(Intent::LaunchApplication {
                application: Application::Explorer
            })
        ));
    }

    #[test]
    fn parses_folder_paths() {
        match parse_command("go to D:\\VoiceOperator") {
            Some(Intent::OpenPath { path }) => {
                assert_eq!(path, "D:\\VoiceOperator");
            }
            _ => panic!("Folder path was not parsed correctly"),
        }
    }

    #[test]
    fn parses_open_folder_command() {
        match parse_command("OPEN FOLDER D:\\VoiceOperator") {
            Some(Intent::OpenPath { path }) => {
                assert_eq!(path, "D:\\VoiceOperator");
            }
            _ => panic!("Open folder command was not parsed correctly"),
        }
    }

    #[test]
    fn parses_help() {
        assert!(matches!(
            parse_command("help"),
            Some(Intent::ShowHelp)
        ));
    }

    #[test]
    fn parses_shutdown_aliases() {
        for command in ["exit", "quit", "stop"] {
            assert!(matches!(
                parse_command(command),
                Some(Intent::Shutdown)
            ));
        }
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(parse_command("do something random").is_none());
    }
}

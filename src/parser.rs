use crate::commands::{Application, Intent};
use crate::discovery::AppCatalog;
use crate::registry::find_action;

pub fn parse_command(input: &str, catalog: &AppCatalog) -> Option<Intent> {
    let trimmed = input.trim();
    let normalized = trimmed.to_lowercase();

    let command_candidate = trimmed.trim_end_matches(['.', ',', '!', '?']);

    // Preserve existing fixed aliases.
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

    // Try resolving a discovered application name.
    if normalized.starts_with("open ")
        || normalized.starts_with("launch ")
        || normalized.starts_with("start ")
    {
        let query = trimmed
            .split_once(' ')
            .map(|(_, remainder)| remainder.trim())
            .unwrap_or("");

        if !query.is_empty() {
            let exact_matches = catalog.exact_match(query);

            if exact_matches.len() == 1 {
                let app = exact_matches[0];

                return Some(Intent::LaunchDiscoveredApp {
                    name: app.name.clone(),
                    target: app.target.clone(),
                });
            }

            // Only accept a partial match when it is unambiguous.
            if exact_matches.is_empty() {
                let matches = catalog.search(query);

                if matches.len() == 1 {
                    let app = matches[0];

                    return Some(Intent::LaunchDiscoveredApp {
                        name: app.name.clone(),
                        target: app.target.clone(),
                    });
                }
            }
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
    use crate::discovery::{AppCatalog, DiscoveredApp};

    fn test_catalog() -> AppCatalog {
        AppCatalog::from_apps(vec![
            DiscoveredApp {
                name: "Notepad".into(),
                target: "notepad.exe".into(),
            },
            DiscoveredApp {
                name: "Visual Studio Code".into(),
                target: "code.exe".into(),
            },
            DiscoveredApp {
                name: "Google Chrome".into(),
                target: "chrome.exe".into(),
            },
        ])
    }

    #[test]
    fn parses_notepad_aliases() {
        let catalog = test_catalog();

        for command in [
            "open notepad",
            "start notepad",
            "launch notepad",
            "OPEN NOTEPAD",
            "  open notepad  ",
        ] {
            assert!(matches!(
                parse_command(command, &catalog),
                Some(Intent::LaunchApplication {
                    application: Application::Notepad
                })
            ));
        }
    }

    #[test]
    fn parses_calculator() {
        let catalog = test_catalog();

        assert!(matches!(
            parse_command("open calculator", &catalog),
            Some(Intent::LaunchApplication {
                application: Application::Calculator
            })
        ));
    }

    #[test]
    fn parses_explorer() {
        let catalog = test_catalog();

        assert!(matches!(
            parse_command("open explorer", &catalog),
            Some(Intent::LaunchApplication {
                application: Application::Explorer
            })
        ));
    }

    #[test]
    fn parses_folder_paths() {
        let catalog = test_catalog();

        match parse_command("go to D:\\VoiceOperator", &catalog) {
            Some(Intent::OpenPath { path }) => {
                assert_eq!(path, "D:\\VoiceOperator");
            }
            _ => panic!("Folder path was not parsed correctly"),
        }
    }

    #[test]
    fn parses_open_folder_command() {
        let catalog = test_catalog();

        match parse_command("OPEN FOLDER D:\\VoiceOperator", &catalog) {
            Some(Intent::OpenPath { path }) => {
                assert_eq!(path, "D:\\VoiceOperator");
            }
            _ => panic!("Open folder command was not parsed correctly"),
        }
    }

    #[test]
    fn parses_help() {
        let catalog = test_catalog();

        assert!(matches!(
            parse_command("help", &catalog),
            Some(Intent::ShowHelp)
        ));
    }

    #[test]
    fn parses_shutdown_aliases() {
        let catalog = test_catalog();

        for command in ["exit", "quit", "stop"] {
            assert!(matches!(
                parse_command(command, &catalog),
                Some(Intent::Shutdown)
            ));
        }
    }

    #[test]
    fn resolves_discovered_application() {
        let catalog = test_catalog();

        match parse_command("open Visual Studio Code", &catalog) {
            Some(Intent::LaunchDiscoveredApp { name, target }) => {
                assert_eq!(name, "Visual Studio Code");
                assert_eq!(target, "code.exe");
            }
            _ => panic!("Discovered application was not resolved"),
        }
    }

    #[test]
    fn resolves_unique_partial_application_match() {
        let catalog = test_catalog();

        match parse_command("launch Visual Studio", &catalog) {
            Some(Intent::LaunchDiscoveredApp { name, .. }) => {
                assert_eq!(name, "Visual Studio Code");
            }
            _ => panic!("Unique partial match was not resolved"),
        }
    }

    #[test]
    fn rejects_unknown_commands() {
        let catalog = test_catalog();

        assert!(parse_command("do something random", &catalog).is_none());
    }
}

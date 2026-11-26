use crate::commands::{Application, Intent};
use crate::discovery::AppCatalog;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const PYTHON_PATH: &str =
    r"D:\VoiceOperator\laya_test\.venv\Scripts\python.exe";

const BRIDGE_PATH: &str =
    r"D:\VoiceOperator\laya_test\laya-intent-router\bridge.py";

const BRIDGE_DIRECTORY: &str =
    r"D:\VoiceOperator\laya_test\laya-intent-router";

#[derive(Serialize)]
struct BridgeRequest {
    text: String,
}

#[derive(Deserialize, Debug)]
struct BridgeResponse {
    status: String,
    reason: String,
    intent: Option<String>,
    score: f64,
}

pub struct IntentBridge {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl IntentBridge {
    pub fn start() -> Result<Self, String> {
        let mut child = Command::new(PYTHON_PATH)
            .arg(BRIDGE_PATH)
            .current_dir(BRIDGE_DIRECTORY)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to start Python bridge: {e}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or("Failed to connect to Python bridge stdin")?;

        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to connect to Python bridge stdout")?;

        Ok(Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    pub fn classify(
        &mut self,
        text: &str,
        catalog: &AppCatalog,
    ) -> Result<Option<Intent>, String> {
        let request = BridgeRequest {
            text: text.to_string(),
        };

        let json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;

        writeln!(self.stdin, "{json}")
            .map_err(|e| format!("Failed to send command to bridge: {e}"))?;

        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush bridge input: {e}"))?;

        let mut response_line = String::new();

        self.stdout
            .read_line(&mut response_line)
            .map_err(|e| format!("Failed to read bridge response: {e}"))?;

        if response_line.trim().is_empty() {
            return Err("Python bridge returned an empty response.".to_string());
        }

        let response: BridgeResponse = serde_json::from_str(&response_line)
            .map_err(|e| format!("Invalid bridge response: {e}"))?;

        println!(
            "Router status: {} | reason: {} | score: {:.3}",
            response.status,
            response.reason,
            response.score
        );

        if response.status != "approved" {
            return Ok(None);
        }

        let intent_id = response.intent
            .ok_or("Bridge approved a command without an intent")?;

        match intent_id.as_str() {
            "open_notepad" => Ok(Some(Intent::LaunchApplication {
                application: Application::Notepad,
            })),

            "open_explorer" => Ok(Some(Intent::LaunchApplication {
                application: Application::Explorer,
            })),

            "open_calculator" => Ok(Some(Intent::LaunchApplication {
                application: Application::Calculator,
            })),

            "open_last_folder" => Ok(Some(Intent::ReopenLastPath)),

            "help" => Ok(Some(Intent::ShowHelp)),

            "exit" => Ok(Some(Intent::Shutdown)),

            "launch_application" => {
                resolve_application(text, catalog)
            }

            _ => Err(format!(
                "Unknown intent received from bridge: {intent_id}"
            )),
        }
    }
}


// --------------------------------------------------
// Dynamic application resolution
// --------------------------------------------------

fn extract_application_query(text: &str) -> String {
    let mut query = text.trim().to_string();

    let prefixes = [
        "please ",
        "can you ",
        "could you ",
        "would you ",
        "open ",
        "launch ",
        "start ",
    ];

    // Remove polite prefixes and launch verbs repeatedly.
    loop {
        let lower = query.to_lowercase();
        let mut changed = false;

        for prefix in prefixes {
            if lower.starts_with(prefix) {
                query = query[prefix.len()..].trim().to_string();
                changed = true;
                break;
            }
        }

        if !changed {
            break;
        }
    }

    query = query
        .trim_matches(|c: char| {
            c.is_ascii_punctuation()
        })
        .trim()
        .to_string();

    query
}


fn resolve_application(
    text: &str,
    catalog: &AppCatalog,
) -> Result<Option<Intent>, String> {
    let query = extract_application_query(text);

    if query.is_empty() {
        println!("No application name was provided.");
        return Ok(None);
    }

    // Prefer exact matches.
    let exact_matches = catalog.exact_match(&query);

    if exact_matches.len() == 1 {
        let app = exact_matches[0];

        println!("Resolved application: {}", app.name);

        return Ok(Some(Intent::LaunchDiscoveredApp {
            name: app.name.clone(),
            target: app.target.clone(),
        }));
    }

    if exact_matches.len() > 1 {
        println!("Application match is ambiguous: {query}");
        return Ok(None);
    }

    // If there is no exact match, try a unique partial match.
    let partial_matches = catalog.search(&query);

    if partial_matches.len() == 1 {
        let app = partial_matches[0];

        println!("Resolved application: {}", app.name);

        return Ok(Some(Intent::LaunchDiscoveredApp {
            name: app.name.clone(),
            target: app.target.clone(),
        }));
    }

    if partial_matches.is_empty() {
        println!("No installed application matched: {query}");
        return Ok(None);
    }

    println!(
        "Application match is ambiguous for '{query}':"
    );

    for app in partial_matches {
        println!("  - {}", app.name);
    }

    Ok(None)
}


impl Drop for IntentBridge {
    fn drop(&mut self) {
        let _ = self._child.kill();
        let _ = self._child.wait();
    }
}


// --------------------------------------------------
// Tests
// --------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveredApp;

    fn sample_catalog() -> AppCatalog {
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
            DiscoveredApp {
                name: "Google Chrome Beta".into(),
                target: "chrome-beta.exe".into(),
            },
            DiscoveredApp {
                name: "Steam".into(),
                target: "steam.exe".into(),
            },
        ])
    }

    #[test]
    fn extracts_application_name() {
        assert_eq!(
            extract_application_query("Open Google Chrome"),
            "Google Chrome"
        );

        assert_eq!(
            extract_application_query("Please open Steam"),
            "Steam"
        );

        assert_eq!(
            extract_application_query("Can you launch Notepad"),
            "Notepad"
        );
    }

    #[test]
    fn resolves_exact_application_match() {
        let catalog = sample_catalog();

        match resolve_application("Open Google Chrome", &catalog)
            .unwrap()
        {
            Some(Intent::LaunchDiscoveredApp { name, target }) => {
                assert_eq!(name, "Google Chrome");
                assert_eq!(target, "chrome.exe");
            }
            _ => panic!("Expected Google Chrome to resolve"),
        }
    }

    #[test]
    fn resolves_unique_partial_match() {
        let catalog = sample_catalog();

        match resolve_application("Launch Steam", &catalog)
            .unwrap()
        {
            Some(Intent::LaunchDiscoveredApp { name, target }) => {
                assert_eq!(name, "Steam");
                assert_eq!(target, "steam.exe");
            }
            _ => panic!("Expected Steam to resolve"),
        }
    }

    #[test]
    fn rejects_ambiguous_partial_match() {
        let catalog = sample_catalog();

        let result = resolve_application("Open Google Chrome", &catalog)
            .unwrap();

        // Google Chrome is an exact match, so it should resolve
        // rather than being considered ambiguous.
        assert!(result.is_some());

        let result = resolve_application("Open Google", &catalog)
            .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn rejects_unknown_application() {
        let catalog = sample_catalog();

        let result = resolve_application(
            "Open an application that does not exist",
            &catalog,
        )
        .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn rejects_empty_application_query() {
        let catalog = sample_catalog();

        let result = resolve_application("Open", &catalog)
            .unwrap();

        assert!(result.is_none());
    }
}
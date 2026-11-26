use crate::commands::{Application, Intent};
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

    pub fn classify(&mut self, text: &str) -> Result<Option<Intent>, String> {
        let request = BridgeRequest {
            text: text.to_string(),
        };

        let json = serde_json::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;

        writeln!(self.stdin, "{json}")
            .map_err(|e| format!("Failed to send command to bridge: {e}"))?;

        self.stdin.flush()
            .map_err(|e| format!("Failed to flush bridge input: {e}"))?;

        let mut response_line = String::new();

        self.stdout.read_line(&mut response_line)
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

            _ => Err(format!("Unknown intent received from bridge: {intent_id}")),
        }
    }
}

impl Drop for IntentBridge {
    fn drop(&mut self) {
        let _ = self._child.kill();
        let _ = self._child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_bridge() {
        let mut bridge = IntentBridge::start()
            .expect("Python bridge failed to start");

        let result = bridge
            .classify("I need a place to jot something down")
            .expect("Classification failed");

        assert!(
            matches!(
                result,
                Some(Intent::LaunchApplication {
                    application: Application::Notepad
                })
            ),
            "Expected Notepad intent"
        );
    }
}
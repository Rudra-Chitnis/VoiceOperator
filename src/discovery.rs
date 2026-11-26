
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredApp {
    pub name: String,
    pub target: String,
}

#[derive(Debug, Clone, Default)]
pub struct AppCatalog {
    pub apps: Vec<DiscoveredApp>,
}

impl AppCatalog {
    pub fn discover() -> Result<Self, String> {
        let script = r#"
$ErrorActionPreference = 'Stop'

$roots = @(
    [Environment]::GetFolderPath('StartMenu'),
    [Environment]::GetFolderPath('CommonStartMenu')
)

$roots = $roots | Where-Object { $_ -and (Test-Path $_) } | Select-Object -Unique

$shell = New-Object -ComObject WScript.Shell
$results = @()

foreach ($root in $roots) {
    Get-ChildItem -Path $root -Filter '*.lnk' -Recurse -File -ErrorAction SilentlyContinue |
        ForEach-Object {
            try {
                $shortcut = $shell.CreateShortcut($_.FullName)
                $target = $shortcut.TargetPath

                if ($target -and (Test-Path $target)) {
                    $name = [System.IO.Path]::GetFileNameWithoutExtension($_.Name)

                    $results += [PSCustomObject]@{
                        name = $name
                        target = $target
                    }
                }
            }
            catch {
            }
        }
}

$results | ConvertTo-Json -Depth 3 -Compress
"#;

        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ])
            .output()
            .map_err(|e| format!("Failed to run application discovery: {e}"))?;

        if !output.status.success() {
            return Err(format!(
                "Application discovery failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let json = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if json.is_empty() {
            return Ok(Self::default());
        }

        let apps: Vec<DiscoveredApp> = match serde_json::from_str(&json) {
            Ok(apps) => apps,
            Err(_) => {
                // PowerShell emits a single object rather than an array
                // when exactly one shortcut is found.
                let single: DiscoveredApp = serde_json::from_str(&json)
                    .map_err(|e| format!("Invalid discovery output: {e}"))?;

                vec![single]
            }
        };

        Ok(Self::from_apps(apps))
    }

    pub fn from_apps(apps: Vec<DiscoveredApp>) -> Self {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for app in apps {
            let key = format!(
                "{}|{}",
                app.name.trim().to_lowercase(),
                app.target.trim().to_lowercase()
            );

            if seen.insert(key) {
                unique.push(app);
            }
        }

        Self { apps: unique }
    }

    pub fn search(&self, query: &str) -> Vec<&DiscoveredApp> {
        let query = query.trim().to_lowercase();

        self.apps
            .iter()
            .filter(|app| app.name.to_lowercase().contains(&query))
            .collect()
    }

    pub fn exact_match(&self, query: &str) -> Vec<&DiscoveredApp> {
        let query = query.trim();

        self.apps
            .iter()
            .filter(|app| app.name.eq_ignore_ascii_case(query))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                name: "Visual Studio Code".into(),
                target: "code.exe".into(),
            },
        ])
    }

    #[test]
    fn removes_exact_duplicates() {
        let catalog = sample_catalog();

        assert_eq!(catalog.apps.len(), 3);
    }

    #[test]
    fn finds_partial_name_matches() {
        let catalog = sample_catalog();

        let results = catalog.search("visual");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Visual Studio Code");
    }

    #[test]
    fn finds_case_insensitive_exact_matches() {
        let catalog = sample_catalog();

        let results = catalog.exact_match("notepad");

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn returns_empty_for_unknown_app() {
        let catalog = sample_catalog();

        assert!(catalog.search("does not exist").is_empty());
    }
}

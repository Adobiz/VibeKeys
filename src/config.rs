use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mapper::{KeyAction, MouseButton};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ActivationScope {
    Cli,
    All,
}

impl Default for ActivationScope {
    fn default() -> Self {
        Self::Cli
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UiLanguage {
    Zh,
    En,
}

impl Default for UiLanguage {
    fn default() -> Self {
        Self::Zh
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_terminals")]
    pub terminal_processes: Vec<String>,
    #[serde(default)]
    pub scope: ActivationScope,
    #[serde(default)]
    pub language: UiLanguage,
    #[serde(default = "default_bindings")]
    pub bindings: BTreeMap<String, KeyAction>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            terminal_processes: default_terminals(),
            scope: ActivationScope::Cli,
            language: UiLanguage::Zh,
            bindings: default_bindings(),
        }
    }
}

impl AppConfig {
    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let path = path
            .map(Path::to_path_buf)
            .unwrap_or_else(default_config_path);
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|err| format!("failed to parse {}: {err}", path.display()))
    }

    pub fn save_default(path: &Path) -> Result<(), String> {
        Self::default().save(path)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|err| format!("failed to serialize default config: {err}"))?;
        std::fs::write(path, format!("{content}\n"))
            .map_err(|err| format!("failed to write {}: {err}", path.display()))
    }

    pub fn map_button(&self, button: MouseButton) -> Option<KeyAction> {
        if !self.enabled {
            return None;
        }
        self.bindings.get(button.config_key()).copied()
    }

    pub fn is_terminal_process(&self, process_name: &str) -> bool {
        self.terminal_processes
            .iter()
            .any(|name| name.eq_ignore_ascii_case(process_name))
    }

    pub fn applies_to_process(&self, process_name: &str) -> bool {
        self.scope == ActivationScope::All || self.is_terminal_process(process_name)
    }
}

pub fn default_config_path() -> PathBuf {
    PathBuf::from("vibekeys.json")
}

pub fn app_config_path() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VibeKeys")
        .join("vibekeys.json")
}

pub fn webview_data_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VibeKeys")
        .join("WebView2")
}

fn default_enabled() -> bool {
    true
}

fn default_terminals() -> Vec<String> {
    vec![
        "WindowsTerminal.exe".to_string(),
        "pwsh.exe".to_string(),
        "powershell.exe".to_string(),
        "cmd.exe".to_string(),
    ]
}

fn default_bindings() -> BTreeMap<String, KeyAction> {
    BTreeMap::from([
        ("middle".to_string(), KeyAction::Enter),
        ("x1".to_string(), KeyAction::Up),
        ("x2".to_string(), KeyAction::Down),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_maps_buttons() {
        let config = AppConfig::default();
        assert_eq!(config.map_button(MouseButton::X1), Some(KeyAction::Up));
        assert_eq!(config.map_button(MouseButton::X2), Some(KeyAction::Down));
        assert_eq!(
            config.map_button(MouseButton::Middle),
            Some(KeyAction::Enter)
        );
    }

    #[test]
    fn disabled_config_maps_nothing() {
        let mut config = AppConfig::default();
        config.enabled = false;
        assert_eq!(config.map_button(MouseButton::X1), None);
    }

    #[test]
    fn default_scope_only_applies_to_cli() {
        let config = AppConfig::default();
        assert!(config.applies_to_process("pwsh.exe"));
        assert!(!config.applies_to_process("chrome.exe"));
    }

    #[test]
    fn all_scope_applies_to_every_process() {
        let mut config = AppConfig::default();
        config.scope = ActivationScope::All;
        assert!(config.applies_to_process("chrome.exe"));
    }

    #[test]
    fn configured_terminal_allowlist_is_the_source_of_truth() {
        let mut config = AppConfig::default();
        config.terminal_processes = vec!["wezterm-gui.exe".to_string()];

        assert!(config.applies_to_process("WEZTERM-GUI.EXE"));
        assert!(!config.applies_to_process("pwsh.exe"));
    }

    #[test]
    fn default_language_is_chinese() {
        assert_eq!(AppConfig::default().language, UiLanguage::Zh);
    }
}

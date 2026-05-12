use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use super::error::ProcessError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StartMode {
    Normal,
    Safe,
    Offline,
    Recovery,
    Debug,
    Minimal,
}

impl Default for StartMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl StartMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            StartMode::Normal => "normal",
            StartMode::Safe => "safe",
            StartMode::Offline => "offline",
            StartMode::Recovery => "recovery",
            StartMode::Debug => "debug",
            StartMode::Minimal => "minimal",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, ProcessError> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(StartMode::Normal),
            "safe" => Ok(StartMode::Safe),
            "offline" => Ok(StartMode::Offline),
            "recovery" => Ok(StartMode::Recovery),
            "debug" => Ok(StartMode::Debug),
            "minimal" => Ok(StartMode::Minimal),
            _ => Err(ProcessError::InvalidStartMode(s.to_string())),
        }
    }

    pub fn get_jvm_args(&self) -> Vec<String> {
        match self {
            StartMode::Normal => vec![
                "-Xmx4G".to_string(),
                "-Xms2G".to_string(),
            ],
            StartMode::Safe => vec![
                "-Xmx2G".to_string(),
                "-Xms1G".to_string(),
                "-XX:+UseG1GC".to_string(),
                "-XX:+UnlockExperimentalVMOptions".to_string(),
            ],
            StartMode::Offline => vec![
                "-Xmx2G".to_string(),
                "-Xms1G".to_string(),
                "-Donline-mode=false".to_string(),
            ],
            StartMode::Recovery => vec![
                "-Xmx1G".to_string(),
                "-Xms512M".to_string(),
                "-Dfailsafe=true".to_string(),
            ],
            StartMode::Debug => vec![
                "-Xmx4G".to_string(),
                "-Xms2G".to_string(),
                "-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005".to_string(),
                "-XX:+UseG1GC".to_string(),
            ],
            StartMode::Minimal => vec![
                "-Xmx1G".to_string(),
                "-Xms512M".to_string(),
                "-XX:+UseSerialGC".to_string(),
            ],
        }
    }

    pub fn get_description(&self) -> &'static str {
        match self {
            StartMode::Normal => "Normal server startup with full features",
            StartMode::Safe => "Safe mode with reduced memory and experimental options",
            StartMode::Offline => "Offline mode for LAN or cracked server testing",
            StartMode::Recovery => "Recovery mode with minimal resources for world repair",
            StartMode::Debug => "Debug mode with JDWP enabled for remote debugging",
            StartMode::Minimal => "Minimal mode with minimal resources for testing",
        }
    }

    pub fn get_server_properties_overrides(&self) -> HashMap<String, String> {
        let mut overrides = HashMap::new();

        match self {
            StartMode::Offline => {
                overrides.insert("online-mode".to_string(), "false".to_string());
            }
            StartMode::Safe => {
                overrides.insert("view-distance".to_string(), "8".to_string());
                overrides.insert("simulation-distance".to_string(), "4".to_string());
            }
            StartMode::Recovery => {
                overrides.insert("view-distance".to_string(), "4".to_string());
                overrides.insert("spawn-monsters".to_string(), "false".to_string());
            }
            StartMode::Minimal => {
                overrides.insert("view-distance".to_string(), "6".to_string());
                overrides.insert("spawn-npcs".to_string(), "false".to_string());
                overrides.insert("spawn-monsters".to_string(), "false".to_string());
            }
            _ => {}
        }

        overrides
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartModeConfig {
    pub default_mode: StartMode,
    pub allowed_modes: Vec<StartMode>,
    pub require_confirmation: Vec<StartMode>,
}

impl Default for StartModeConfig {
    fn default() -> Self {
        Self {
            default_mode: StartMode::Normal,
            allowed_modes: vec![
                StartMode::Normal,
                StartMode::Safe,
                StartMode::Offline,
                StartMode::Recovery,
                StartMode::Debug,
                StartMode::Minimal,
            ],
            require_confirmation: vec![
                StartMode::Recovery,
                StartMode::Debug,
            ],
        }
    }
}

pub struct StartModeManager {
    config: StartModeConfig,
}

impl StartModeManager {
    pub fn new(config: StartModeConfig) -> Self {
        Self { config }
    }

    pub fn get_default_mode(&self) -> StartMode {
        self.config.default_mode
    }

    pub fn is_mode_allowed(&self, mode: StartMode) -> bool {
        self.config.allowed_modes.contains(&mode)
    }

    pub fn requires_confirmation(&self, mode: StartMode) -> bool {
        self.config.require_confirmation.contains(&mode)
    }

    pub fn get_allowed_modes(&self) -> Vec<StartMode> {
        self.config.allowed_modes.clone()
    }

    pub fn set_default_mode(&mut self, mode: StartMode) -> Result<(), ProcessError> {
        if !self.is_mode_allowed(mode) {
            return Err(ProcessError::InvalidStartMode(format!(
                "Mode {:?} is not in allowed modes",
                mode
            )));
        }

        self.config.default_mode = mode;
        info!("Default start mode set to: {:?}", mode);
        Ok(())
    }

    pub fn add_allowed_mode(&mut self, mode: StartMode) {
        if !self.config.allowed_modes.contains(&mode) {
            self.config.allowed_modes.push(mode);
        }
    }

    pub fn remove_allowed_mode(&mut self, mode: StartMode) {
        self.config.allowed_modes.retain(|m| *m != mode);
    }

    pub fn prepare_start_args(&self, mode: StartMode) -> Result<StartArgs, ProcessError> {
        if !self.is_mode_allowed(mode) {
            return Err(ProcessError::InvalidStartMode(format!(
                "Mode {:?} is not allowed",
                mode
            )));
        }

        let jvm_args = mode.get_jvm_args();
        let properties_overrides = mode.get_server_properties_overrides();

        Ok(StartArgs {
            mode,
            jvm_args,
            properties_overrides,
            description: mode.get_description().to_string(),
        })
    }

    pub fn get_config(&self) -> StartModeConfig {
        self.config.clone()
    }
}

#[derive(Debug, Clone)]
pub struct StartArgs {
    pub mode: StartMode,
    pub jvm_args: Vec<String>,
    pub properties_overrides: HashMap<String, String>,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_mode_default() {
        let mode = StartMode::default();
        assert_eq!(mode, StartMode::Normal);
    }

    #[test]
    fn test_start_mode_from_str() {
        assert_eq!(StartMode::from_str("normal").unwrap(), StartMode::Normal);
        assert_eq!(StartMode::from_str("safe").unwrap(), StartMode::Safe);
        assert_eq!(StartMode::from_str("debug").unwrap(), StartMode::Debug);
        assert!(StartMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_start_mode_jvm_args() {
        let normal_args = StartMode::Normal.get_jvm_args();
        assert!(normal_args.contains(&"-Xmx4G".to_string()));

        let debug_args = StartMode::Debug.get_jvm_args();
        assert!(debug_args.iter().any(|arg| arg.contains("jdwp")));

        let offline_args = StartMode::Offline.get_jvm_args();
        assert!(offline_args.contains(&"-Donline-mode=false".to_string()));
    }

    #[test]
    fn test_start_mode_properties() {
        let overrides = StartMode::Offline.get_server_properties_overrides();
        assert_eq!(overrides.get("online-mode"), Some(&"false".to_string()));

        let minimal_overrides = StartMode::Minimal.get_server_properties_overrides();
        assert_eq!(minimal_overrides.get("spawn-monsters"), Some(&"false".to_string()));

        let normal_overrides = StartMode::Normal.get_server_properties_overrides();
        assert!(normal_overrides.is_empty());
    }

    #[test]
    fn test_start_mode_description() {
        assert!(!StartMode::Normal.get_description().is_empty());
        assert!(!StartMode::Recovery.get_description().is_empty());
    }

    #[test]
    fn test_start_mode_manager() {
        let manager = StartModeManager::new(StartModeConfig::default());

        assert_eq!(manager.get_default_mode(), StartMode::Normal);
        assert!(manager.is_mode_allowed(StartMode::Normal));
        assert!(!manager.requires_confirmation(StartMode::Normal));
        assert!(manager.requires_confirmation(StartMode::Recovery));
    }

    #[test]
    fn test_prepare_start_args() {
        let manager = StartModeManager::new(StartModeConfig::default());

        let args = manager.prepare_start_args(StartMode::Normal).unwrap();
        assert_eq!(args.mode, StartMode::Normal);
        assert!(!args.jvm_args.is_empty());

        let result = manager.prepare_start_args(StartMode::Debug);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_remove_mode() {
        let mut manager = StartModeManager::new(StartModeConfig::default());

        assert!(manager.is_mode_allowed(StartMode::Minimal));

        manager.remove_allowed_mode(StartMode::Minimal);
        assert!(!manager.is_mode_allowed(StartMode::Minimal));

        manager.add_allowed_mode(StartMode::Minimal);
        assert!(manager.is_mode_allowed(StartMode::Minimal));
    }

    #[test]
    fn test_set_default_mode() {
        let mut manager = StartModeManager::new(StartModeConfig::default());

        manager.set_default_mode(StartMode::Safe).unwrap();
        assert_eq!(manager.get_default_mode(), StartMode::Safe);

        let result = manager.set_default_mode(StartMode::Minimal);
        manager.remove_allowed_mode(StartMode::Minimal);
        assert!(result.is_err());
    }

    #[test]
    fn test_mode_serialization() {
        let mode = StartMode::Debug;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"debug\"");

        let deserialized: StartMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, StartMode::Debug);
    }
}

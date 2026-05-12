use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub rcon: RconConfig,
    pub api: ApiConfig,
    pub monitor: MonitorConfig,
    pub files: FilesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilesConfig {
    pub server_root: PathBuf,
    pub max_file_size: usize,
    pub allowed_extensions: Vec<String>,
    pub enable_versioning: bool,
    pub backup_retention_days: u32,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            server_root: PathBuf::from("."),
            max_file_size: 10 * 1024 * 1024,
            allowed_extensions: vec![
                "yml".to_string(),
                "yaml".to_string(),
                "json".to_string(),
                "properties".to_string(),
                "txt".to_string(),
                "log".to_string(),
                "xml".to_string(),
                "toml".to_string(),
            ],
            enable_versioning: true,
            backup_retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub jar_path: PathBuf,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub auto_restart: bool,
    #[serde(default = "default_timeout")]
    pub start_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RconConfig {
    #[serde(default = "default_rcon_host")]
    pub host: String,
    #[serde(default = "default_rcon_port")]
    pub port: u16,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_host")]
    pub host: String,
    #[serde(default = "default_api_port")]
    pub port: u16,
    #[serde(default = "default_max_ws")]
    pub max_ws_connections: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitorConfig {
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    #[serde(default = "default_history_size")]
    pub history_size: usize,
}

fn default_timeout() -> u64 { 60 }
fn default_rcon_host() -> String { "127.0.0.1".to_string() }
fn default_rcon_port() -> u16 { 25575 }
fn default_api_host() -> String { "0.0.0.0".to_string() }
fn default_api_port() -> u16 { 8080 }
fn default_max_ws() -> usize { 100 }
fn default_interval() -> u64 { 1 }
fn default_history_size() -> usize { 300 }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            jar_path: PathBuf::from("server.jar"),
            jvm_args: vec![
                "-Xmx4G".to_string(),
                "-Xms2G".to_string(),
                "-jar".to_string(),
            ],
            auto_restart: false,
            start_timeout_secs: 60,
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            interval_secs: 1,
            history_size: 300,
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        let config: Config = match ext {
            "yaml" | "yml" => {
                serde_yaml::from_str(&content)
                    .with_context(|| "Failed to parse YAML config")
            }
            _ => {
                toml::from_str(&content)
                    .with_context(|| "Failed to parse TOML config")
            }
        }?;

        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            Self::load(path)
        } else {
            tracing::warn!("Config file not found, using defaults");
            Ok(Self::default())
        }
    }

    fn validate(&self) -> Result<()> {
        if self.rcon.password.is_empty() {
            tracing::warn!("RCON password is empty, security risk!");
        }
        if self.api.port < 1024 && self.api.host != "127.0.0.1" && self.api.host != "localhost" {
            tracing::warn!("Listening on privileged port {} as non-localhost", self.api.port);
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            rcon: RconConfig {
                host: default_rcon_host(),
                port: default_rcon_port(),
                password: String::new(),
            },
            api: ApiConfig {
                host: default_api_host(),
                port: default_api_port(),
                max_ws_connections: default_max_ws(),
            },
            monitor: MonitorConfig::default(),
            files: FilesConfig::default(),
        }
    }
}

impl RconConfig {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

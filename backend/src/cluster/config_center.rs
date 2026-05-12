use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConfigCenter {
    state: Arc<ConfigCenterState>,
    subscribers: Arc<RwLock<HashMap<String, ConfigSubscriber>>>,
}

struct ConfigCenterState {
    configs: RwLock<HashMap<String, ClusterConfig>>,
    versions: RwLock<HashMap<String, ConfigVersion>>,
    snapshots: RwLock<Vec<ConfigSnapshot>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub id: String,
    pub name: String,
    pub config_type: ConfigType,
    pub content: serde_json::Value,
    pub version: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
    pub locked: bool,
    pub locked_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigType {
    Proxy,
    LoadBalancer,
    ChatSync,
    Failover,
    RollingUpdate,
    Node,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub comment: Option<String>,
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub id: String,
    pub config_id: String,
    pub version: String,
    pub content: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub auto_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSubscriber {
    pub id: String,
    pub name: String,
    pub node_id: String,
    pub subscribed_configs: Vec<String>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    pub config_id: String,
    pub change_type: ConfigChangeType,
    pub old_version: String,
    pub new_version: String,
    pub changed_by: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigChangeType {
    Created,
    Updated,
    Deleted,
    Locked,
    Unlocked,
}

impl ConfigCenter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(ConfigCenterState {
                configs: RwLock::new(HashMap::new()),
                versions: RwLock::new(HashMap::new()),
                snapshots: RwLock::new(Vec::new()),
            }),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_config(&self, name: String, config_type: ConfigType, content: serde_json::Value, created_by: &str) -> Result<ClusterConfig, ConfigError> {
        let config_id = format!("{:?}:{}", config_type, name);

        let mut configs = self.state.configs.write();
        if configs.contains_key(&config_id) {
            return Err(ConfigError::ConfigAlreadyExists(config_id.clone()));
        }

        let version = "1.0.0".to_string();
        let config = ClusterConfig {
            id: config_id.clone(),
            name,
            config_type: config_type.clone(),
            content: content.clone(),
            version: version.clone(),
            updated_at: Utc::now(),
            updated_by: created_by.to_string(),
            locked: false,
            locked_by: None,
        };

        configs.insert(config_id.clone(), config.clone());
        drop(configs);

        self.create_version(&config_id, &version, created_by, "Initial version", &content)?;
        self.create_snapshot(&config_id, &version, created_by, false)?;

        Ok(config)
    }

    pub fn get_config(&self, config_id: &str) -> Option<ClusterConfig> {
        self.state.configs.read().get(config_id).cloned()
    }

    pub fn get_configs_by_type(&self, config_type: ConfigType) -> Vec<ClusterConfig> {
        self.state.configs.read().values()
            .filter(|c| c.config_type == config_type)
            .cloned()
            .collect()
    }

    pub fn get_all_configs(&self) -> Vec<ClusterConfig> {
        self.state.configs.read().values().cloned().collect()
    }

    pub fn update_config(&self, config_id: &str, content: serde_json::Value, updated_by: &str, comment: Option<String>) -> Result<ClusterConfig, ConfigError> {
        let mut configs = self.state.configs.write();
        let config = configs.get_mut(config_id)
            .ok_or_else(|| ConfigError::ConfigNotFound(config_id.to_string()))?;

        if config.locked {
            return Err(ConfigError::ConfigLocked(config_id.to_string()));
        }

        let old_version = config.version.clone();
        let new_version = self.bump_version(&old_version);

        let snapshot_content = config.content.clone();
        config.content = content;
        config.version = new_version.clone();
        config.updated_at = Utc::now();
        config.updated_by = updated_by.to_string();

        let updated = config.clone();
        drop(configs);

        self.create_version(config_id, &new_version, updated_by, comment.as_deref(), &snapshot_content)?;

        Ok(updated)
    }

    pub fn delete_config(&self, config_id: &str) -> Result<(), ConfigError> {
        let mut configs = self.state.configs.write();
        let config = configs.get(config_id)
            .ok_or_else(|| ConfigError::ConfigNotFound(config_id.to_string()))?;

        if config.locked {
            return Err(ConfigError::ConfigLocked(config_id.to_string()));
        }

        configs.remove(config_id);
        Ok(())
    }

    pub fn lock_config(&self, config_id: &str, locked_by: &str) -> Result<(), ConfigError> {
        let mut configs = self.state.configs.write();
        let config = configs.get_mut(config_id)
            .ok_or_else(|| ConfigError::ConfigNotFound(config_id.to_string()))?;

        if config.locked && config.locked_by.as_deref() != Some(locked_by) {
            return Err(ConfigError::ConfigLockedByOther(config.locked_by.clone().unwrap_or_default()));
        }

        config.locked = true;
        config.locked_by = Some(locked_by.to_string());
        Ok(())
    }

    pub fn unlock_config(&self, config_id: &str, unlocked_by: &str) -> Result<(), ConfigError> {
        let mut configs = self.state.configs.write();
        let config = configs.get_mut(config_id)
            .ok_or_else(|| ConfigError::ConfigNotFound(config_id.to_string()))?;

        if !config.locked {
            return Ok(());
        }

        if config.locked_by.as_deref() != Some(unlocked_by) {
            return Err(ConfigError::ConfigLockedByOther(config.locked_by.clone().unwrap_or_default()));
        }

        config.locked = false;
        config.locked_by = None;
        Ok(())
    }

    pub fn get_versions(&self, config_id: &str) -> Vec<ConfigVersion> {
        let key = format!("{}:versions", config_id);
        self.state.versions.read().get(&key)
            .map(|v| vec![v.clone()])
            .unwrap_or_default()
    }

    pub fn rollback_to_version(&self, config_id: &str, version: &str, rolled_back_by: &str) -> Result<ClusterConfig, ConfigError> {
        let snapshot_id = format!("{}:{}", config_id, version);
        let snapshots = self.state.snapshots.read();
        let snapshot = snapshots.iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| ConfigError::VersionNotFound(version.to_string()))?;
        drop(snapshots);

        self.update_config(config_id, snapshot.content.clone(), rolled_back_by, Some(format!("Rollback to {}", version)))
    }

    fn create_version(&self, config_id: &str, version: &str, created_by: &str, comment: Option<&str>, _content: &serde_json::Value) -> Result<(), ConfigError> {
        let key = format!("{}:{}", config_id, version);
        let snapshot_key = format!("{}:{}", config_id, version);

        let versions = &mut *self.state.versions.write();
        versions.insert(key, ConfigVersion {
            version: version.to_string(),
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            comment: comment.map(String::from),
            snapshot_id: snapshot_key,
        });
        Ok(())
    }

    fn create_snapshot(&self, config_id: &str, version: &str, created_by: &str, auto_generated: bool) -> Result<ConfigSnapshot, ConfigError> {
        let configs = self.state.configs.read();
        let config = configs.get(config_id)
            .ok_or_else(|| ConfigError::ConfigNotFound(config_id.to_string()))?;

        let snapshot = ConfigSnapshot {
            id: format!("{}:{}", config_id, version),
            config_id: config_id.to_string(),
            version: version.to_string(),
            content: config.content.clone(),
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            auto_generated,
        };

        drop(configs);
        self.state.snapshots.write().push(snapshot.clone());
        Ok(snapshot)
    }

    fn bump_version(&self, current: &str) -> String {
        let parts: Vec<u32> = current.split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() == 3 {
            let patch = parts[2] + 1;
            format!("{}.{}.{}", parts[0], parts[1], patch)
        } else {
            "1.0.0".to_string()
        }
    }

    pub fn subscribe(&self, subscriber: ConfigSubscriber) {
        self.subscribers.write().insert(subscriber.id.clone(), subscriber);
    }

    pub fn unsubscribe(&self, subscriber_id: &str) {
        self.subscribers.write().remove(subscriber_id);
    }

    pub fn notify_subscribers(&self, event: ConfigChangeEvent) {
        let subscribers = self.subscribers.read();
        for subscriber in subscribers.values() {
            if subscriber.subscribed_configs.contains(&event.config_id) {
                tracing::debug!("Notifying subscriber {} about config change", subscriber.name);
            }
        }
    }

    pub fn export_config(&self, config_id: &str) -> Result<String, ConfigError> {
        let config = self.get_config(config_id)
            .ok_or_else(|| ConfigError::ConfigNotFound(config_id.to_string()))?;
        serde_json::to_string_pretty(&config).map_err(|e| ConfigError::SerializationError(e.to_string()))
    }

    pub fn import_config(&self, data: &str, imported_by: &str) -> Result<ClusterConfig, ConfigError> {
        let imported: ClusterConfig = serde_json::from_str(data)
            .map_err(|e| ConfigError::DeserializationError(e.to_string()))?;

        self.create_config(imported.name, imported.config_type, imported.content, imported_by)
    }
}

impl Default for ConfigCenter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config {0} already exists")]
    ConfigAlreadyExists(String),

    #[error("Config {0} not found")]
    ConfigNotFound(String),

    #[error("Config {0} is locked")]
    ConfigLocked(String),

    #[error("Config is locked by {0}")]
    ConfigLockedByOther(String),

    #[error("Version {0} not found")]
    VersionNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

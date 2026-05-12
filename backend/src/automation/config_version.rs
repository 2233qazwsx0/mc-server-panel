use crate::automation::ConfigVersion;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ConfigVersionManager {
    versions: RwLock<HashMap<String, ConfigVersion>>,
    current_version_id: RwLock<Option<String>>,
}

impl ConfigVersionManager {
    pub fn new() -> Self {
        Self {
            versions: RwLock::new(HashMap::new()),
            current_version_id: RwLock::new(None),
        }
    }

    pub fn create_version(&self, config_content: &str, description: &str) -> Result<ConfigVersion, String> {
        let version = ConfigVersion {
            id: Uuid::new_v4().to_string(),
            version: format!("v{}", self.versions.read().len() + 1),
            created_at: Utc::now(),
            description: description.to_string(),
            config_snapshot: config_content.to_string(),
        };

        {
            let mut versions = self.versions.write();
            versions.insert(version.id.clone(), version.clone());
        }
        {
            let mut current = self.current_version_id.write();
            *current = Some(version.id.clone());
        }

        info!("Config version created: {} ({})", version.version, version.id);
        Ok(version)
    }

    pub fn get_version(&self, version_id: &str) -> Option<ConfigVersion> {
        self.versions.read().get(version_id).cloned()
    }

    pub fn list_versions(&self) -> Vec<ConfigVersion> {
        let versions = self.versions.read();
        let mut list: Vec<_> = versions.values().cloned().collect();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    pub fn get_current_version(&self) -> Option<ConfigVersion> {
        let current_id = self.current_version_id.read();
        if let Some(id) = current_id.as_ref() {
            self.versions.read().get(id).cloned()
        } else {
            None
        }
    }

    pub fn set_current_version(&self, version_id: &str) -> Result<(), String> {
        let versions = self.versions.read();
        if !versions.contains_key(version_id) {
            return Err(format!("Version not found: {}", version_id));
        }
        drop(versions);

        {
            let mut current = self.current_version_id.write();
            *current = Some(version_id.to_string());
        }

        info!("Current version set to: {}", version_id);
        Ok(())
    }

    pub fn compare_versions(&self, version_id1: &str, version_id2: &str) -> Result<VersionDiff, String> {
        let versions = self.versions.read();

        let v1 = versions
            .get(version_id1)
            .ok_or_else(|| format!("Version not found: {}", version_id1))?;
        let v2 = versions
            .get(version_id2)
            .ok_or_else(|| format!("Version not found: {}", version_id2))?;

        let diff = VersionDiff {
            version1: v1.version.clone(),
            version2: v2.version.clone(),
            changes: self.compute_diff(&v1.config_snapshot, &v2.config_snapshot),
            size_diff: v2.config_snapshot.len() as i64 - v1.config_snapshot.len() as i64,
            timestamp_diff: v2.created_at.signed_duration_since(v1.created_at),
        };

        Ok(diff)
    }

    fn compute_diff(&self, old: &str, new: &str) -> Vec<ConfigChange> {
        let mut changes = Vec::new();

        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        let old_map: HashMap<&str, &str> = old_lines
            .iter()
            .filter(|l| l.contains('='))
            .map(|l| {
                let parts: Vec<&str> = l.splitn(2, '=').collect();
                (parts[0].trim(), l)
            })
            .collect();

        let new_map: HashMap<&str, &str> = new_lines
            .iter()
            .filter(|l| l.contains('='))
            .map(|l| {
                let parts: Vec<&str> = l.splitn(2, '=').collect();
                (parts[0].trim(), l)
            })
            .collect();

        for (key, new_value) in &new_map {
            if let Some(old_value) = old_map.get(key) {
                if old_value != new_value {
                    changes.push(ConfigChange {
                        key: key.to_string(),
                        change_type: ChangeType::Modified,
                        old_value: Some(old_value.to_string()),
                        new_value: Some(new_value.to_string()),
                    });
                }
            } else {
                changes.push(ConfigChange {
                    key: key.to_string(),
                    change_type: ChangeType::Added,
                    old_value: None,
                    new_value: Some(new_value.to_string()),
                });
            }
        }

        for (key, old_value) in &old_map {
            if !new_map.contains_key(key) {
                changes.push(ConfigChange {
                    key: key.to_string(),
                    change_type: ChangeType::Removed,
                    old_value: Some(old_value.to_string()),
                    new_value: None,
                });
            }
        }

        changes
    }

    pub fn rollback_to_version(&self, version_id: &str) -> Result<String, String> {
        let versions = self.versions.read();
        let version = versions
            .get(version_id)
            .ok_or_else(|| format!("Version not found: {}", version_id))?;
        let config_content = version.config_snapshot.clone();
        drop(versions);

        self.set_current_version(version_id)?;

        info!("Rolled back to version: {}", version_id);
        Ok(config_content)
    }

    pub fn delete_version(&self, version_id: &str) -> Result<(), String> {
        let mut versions = self.versions.write();

        if versions.len() <= 1 {
            return Err("Cannot delete the last version".to_string());
        }

        if !versions.contains_key(version_id) {
            return Err(format!("Version not found: {}", version_id));
        }

        versions.remove(version_id);

        {
            let mut current = self.current_version_id.write();
            if *current == Some(version_id.to_string()) {
                if let Some((id, _)) = versions.iter().next() {
                    *current = Some(id.clone());
                } else {
                    *current = None;
                }
            }
        }

        info!("Version deleted: {}", version_id);
        Ok(())
    }

    pub fn get_stats(&self) -> ConfigVersionStats {
        let versions = self.versions.read();
        ConfigVersionStats {
            total_versions: versions.len(),
            current_version: self.current_version_id.read().clone(),
            oldest_version: versions
                .values()
                .min_by_key(|v| v.created_at)
                .map(|v| v.id.clone()),
            newest_version: versions
                .values()
                .max_by_key(|v| v.created_at)
                .map(|v| v.id.clone()),
        }
    }

    pub fn export_version(&self, version_id: &str) -> Result<String, String> {
        let versions = self.versions.read();
        let version = versions
            .get(version_id)
            .ok_or_else(|| format!("Version not found: {}", version_id))?;

        let export = VersionExport {
            id: version.id.clone(),
            version: version.version.clone(),
            created_at: version.created_at.to_rfc3339(),
            description: version.description.clone(),
            config_snapshot: version.config_snapshot.clone(),
        };

        serde_json::to_string_pretty(&export)
            .map_err(|e| format!("Failed to serialize version: {}", e))
    }

    pub fn import_version(&self, export_json: &str) -> Result<ConfigVersion, String> {
        let export: VersionExport = serde_json::from_str(export_json)
            .map_err(|e| format!("Failed to parse export: {}", e))?;

        let version = ConfigVersion {
            id: export.id,
            version: export.version,
            created_at: DateTime::parse_from_rfc3339(&export.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            description: export.description,
            config_snapshot: export.config_snapshot,
        };

        {
            let mut versions = self.versions.write();
            versions.insert(version.id.clone(), version.clone());
        }

        info!("Version imported: {}", version.id);
        Ok(version)
    }
}

impl Default for ConfigVersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub version1: String,
    pub version2: String,
    pub changes: Vec<ConfigChange>,
    pub size_diff: i64,
    pub timestamp_diff: chrono::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub key: String,
    pub change_type: ChangeType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionExport {
    pub id: String,
    pub version: String,
    pub created_at: String,
    pub description: String,
    pub config_snapshot: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigVersionStats {
    pub total_versions: usize,
    pub current_version: Option<String>,
    pub oldest_version: Option<String>,
    pub newest_version: Option<String>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use super::error::ProcessError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub id: String,
    pub instance_id: String,
    pub created_at: DateTime<Utc>,
    pub snapshot_type: SnapshotType,
    pub metadata: SnapshotMetadata,
    pub state_data: Option<Vec<u8>>,
    pub checksums: SnapshotChecksums,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotType {
    Full,
    Incremental,
    StateOnly,
    MemoryDump,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub pid: u32,
    pub uptime_secs: u64,
    pub memory_usage_mb: u64,
    pub cpu_percent: f32,
    pub world_name: Option<String>,
    pub player_count: u32,
    pub tick_rate: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotChecksums {
    pub md5: Option<String>,
    pub sha256: Option<String>,
    pub crc32: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    pub enabled: bool,
    pub storage_path: PathBuf,
    pub max_snapshots: usize,
    pub auto_snapshot_interval_secs: u64,
    pub compress_snapshots: bool,
    pub incremental_enabled: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            storage_path: PathBuf::from("./snapshots"),
            max_snapshots: 10,
            auto_snapshot_interval_secs: 3600,
            compress_snapshots: true,
            incremental_enabled: true,
        }
    }
}

pub struct SnapshotManager {
    config: SnapshotConfig,
    snapshots: RwLock<VecDeque<ProcessSnapshot>>,
}

impl SnapshotManager {
    pub fn new(config: SnapshotConfig) -> Self {
        if !config.storage_path.exists() {
            if let Err(e) = fs::create_dir_all(&config.storage_path) {
                warn!("Failed to create snapshot directory: {}", e);
            }
        }

        Self {
            config,
            snapshots: RwLock::new(VecDeque::new()),
        }
    }

    pub async fn create_snapshot(
        &self,
        instance_id: String,
        snapshot_type: SnapshotType,
        metadata: SnapshotMetadata,
        state_data: Option<Vec<u8>>,
    ) -> Result<ProcessSnapshot, ProcessError> {
        let id = Uuid::new_v4().to_string();

        let checksums = if let Some(ref data) = state_data {
            SnapshotChecksums {
                md5: Some(format!("{:x}", md5_hash(data))),
                sha256: Some(sha256_hash(data)),
                crc32: Some(crc32_hash(data)),
            }
        } else {
            SnapshotChecksums {
                md5: None,
                sha256: None,
                crc32: None,
            }
        };

        let snapshot = ProcessSnapshot {
            id: id.clone(),
            instance_id,
            created_at: Utc::now(),
            snapshot_type,
            metadata,
            state_data,
            checksums,
        };

        let mut snapshots = self.snapshots.write().await;
        snapshots.push_back(snapshot.clone());

        if snapshots.len() > self.config.max_snapshots {
            snapshots.pop_front();
        }

        if self.config.compress_snapshots {
            self.save_snapshot_to_disk(&snapshot)?;
        }

        info!("Created snapshot: {} (type: {:?})", id, snapshot_type);
        Ok(snapshot)
    }

    fn save_snapshot_to_disk(&self, snapshot: &ProcessSnapshot) -> Result<(), ProcessError> {
        let path = self.config.storage_path.join(format!("{}.json", snapshot.id));

        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| ProcessError::SerializationError(e))?;

        fs::write(&path, json)
            .map_err(|e| ProcessError::IoError(e))?;

        info!("Saved snapshot to disk: {}", path.display());
        Ok(())
    }

    pub async fn get_snapshot(&self, id: &str) -> Option<ProcessSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.iter().find(|s| s.id == id).cloned()
    }

    pub async fn list_snapshots(&self) -> Vec<ProcessSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.iter().cloned().collect()
    }

    pub async fn list_snapshots_by_instance(&self, instance_id: &str) -> Vec<ProcessSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots
            .iter()
            .filter(|s| s.instance_id == instance_id)
            .cloned()
            .collect()
    }

    pub async fn delete_snapshot(&self, id: &str) -> Result<(), ProcessError> {
        let mut snapshots = self.snapshots.write().await;
        let pos = snapshots
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| ProcessError::SnapshotError(format!("Snapshot not found: {}", id)))?;

        snapshots.remove(pos);

        let path = self.config.storage_path.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| ProcessError::IoError(e))?;
        }

        info!("Deleted snapshot: {}", id);
        Ok(())
    }

    pub async fn cleanup_old_snapshots(&self) -> Result<usize, ProcessError> {
        let mut snapshots = self.snapshots.write().await;
        let initial_len = snapshots.len();

        while snapshots.len() > self.config.max_snapshots {
            if let Some(snapshot) = snapshots.pop_front() {
                let path = self.config.storage_path.join(format!("{}.json", snapshot.id));
                if path.exists() {
                    let _ = fs::remove_file(&path);
                }
            }
        }

        let removed = initial_len - snapshots.len();
        info!("Cleaned up {} old snapshots", removed);
        Ok(removed)
    }

    pub async fn restore_snapshot(&self, id: &str) -> Result<ProcessSnapshot, ProcessError> {
        let snapshot = self
            .get_snapshot(id)
            .await
            .ok_or_else(|| ProcessError::SnapshotError(format!("Snapshot not found: {}", id)))?;

        if snapshot.snapshot_type == SnapshotType::Full {
            info!("Restoring full snapshot: {}", id);
        } else {
            info!("Restoring {} snapshot: {}", snapshot.snapshot_type.as_str(), id);
        }

        Ok(snapshot)
    }

    pub async fn verify_snapshot(&self, id: &str) -> Result<bool, ProcessError> {
        let snapshot = self
            .get_snapshot(id)
            .await
            .ok_or_else(|| ProcessError::SnapshotError(format!("Snapshot not found: {}", id)))?;

        if let Some(ref data) = snapshot.state_data {
            if let Some(ref expected_md5) = snapshot.checksums.md5 {
                let actual_md5 = format!("{:x}", md5_hash(data));
                if actual_md5 != *expected_md5 {
                    warn!(
                        "Snapshot {} MD5 checksum mismatch: expected {}, got {}",
                        id, expected_md5, actual_md5
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    pub fn get_config(&self) -> SnapshotConfig {
        self.config.clone()
    }

    pub async fn load_snapshots_from_disk(&self) -> Result<usize, ProcessError> {
        if !self.config.storage_path.exists() {
            return Ok(0);
        }

        let entries = fs::read_dir(&self.config.storage_path)
            .map_err(|e| ProcessError::IoError(e))?;

        let mut loaded = 0;
        let mut snapshots = self.snapshots.write().await;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(snapshot) = serde_json::from_str::<ProcessSnapshot>(&content) {
                        snapshots.push_back(snapshot);
                        loaded += 1;
                    }
                }
            }
        }

        info!("Loaded {} snapshots from disk", loaded);
        Ok(loaded)
    }
}

fn md5_hash(data: &[u8]) -> String {
    let mut hash: u64 = 0;
    for (i, &byte) in data.iter().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul((i as u64).wrapping_add(1)));
        hash = hash.rotate_left(3);
    }
    format!("{:016x}", hash)
}

fn sha256_hash(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn crc32_hash(data: &[u8]) -> u32 {
    let mut crc = 0u32;
    for byte in data {
        crc = crc ^ (*byte as u32);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

impl SnapshotType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnapshotType::Full => "full",
            SnapshotType::Incremental => "incremental",
            SnapshotType::StateOnly => "state_only",
            SnapshotType::MemoryDump => "memory_dump",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> SnapshotMetadata {
        SnapshotMetadata {
            pid: 12345,
            uptime_secs: 3600,
            memory_usage_mb: 2048,
            cpu_percent: 45.5,
            world_name: Some("world".to_string()),
            player_count: 5,
            tick_rate: 20.0,
            description: "Test snapshot".to_string(),
        }
    }

    #[tokio::test]
    async fn test_snapshot_manager_creation() {
        let config = SnapshotConfig::default();
        let manager = SnapshotManager::new(config);

        let snapshots = manager.list_snapshots().await;
        assert!(snapshots.is_empty());
    }

    #[tokio::test]
    async fn test_create_snapshot() {
        let config = SnapshotConfig {
            storage_path: PathBuf::from("/tmp/test_snapshots"),
            ..Default::default()
        };
        let manager = SnapshotManager::new(config);

        let metadata = create_test_metadata();
        let state_data = Some(vec![1, 2, 3, 4, 5]);

        let snapshot = manager
            .create_snapshot(
                "test-instance".to_string(),
                SnapshotType::Full,
                metadata,
                state_data,
            )
            .await
            .unwrap();

        assert!(!snapshot.id.is_empty());
        assert_eq!(snapshot.snapshot_type, SnapshotType::Full);
        assert!(snapshot.checksums.md5.is_some());
    }

    #[tokio::test]
    async fn test_get_snapshot() {
        let config = SnapshotConfig::default();
        let manager = SnapshotManager::new(config);

        let metadata = create_test_metadata();
        let snapshot = manager
            .create_snapshot(
                "test-instance".to_string(),
                SnapshotType::Full,
                metadata,
                None,
            )
            .await
            .unwrap();

        let retrieved = manager.get_snapshot(&snapshot.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, snapshot.id);
    }

    #[tokio::test]
    async fn test_delete_snapshot() {
        let config = SnapshotConfig::default();
        let manager = SnapshotManager::new(config);

        let metadata = create_test_metadata();
        let snapshot = manager
            .create_snapshot(
                "test-instance".to_string(),
                SnapshotType::Full,
                metadata,
                None,
            )
            .await
            .unwrap();

        assert!(manager.delete_snapshot(&snapshot.id).await.is_ok());
        assert!(manager.get_snapshot(&snapshot.id).await.is_none());
    }

    #[tokio::test]
    async fn test_list_snapshots_by_instance() {
        let config = SnapshotConfig::default();
        let manager = SnapshotManager::new(config);

        let metadata = create_test_metadata();

        manager
            .create_snapshot(
                "instance1".to_string(),
                SnapshotType::Full,
                metadata.clone(),
                None,
            )
            .await
            .unwrap();

        manager
            .create_snapshot(
                "instance2".to_string(),
                SnapshotType::Full,
                metadata,
                None,
            )
            .await
            .unwrap();

        let instance1_snapshots = manager.list_snapshots_by_instance("instance1").await;
        assert_eq!(instance1_snapshots.len(), 1);
    }

    #[tokio::test]
    async fn test_cleanup_old_snapshots() {
        let config = SnapshotConfig {
            max_snapshots: 2,
            ..Default::default()
        };
        let manager = SnapshotManager::new(config);

        let metadata = create_test_metadata();

        for i in 0..5 {
            let mut metadata = metadata.clone();
            metadata.description = format!("Snapshot {}", i);
            manager
                .create_snapshot(
                    "test-instance".to_string(),
                    SnapshotType::Full,
                    metadata,
                    None,
                )
                .await
                .unwrap();
        }

        let removed = manager.cleanup_old_snapshots().await.unwrap();
        assert_eq!(removed, 3);

        let snapshots = manager.list_snapshots().await;
        assert_eq!(snapshots.len(), 2);
    }

    #[tokio::test]
    async fn test_verify_snapshot() {
        let config = SnapshotConfig::default();
        let manager = SnapshotManager::new(config);

        let metadata = create_test_metadata();
        let state_data = vec![1, 2, 3, 4, 5];

        let snapshot = manager
            .create_snapshot(
                "test-instance".to_string(),
                SnapshotType::Full,
                metadata,
                Some(state_data),
            )
            .await
            .unwrap();

        let valid = manager.verify_snapshot(&snapshot.id).await.unwrap();
        assert!(valid);
    }
}

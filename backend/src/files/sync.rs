use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use uuid::Uuid;

pub struct SyncService {
    server_root: PathBuf,
    configs: RwLock<HashMap<String, SyncConfig>>,
    sync_history: RwLock<HashMap<String, Vec<SyncEvent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub id: String,
    pub name: String,
    pub source: SyncEndpoint,
    pub target: SyncEndpoint,
    pub direction: SyncDirection,
    pub auto_sync: bool,
    pub sync_interval_seconds: u64,
    pub last_sync: Option<String>,
    pub status: SyncStatus,
    pub filters: SyncFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEndpoint {
    pub protocol: SyncProtocol,
    pub host: String,
    pub port: u16,
    pub base_path: String,
    pub credentials: Option<SyncCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncProtocol {
    SFTP,
    WebDAV,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCredentials {
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFilters {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub id: String,
    pub config_id: String,
    pub timestamp: String,
    pub event_type: SyncEventType,
    pub files_affected: Vec<SyncFileChange>,
    pub duration_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEventType {
    Started,
    Completed,
    Failed,
    FileAdded,
    FileModified,
    FileDeleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncFileChange {
    pub path: String,
    pub change_type: String,
    pub size: u64,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPreview {
    pub to_upload: Vec<SyncFileChange>,
    pub to_download: Vec<SyncFileChange>,
    pub to_delete: Vec<String>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub path: String,
    pub source_modified: String,
    pub target_modified: String,
    pub source_size: u64,
    pub target_size: u64,
}

impl SyncService {
    pub fn new(server_root: PathBuf) -> Self {
        Self {
            server_root,
            configs: RwLock::new(HashMap::new()),
            sync_history: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_config(&self, config: SyncConfigRequest) -> Result<SyncConfig> {
        let sync_config = SyncConfig {
            id: Uuid::new_v4().to_string(),
            name: config.name,
            source: config.source,
            target: config.target,
            direction: config.direction,
            auto_sync: config.auto_sync,
            sync_interval_seconds: config.sync_interval_seconds,
            last_sync: None,
            status: SyncStatus::Idle,
            filters: config.filters,
        };

        {
            let mut guard = self.configs.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.insert(sync_config.id.clone(), sync_config.clone());
        }

        Ok(sync_config)
    }

    pub fn update_config(&self, config_id: &str, update: SyncConfigUpdate) -> Result<SyncConfig> {
        let mut guard = self.configs.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let config = guard.get_mut(config_id)
            .ok_or_else(|| anyhow::anyhow!("Sync config not found: {}", config_id))?;

        if let Some(name) = update.name {
            config.name = name;
        }
        if let Some(source) = update.source {
            config.source = source;
        }
        if let Some(target) = update.target {
            config.target = target;
        }
        if let Some(direction) = update.direction {
            config.direction = direction;
        }
        if let Some(auto_sync) = update.auto_sync {
            config.auto_sync = auto_sync;
        }
        if let Some(interval) = update.sync_interval_seconds {
            config.sync_interval_seconds = interval;
        }
        if let Some(filters) = update.filters {
            config.filters = filters;
        }

        Ok(config.clone())
    }

    pub fn delete_config(&self, config_id: &str) -> Result<()> {
        let mut guard = self.configs.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        guard.remove(config_id)
            .ok_or_else(|| anyhow::anyhow!("Sync config not found: {}", config_id))?;
        Ok(())
    }

    pub fn list_configs(&self) -> Result<Vec<SyncConfig>> {
        let guard = self.configs.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(guard.values().cloned().collect())
    }

    pub fn get_config(&self, config_id: &str) -> Result<SyncConfig> {
        let guard = self.configs.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        guard.get(config_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Sync config not found: {}", config_id))
    }

    pub fn preview_sync(&self, config_id: &str) -> Result<SyncPreview> {
        let config = self.get_config(config_id)?;
        
        let source_path = if config.source.protocol == SyncProtocol::Local {
            self.server_root.join(&config.source.base_path)
        } else {
            self.server_root.join(".sync").join(&config.id)
        };

        let target_path = if config.target.protocol == SyncProtocol::Local {
            self.server_root.join(&config.target.base_path)
        } else {
            self.server_root.join(".sync").join(&config.id).join("remote")
        };

        let source_files = self.scan_directory(&source_path, &config.filters)?;
        let target_files = if target_path.exists() {
            self.scan_directory(&target_path, &config.filters)?
        } else {
            Vec::new()
        };

        let mut to_upload = Vec::new();
        let mut to_download = Vec::new();
        let mut to_delete = Vec::new();
        let mut conflicts = Vec::new();

        let source_map: HashMap<String, SyncFileChange> = source_files
            .into_iter()
            .map(|f| (f.path.clone(), f))
            .collect();

        let target_map: HashMap<String, SyncFileChange> = target_files
            .into_iter()
            .map(|f| (f.path.clone(), f))
            .collect();

        match config.direction {
            SyncDirection::Push => {
                for (path, source_file) in &source_map {
                    if let Some(target_file) = target_map.get(path) {
                        if source_file.checksum != target_file.checksum {
                            if source_file.checksum.is_some() && target_file.checksum.is_some() {
                                conflicts.push(SyncConflict {
                                    path: path.clone(),
                                    source_modified: source_file.checksum.clone().unwrap_or_default(),
                                    target_modified: target_file.checksum.clone().unwrap_or_default(),
                                    source_size: source_file.size,
                                    target_size: target_file.size,
                                });
                            } else {
                                to_upload.push(source_file.clone());
                            }
                        }
                    } else {
                        to_upload.push(source_file.clone());
                    }
                }

                for path in target_map.keys() {
                    if !source_map.contains_key(path) {
                        to_delete.push(path.clone());
                    }
                }
            }
            SyncDirection::Pull => {
                for (path, target_file) in &target_map {
                    if let Some(source_file) = source_map.get(path) {
                        if source_file.checksum != target_file.checksum {
                            if source_file.checksum.is_some() && target_file.checksum.is_some() {
                                conflicts.push(SyncConflict {
                                    path: path.clone(),
                                    source_modified: source_file.checksum.clone().unwrap_or_default(),
                                    target_modified: target_file.checksum.clone().unwrap_or_default(),
                                    source_size: source_file.size,
                                    target_size: target_file.size,
                                });
                            } else {
                                to_download.push(target_file.clone());
                            }
                        }
                    } else {
                        to_download.push(target_file.clone());
                    }
                }

                for path in source_map.keys() {
                    if !target_map.contains_key(path) {
                        to_delete.push(path.clone());
                    }
                }
            }
            SyncDirection::Bidirectional => {
                for (path, source_file) in &source_map {
                    if let Some(target_file) = target_map.get(path) {
                        if source_file.checksum != target_file.checksum {
                            conflicts.push(SyncConflict {
                                path: path.clone(),
                                source_modified: source_file.checksum.clone().unwrap_or_default(),
                                target_modified: target_file.checksum.clone().unwrap_or_default(),
                                source_size: source_file.size,
                                target_size: target_file.size,
                            });
                        }
                    } else {
                        to_upload.push(source_file.clone());
                    }
                }

                for (path, target_file) in &target_map {
                    if !source_map.contains_key(path) {
                        to_download.push(target_file.clone());
                    }
                }
            }
        }

        Ok(SyncPreview {
            to_upload,
            to_download,
            to_delete,
            conflicts,
        })
    }

    pub fn execute_sync(&self, config_id: &str) -> Result<SyncEvent> {
        let config = self.get_config(config_id)?;
        
        let start_time = std::time::Instant::now();
        let event_id = Uuid::new_v4().to_string();

        {
            let mut guard = self.configs.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            if let Some(c) = guard.get_mut(config_id) {
                c.status = SyncStatus::Syncing;
            }
        }

        let result = self.perform_sync(&config);

        let event = match result {
            Ok(files) => {
                let duration = start_time.elapsed().as_millis() as u64;
                
                {
                    let mut guard = self.configs.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
                    if let Some(c) = guard.get_mut(config_id) {
                        c.status = SyncStatus::Completed;
                        c.last_sync = Some(chrono::Utc::now().to_rfc3339());
                    }
                }

                SyncEvent {
                    id: event_id,
                    config_id: config_id.to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    event_type: SyncEventType::Completed,
                    files_affected: files,
                    duration_ms: duration,
                    success: true,
                    error_message: None,
                }
            }
            Err(e) => {
                {
                    let mut guard = self.configs.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
                    if let Some(c) = guard.get_mut(config_id) {
                        c.status = SyncStatus::Error;
                    }
                }

                SyncEvent {
                    id: event_id,
                    config_id: config_id.to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    event_type: SyncEventType::Failed,
                    files_affected: vec![],
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    success: false,
                    error_message: Some(e.to_string()),
                }
            }
        };

        {
            let mut guard = self.sync_history.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.entry(config_id.to_string())
                .or_insert_with(Vec::new)
                .push(event.clone());
        }

        Ok(event)
    }

    fn perform_sync(&self, config: &SyncConfig) -> Result<Vec<SyncFileChange>> {
        let preview = self.preview_sync(&config.id)?;
        let mut affected = Vec::new();

        for file in preview.to_upload {
            affected.push(file.clone());
        }
        for file in preview.to_download {
            affected.push(file.clone());
        }

        Ok(affected)
    }

    fn scan_directory(&self, path: &PathBuf, filters: &SyncFilters) -> Result<Vec<SyncFileChange>> {
        let mut files = Vec::new();

        if !path.exists() {
            return Ok(files);
        }

        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            
            if entry_path.is_file() {
                let relative = entry_path.strip_prefix(path)
                    .unwrap_or(entry_path)
                    .to_string_lossy()
                    .to_string();

                if Self::matches_filters(&relative, filters) {
                    let metadata = fs::metadata(entry_path)?;
                    
                    let checksum = {
                        use sha2::{Sha256, Digest};
                        let content = fs::read(entry_path)?;
                        let mut hasher = Sha256::new();
                        hasher.update(&content);
                        Some(hex::encode(hasher.finalize()))
                    };

                    files.push(SyncFileChange {
                        path: relative,
                        change_type: "file".to_string(),
                        size: metadata.len(),
                        checksum,
                    });
                }
            }
        }

        Ok(files)
    }

    fn matches_filters(path: &str, filters: &SyncFilters) -> bool {
        if let Some(max_size) = filters.max_file_size {
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.len() > max_size {
                    return false;
                }
            }
        }

        for pattern in &filters.exclude_patterns {
            if Self::matches_pattern(path, pattern) {
                return false;
            }
        }

        if !filters.include_patterns.is_empty() {
            for pattern in &filters.include_patterns {
                if Self::matches_pattern(path, pattern) {
                    return true;
                }
            }
            return false;
        }

        true
    }

    fn matches_pattern(path: &str, pattern: &str) -> bool {
        let pattern = pattern.replace("**", "\0");
        let path = path.replace("**", "\0");
        
        if pattern.starts_with("*.") {
            let ext = &pattern[2..];
            return path.ends_with(&format!(".{}", ext));
        }

        if pattern.starts_with('/') || pattern.starts_with("./") {
            let trimmed = pattern.trim_start_matches("./");
            return path.starts_with(trimmed) || path.contains(trimmed);
        }
        
        path.contains(pattern)
    }

    pub fn get_sync_history(&self, config_id: &str) -> Result<Vec<SyncEvent>> {
        let guard = self.sync_history.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        Ok(guard.get(config_id).cloned().unwrap_or_default())
    }

    pub fn resolve_conflict(&self, config_id: &str, conflict: &SyncConflict, resolution: ConflictResolution) -> Result<()> {
        match resolution {
            ConflictResolution::KeepSource => {
                tracing::info!("Keeping source version for conflict at {}", conflict.path);
            }
            ConflictResolution::KeepTarget => {
                tracing::info!("Keeping target version for conflict at {}", conflict.path);
            }
            ConflictResolution::KeepBoth => {
                tracing::info!("Keeping both versions for conflict at {}", conflict.path);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfigRequest {
    pub name: String,
    pub source: SyncEndpoint,
    pub target: SyncEndpoint,
    pub direction: SyncDirection,
    pub auto_sync: bool,
    pub sync_interval_seconds: u64,
    pub filters: SyncFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfigUpdate {
    pub name: Option<String>,
    pub source: Option<SyncEndpoint>,
    pub target: Option<SyncEndpoint>,
    pub direction: Option<SyncDirection>,
    pub auto_sync: Option<bool>,
    pub sync_interval_seconds: Option<u64>,
    pub filters: Option<SyncFilters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    KeepSource,
    KeepTarget,
    KeepBoth,
}

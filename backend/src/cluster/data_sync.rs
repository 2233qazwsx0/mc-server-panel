use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

#[derive(Clone)]
pub struct DataSyncManager {
    state: Arc<DataSyncState>,
    config: Arc<RwLock<DataSyncConfig>>,
}

struct DataSyncState {
    sync_jobs: RwLock<HashMap<String, SyncJob>>,
    sync_queue: RwLock<VecDeque<SyncTask>>,
    last_sync: RwLock<HashMap<String, DateTime<Utc>>>,
    conflicts: RwLock<Vec<SyncConflict>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSyncConfig {
    pub enabled: bool,
    pub sync_interval_secs: u64,
    pub sync_types: Vec<DataSyncType>,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub conflict_resolution: ConflictResolution,
    pub batch_size: u32,
    pub max_concurrent_syncs: u32,
}

impl Default for DataSyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_interval_secs: 300,
            sync_types: vec![DataSyncType::PlayerData, DataSyncType::Permissions],
            compression_enabled: true,
            encryption_enabled: false,
            conflict_resolution: ConflictResolution::LastWriteWins,
            batch_size: 100,
            max_concurrent_syncs: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataSyncType {
    PlayerData,
    Permissions,
    Economy,
    Inventories,
    Homes,
    Stats,
    Cosmetics,
    Rankings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    LastWriteWins,
    FirstWriteWins,
    Merge,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub id: String,
    pub sync_type: DataSyncType,
    pub source_node: String,
    pub target_nodes: Vec<String>,
    pub status: SyncJobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_records: u64,
    pub synced_records: u64,
    pub failed_records: u64,
    pub progress_percent: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncJobStatus {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTask {
    pub id: String,
    pub job_id: String,
    pub data_type: DataSyncType,
    pub record_id: String,
    pub data: serde_json::Value,
    pub source_node: String,
    pub target_node: String,
    pub priority: SyncPriority,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SyncPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub id: String,
    pub data_type: DataSyncType,
    pub record_id: String,
    pub local_value: serde_json::Value,
    pub remote_value: serde_json::Value,
    pub local_timestamp: DateTime<Utc>,
    pub remote_timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub resolution: Option<ConflictResolution>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDataSync {
    pub player_uuid: String,
    pub player_name: String,
    pub last_server: String,
    pub last_login: DateTime<Utc>,
    pub play_time_secs: u64,
    pub logout_location: Option<serde_json::Value>,
    pub inventory: Option<serde_json::Value>,
    pub ender_chest: Option<serde_json::Value>,
    pub health: f32,
    pub hunger: i32,
    pub experience: i32,
    pub gamemode: String,
}

impl DataSyncManager {
    pub fn new(config: DataSyncConfig) -> Self {
        Self {
            state: Arc::new(DataSyncState {
                sync_jobs: RwLock::new(HashMap::new()),
                sync_queue: RwLock::new(VecDeque::new()),
                last_sync: RwLock::new(HashMap::new()),
                conflicts: RwLock::new(Vec::new()),
            }),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn update_config(&self, config: DataSyncConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> DataSyncConfig {
        self.config.read().clone()
    }

    pub fn create_sync_job(&self, sync_type: DataSyncType, source: &str, targets: Vec<String>) -> SyncJob {
        let job = SyncJob {
            id: Uuid::new_v4().to_string(),
            sync_type,
            source_node: source.to_string(),
            target_nodes: targets,
            status: SyncJobStatus::Queued,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            total_records: 0,
            synced_records: 0,
            failed_records: 0,
            progress_percent: 0.0,
            error: None,
        };

        self.state.sync_jobs.write().insert(job.id.clone(), job.clone());
        job
    }

    pub fn start_sync_job(&self, job_id: &str) -> Result<(), SyncError> {
        let mut jobs = self.state.sync_jobs.write();
        let job = jobs.get_mut(job_id)
            .ok_or_else(|| SyncError::JobNotFound(job_id.to_string()))?;

        if job.status != SyncJobStatus::Queued {
            return Err(SyncError::InvalidJobState(job_id.to_string()));
        }

        job.status = SyncJobStatus::Running;
        job.started_at = Some(Utc::now());
        Ok(())
    }

    pub fn update_job_progress(&self, job_id: &str, synced: u64, failed: u64) {
        let mut jobs = self.state.sync_jobs.write();
        if let Some(job) = jobs.get_mut(job_id) {
            job.synced_records = synced;
            job.failed_records = failed;
            if job.total_records > 0 {
                job.progress_percent = (synced as f64 / job.total_records as f64) * 100.0;
            }
        }
    }

    pub fn complete_job(&self, job_id: &str, success: bool, error: Option<String>) {
        let mut jobs = self.state.sync_jobs.write();
        if let Some(job) = jobs.get_mut(job_id) {
            job.completed_at = Some(Utc::now());
            job.status = if success { SyncJobStatus::Completed } else { SyncJobStatus::Failed };
            job.error = error;

            if success {
                let mut last_sync = self.state.last_sync.write();
                last_sync.insert(job.source_node.clone(), Utc::now());
            }
        }
    }

    pub fn get_job(&self, job_id: &str) -> Option<SyncJob> {
        self.state.sync_jobs.read().get(job_id).cloned()
    }

    pub fn get_active_jobs(&self) -> Vec<SyncJob> {
        self.state.sync_jobs.read()
            .values()
            .filter(|j| matches!(j.status, SyncJobStatus::Queued | SyncJobStatus::Running | SyncJobStatus::Paused))
            .cloned()
            .collect()
    }

    pub fn get_job_history(&self, limit: usize) -> Vec<SyncJob> {
        let jobs: Vec<SyncJob> = self.state.sync_jobs.read().values().cloned().collect();
        let mut sorted: Vec<_> = jobs.into_iter()
            .filter(|j| matches!(j.status, SyncJobStatus::Completed | SyncJobStatus::Failed | SyncJobStatus::Cancelled))
            .collect();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted.truncate(limit);
        sorted
    }

    pub fn queue_sync_task(&self, task: SyncTask) {
        let mut queue = self.state.sync_queue.write();
        let mut tasks: Vec<_> = queue.iter().cloned().collect();
        let insert_pos = tasks.iter().position(|t| t.priority < task.priority).unwrap_or(tasks.len());
        queue.insert(insert_pos, task);
    }

    pub fn get_next_task(&self) -> Option<SyncTask> {
        self.state.sync_queue.write().pop_front()
    }

    pub fn add_pending_tasks(&self, job_id: &str, records: Vec<(String, serde_json::Value)>) -> u32 {
        let mut count = 0;
        let mut queue = self.state.sync_queue.write();

        for (record_id, data) in records {
            let task = SyncTask {
                id: Uuid::new_v4().to_string(),
                job_id: job_id.to_string(),
                data_type: DataSyncType::PlayerData,
                record_id,
                data,
                source_node: "local".to_string(),
                target_node: "all".to_string(),
                priority: SyncPriority::Normal,
                created_at: Utc::now(),
            };
            queue.push_back(task);
            count += 1;
        }

        count
    }

    pub fn create_conflict(&self, conflict: SyncConflict) {
        let mut conflicts = self.state.conflicts.write();
        conflicts.push(conflict);
    }

    pub fn resolve_conflict(&self, conflict_id: &str, resolution: ConflictResolution, resolved_by: &str, value: Option<serde_json::Value>) -> Result<(), SyncError> {
        let mut conflicts = self.state.conflicts.write();
        if let Some(conflict) = conflicts.iter_mut().find(|c| c.id == conflict_id) {
            conflict.resolved = true;
            conflict.resolution = Some(resolution);
            conflict.resolved_by = Some(resolved_by.to_string());

            if resolution == ConflictResolution::Manual {
                conflict.local_value = value.unwrap_or(conflict.local_value.clone());
            }
            Ok(())
        } else {
            Err(SyncError::ConflictNotFound(conflict_id.to_string()))
        }
    }

    pub fn get_unresolved_conflicts(&self) -> Vec<SyncConflict> {
        self.state.conflicts.read()
            .iter()
            .filter(|c| !c.resolved)
            .cloned()
            .collect()
    }

    pub fn get_last_sync_time(&self, node_id: &str) -> Option<DateTime<Utc>> {
        self.state.last_sync.read().get(node_id).cloned()
    }

    pub fn get_stats(&self) -> DataSyncStats {
        let jobs = self.state.sync_jobs.read();
        let queue = self.state.sync_queue.read();
        let conflicts = self.state.conflicts.read();

        let total_jobs = jobs.len();
        let active_jobs = jobs.values().filter(|j| matches!(j.status, SyncJobStatus::Running | SyncJobStatus::Queued)).count();
        let queued_tasks = queue.len();
        let unresolved_conflicts = conflicts.iter().filter(|c| !c.resolved).count();

        DataSyncStats {
            total_jobs,
            active_jobs,
            queued_tasks,
            unresolved_conflicts,
            compression_enabled: self.config.read().compression_enabled,
            encryption_enabled: self.config.read().encryption_enabled,
        }
    }

    pub fn sync_player_data(&self, player_data: PlayerDataSync, targets: Vec<String>) -> Result<SyncJob, SyncError> {
        let config = self.config.read();
        if !config.enabled || !config.sync_types.contains(&DataSyncType::PlayerData) {
            return Err(SyncError::SyncDisabled);
        }

        let job = self.create_sync_job(DataSyncType::PlayerData, "local", targets);
        self.start_sync_job(&job.id)?;

        Ok(job)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSyncStats {
    pub total_jobs: usize,
    pub active_jobs: usize,
    pub queued_tasks: usize,
    pub unresolved_conflicts: usize,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Sync job {0} not found")]
    JobNotFound(String),

    #[error("Invalid job state for job {0}")]
    InvalidJobState(String),

    #[error("Sync conflict {0} not found")]
    ConflictNotFound(String),

    #[error("Data sync is disabled")]
    SyncDisabled,
}

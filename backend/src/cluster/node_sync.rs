use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::HashMap;

use crate::cluster::types::*;

#[derive(Clone)]
pub struct NodeSyncManager {
    state: Arc<NodeSyncState>,
    heartbeat_interval_secs: u64,
    stale_threshold_secs: u64,
}

struct NodeSyncState {
    local_node: RwLock<Option<NodeInfo>>,
    remote_nodes: RwLock<HashMap<String, NodeInfo>>,
    pending_syncs: RwLock<Vec<SyncTask>>,
    sync_history: RwLock<Vec<SyncRecord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTask {
    pub id: String,
    pub task_type: SyncTaskType,
    pub source_node: String,
    pub target_nodes: Vec<String>,
    pub data: serde_json::Value,
    pub priority: SyncPriority,
    pub created_at: DateTime<Utc>,
    pub status: SyncTaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncTaskType {
    ConfigSync,
    PlayerDataSync,
    ChatSync,
    WorldDataSync,
    MetricsSync,
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SyncPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncTaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub task_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub source_node: String,
    pub target_nodes: Vec<String>,
    pub records_synced: u64,
    pub status: SyncTaskStatus,
    pub error: Option<String>,
}

impl NodeSyncManager {
    pub fn new(heartbeat_interval_secs: u64, stale_threshold_secs: u64) -> Self {
        Self {
            state: Arc::new(NodeSyncState {
                local_node: RwLock::new(None),
                remote_nodes: RwLock::new(HashMap::new()),
                pending_syncs: RwLock::new(Vec::new()),
                sync_history: RwLock::new(Vec::new()),
            }),
            heartbeat_interval_secs,
            stale_threshold_secs,
        }
    }

    pub fn register_local_node(&self, name: String, host: String, port: u16) -> NodeInfo {
        let node = NodeInfo::new(
            Uuid::new_v4().to_string(),
            name,
            host,
            port,
        );
        *self.state.local_node.write() = Some(node.clone());
        node
    }

    pub fn get_local_node(&self) -> Option<NodeInfo> {
        self.state.local_node.read().clone()
    }

    pub fn update_local_metrics(&self, cpu: f64, memory: f64, tps: f64, players: u32) {
        let mut node = self.state.local_node.write();
        if let Some(ref mut n) = *node {
            n.cpu_usage = cpu;
            n.memory_usage = memory;
            n.tps = tps;
            n.player_count = players;
            n.last_heartbeat = Utc::now();
        }
    }

    pub fn register_remote_node(&self, node: NodeInfo) {
        let mut nodes = self.state.remote_nodes.write();
        nodes.insert(node.id.clone(), node);
    }

    pub fn unregister_remote_node(&self, node_id: &str) -> Option<NodeInfo> {
        let mut nodes = self.state.remote_nodes.write();
        nodes.remove(node_id)
    }

    pub fn get_remote_node(&self, node_id: &str) -> Option<NodeInfo> {
        self.state.remote_nodes.read().get(node_id).cloned()
    }

    pub fn get_all_nodes(&self) -> Vec<NodeInfo> {
        let mut nodes = Vec::new();
        if let Some(local) = self.state.local_node.read().clone() {
            nodes.push(local);
        }
        nodes.extend(self.state.remote_nodes.read().values().cloned());
        nodes
    }

    pub fn get_healthy_nodes(&self) -> Vec<NodeInfo> {
        self.get_all_nodes()
            .into_iter()
            .filter(|n| n.is_healthy())
            .collect()
    }

    pub fn process_heartbeat(&self, node_id: &str, heartbeat: HeartbeatData) -> HeartbeatResponse {
        let mut nodes = self.state.remote_nodes.write();

        if let Some(node) = nodes.get_mut(node_id) {
            node.last_heartbeat = Utc::now();
            node.cpu_usage = heartbeat.cpu_usage;
            node.memory_usage = heartbeat.memory_usage;
            node.player_count = heartbeat.player_count;
            node.tps = heartbeat.tps;
            node.status = heartbeat.status;

            let response = HeartbeatResponse {
                accepted: true,
                cluster_time: Utc::now(),
                config_version: "1.0.0".to_string(),
                sync_required: heartbeat.config_version != "1.0.0",
                pending_players: Vec::new(),
            };

            drop(nodes);
            response
        } else {
            HeartbeatResponse {
                accepted: false,
                cluster_time: Utc::now(),
                config_version: "1.0.0".to_string(),
                sync_required: false,
                pending_players: Vec::new(),
            }
        }
    }

    pub fn detect_stale_nodes(&self) -> Vec<NodeInfo> {
        let threshold = Utc::now() - Duration::seconds(self.stale_threshold_secs as i64);
        self.state.remote_nodes.read()
            .values()
            .filter(|n| n.last_heartbeat < threshold)
            .cloned()
            .collect()
    }

    pub fn create_sync_task(&self, task_type: SyncTaskType, source: &str, targets: Vec<String>, data: serde_json::Value) -> SyncTask {
        let task = SyncTask {
            id: Uuid::new_v4().to_string(),
            task_type,
            source_node: source.to_string(),
            target_nodes: targets,
            data,
            priority: SyncPriority::Normal,
            created_at: Utc::now(),
            status: SyncTaskStatus::Pending,
        };

        self.state.pending_syncs.write().push(task.clone());
        task
    }

    pub fn get_pending_syncs(&self) -> Vec<SyncTask> {
        self.state.pending_syncs.read().clone()
    }

    pub fn update_sync_status(&self, task_id: &str, status: SyncTaskStatus) -> Result<(), SyncError> {
        let mut syncs = self.state.pending_syncs.write();
        if let Some(task) = syncs.iter_mut().find(|t| t.id == task_id) {
            task.status = status.clone();
            if matches!(status, SyncTaskStatus::Completed | SyncTaskStatus::Failed) {
                let record = SyncRecord {
                    task_id: task.id.clone(),
                    started_at: task.created_at,
                    completed_at: Some(Utc::now()),
                    source_node: task.source_node.clone(),
                    target_nodes: task.target_nodes.clone(),
                    records_synced: 0,
                    status,
                    error: None,
                };
                drop(syncs);
                self.state.sync_history.write().push(record);
                self.state.pending_syncs.write().retain(|t| t.id != task_id);
            }
            Ok(())
        } else {
            Err(SyncError::TaskNotFound(task_id.to_string()))
        }
    }

    pub fn get_sync_history(&self, limit: usize) -> Vec<SyncRecord> {
        let history = self.state.sync_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn broadcast_to_all(&self, message: ClusterMessage) -> Vec<String> {
        let node_ids: Vec<String> = self.state.remote_nodes.read().keys().cloned().collect();
        for node_id in &node_ids {
            tracing::debug!("Broadcasting message to node: {}", node_id);
        }
        node_ids
    }

    pub fn send_to_node(&self, node_id: &str, message: ClusterMessage) -> Result<(), SyncError> {
        if self.state.remote_nodes.read().contains_key(node_id) {
            tracing::debug!("Sending message to node {}: {:?}", node_id, message.message_type);
            Ok(())
        } else {
            Err(SyncError::NodeNotFound(node_id.to_string()))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatData {
    pub node_id: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub player_count: u32,
    pub tps: f64,
    pub status: NodeStatus,
    pub config_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub accepted: bool,
    pub cluster_time: DateTime<Utc>,
    pub config_version: String,
    pub sync_required: bool,
    pub pending_players: Vec<PendingPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPlayer {
    pub player_name: String,
    pub from_server: String,
    pub to_server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMessage {
    pub message_type: ClusterMessageType,
    pub source_node: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterMessageType {
    PlayerTransfer,
    ChatMessage,
    ConfigUpdate,
    NodeJoin,
    NodeLeave,
    MaintenanceStart,
    MaintenanceEnd,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Node {0} not found")]
    NodeNotFound(String),

    #[error("Sync task {0} not found")]
    TaskNotFound(String),

    #[error("Sync failed: {0}")]
    SyncFailed(String),
}

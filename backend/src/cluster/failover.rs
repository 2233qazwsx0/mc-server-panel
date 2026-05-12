use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

#[derive(Clone)]
pub struct FailoverManager {
    state: Arc<FailoverState>,
    config: Arc<RwLock<FailoverConfig>>,
}

struct FailoverState {
    active_failovers: RwLock<HashMap<String, FailoverRecord>>,
    failover_history: RwLock<VecDeque<FailoverRecord>>,
    node_health: RwLock<HashMap<String, NodeHealth>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    pub enabled: bool,
    pub auto_failover: bool,
    pub health_check_interval_secs: u64,
    pub health_check_timeout_secs: u64,
    pub unhealthy_threshold: u32,
    pub failover_timeout_secs: u64,
    pub max_failover_attempts: u32,
    pub prefer_local: bool,
    pub cooldown_secs: u64,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_failover: true,
            health_check_interval_secs: 10,
            health_check_timeout_secs: 5,
            unhealthy_threshold: 3,
            failover_timeout_secs: 60,
            max_failover_attempts: 3,
            prefer_local: true,
            cooldown_secs: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverRecord {
    pub id: String,
    pub failed_node_id: String,
    pub target_node_id: String,
    pub trigger_reason: FailoverTrigger,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: FailoverStatus,
    pub affected_players: u32,
    pub affected_servers: Vec<String>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailoverTrigger {
    HealthCheckFailed,
    ManualTrigger,
    LoadThresholdExceeded,
    NetworkError,
    ProcessCrashed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailoverStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub node_id: String,
    pub consecutive_failures: u32,
    pub last_check: DateTime<Utc>,
    pub last_failure: Option<DateTime<Utc>>,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl Default for NodeHealth {
    fn default() -> Self {
        Self {
            node_id: String::new(),
            consecutive_failures: 0,
            last_check: Utc::now(),
            last_failure: None,
            health_status: HealthStatus::Unknown,
        }
    }
}

impl FailoverManager {
    pub fn new(config: FailoverConfig) -> Self {
        Self {
            state: Arc::new(FailoverState {
                active_failovers: RwLock::new(HashMap::new()),
                failover_history: RwLock::new(VecDeque::with_capacity(100)),
                node_health: RwLock::new(HashMap::new()),
            }),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn update_config(&self, config: FailoverConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> FailoverConfig {
        self.config.read().clone()
    }

    pub fn update_node_health(&self, node_id: &str, healthy: bool) -> Option<NodeHealth> {
        let config = self.config.read();
        let mut health = self.state.node_health.write();
        let node = health.entry(node_id.to_string()).or_default();
        node.node_id = node_id.to_string();
        node.last_check = Utc::now();

        if healthy {
            node.consecutive_failures = 0;
            node.health_status = HealthStatus::Healthy;
        } else {
            node.consecutive_failures += 1;
            node.last_failure = Some(Utc::now());

            node.health_status = if node.consecutive_failures >= config.unhealthy_threshold {
                HealthStatus::Unhealthy
            } else {
                HealthStatus::Degraded
            };
        }

        Some(node.clone())
    }

    pub fn get_node_health(&self, node_id: &str) -> Option<NodeHealth> {
        self.state.node_health.read().get(node_id).cloned()
    }

    pub fn get_all_health(&self) -> Vec<NodeHealth> {
        self.state.node_health.read().values().cloned().collect()
    }

    pub fn should_failover(&self, node_id: &str) -> bool {
        let config = self.config.read();
        if !config.enabled || !config.auto_failover {
            return false;
        }

        let health = self.state.node_health.read();
        if let Some(node) = health.get(node_id) {
            node.consecutive_failures >= config.unhealthy_threshold
        } else {
            false
        }
    }

    pub fn initiate_failover(&self, failed_node_id: &str, target_node_id: &str, trigger: FailoverTrigger) -> FailoverRecord {
        let record = FailoverRecord {
            id: Uuid::new_v4().to_string(),
            failed_node_id: failed_node_id.to_string(),
            target_node_id: target_node_id.to_string(),
            trigger_reason: trigger,
            started_at: Utc::now(),
            completed_at: None,
            status: FailoverStatus::Pending,
            affected_players: 0,
            affected_servers: Vec::new(),
            success: false,
            error: None,
        };

        let mut active = self.state.active_failovers.write();
        active.insert(record.id.clone(), record.clone());
        drop(active);

        tracing::info!("Failover initiated: {} -> {}", failed_node_id, target_node_id);
        record
    }

    pub fn update_failover_status(&self, failover_id: &str, status: FailoverStatus, affected_players: u32, servers: Vec<String>) {
        let mut active = self.state.active_failovers.write();
        if let Some(record) = active.get_mut(failover_id) {
            record.status = status.clone();
            record.affected_players = affected_players;
            record.affected_servers = servers;

            if matches!(status, FailoverStatus::Completed | FailoverStatus::Failed) {
                record.completed_at = Some(Utc::now());
                record.success = matches!(status, FailoverStatus::Completed);

                let completed = record.clone();
                drop(active);

                let mut history = self.state.failover_history.write();
                history.push_back(completed);
                if history.len() > 100 {
                    history.pop_front();
                }

                self.state.active_failovers.write().remove(failover_id);
            }
        }
    }

    pub fn get_active_failovers(&self) -> Vec<FailoverRecord> {
        self.state.active_failovers.read().values().cloned().collect()
    }

    pub fn get_failover_history(&self, limit: usize) -> Vec<FailoverRecord> {
        let history = self.state.failover_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn find_optimal_target(&self, failed_node_id: &str, candidates: &[String]) -> Option<String> {
        let config = self.config.read();
        if candidates.is_empty() {
            return None;
        }

        let health = self.state.node_health.read();
        let candidates: Vec<&str> = if config.prefer_local {
            candidates.iter()
                .filter(|c| {
                    health.get(*c)
                        .map(|h| h.health_status == HealthStatus::Healthy)
                        .unwrap_or(false)
                })
                .map(|s| s.as_str())
                .collect()
        } else {
            candidates.iter().map(|s| s.as_str()).collect()
        };

        candidates.first().map(|s| s.to_string())
    }

    pub fn cancel_failover(&self, failover_id: &str) -> Result<(), FailoverError> {
        let mut active = self.state.active_failovers.write();
        if let Some(record) = active.get_mut(failover_id) {
            if matches!(record.status, FailoverStatus::Pending | FailoverStatus::InProgress) {
                record.status = FailoverStatus::Failed;
                record.error = Some("Cancelled by user".to_string());
                record.completed_at = Some(Utc::now());
                let failed = record.clone();
                drop(active);

                let mut history = self.state.failover_history.write();
                history.push_back(failed);
                active.remove(failover_id);
                Ok(())
            } else {
                Err(FailoverError::CannotCancel(failover_id.to_string()))
            }
        } else {
            Err(FailoverError::FailoverNotFound(failover_id.to_string()))
        }
    }

    pub fn get_stats(&self) -> FailoverStats {
        let history = self.state.failover_history.read();
        let active = self.state.active_failovers.read();
        let health = self.state.node_health.read();

        let total_failovers = history.len();
        let successful_failovers = history.iter().filter(|r| r.success).count();
        let failed_failovers = history.iter().filter(|r| !r.success).count();

        FailoverStats {
            total_failovers,
            successful_failovers,
            failed_failovers,
            active_failovers: active.len() as u32,
            healthy_nodes: health.values().filter(|h| h.health_status == HealthStatus::Healthy).count() as u32,
            degraded_nodes: health.values().filter(|h| h.health_status == HealthStatus::Degraded).count() as u32,
            unhealthy_nodes: health.values().filter(|h| h.health_status == HealthStatus::Unhealthy).count() as u32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverStats {
    pub total_failovers: usize,
    pub successful_failovers: usize,
    pub failed_failovers: usize,
    pub active_failovers: u32,
    pub healthy_nodes: u32,
    pub degraded_nodes: u32,
    pub unhealthy_nodes: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum FailoverError {
    #[error("Failover {0} not found")]
    FailoverNotFound(String),

    #[error("Cannot cancel failover {0} - not in cancellable state")]
    CannotCancel(String),
}

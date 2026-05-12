use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Online,
    Offline,
    Maintenance,
    Draining,
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::Offline
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub status: NodeStatus,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub player_count: u32,
    pub max_players: u32,
    pub tps: f64,
    pub region: Option<String>,
    pub labels: Vec<String>,
    pub last_heartbeat: DateTime<Utc>,
    pub version: String,
    pub is_proxy: bool,
}

impl NodeInfo {
    pub fn new(id: String, name: String, host: String, port: u16) -> Self {
        Self {
            id,
            name,
            host,
            port,
            status: NodeStatus::Offline,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            player_count: 0,
            max_players: 100,
            tps: 20.0,
            region: None,
            labels: Vec::new(),
            last_heartbeat: Utc::now(),
            version: "1.21".to_string(),
            is_proxy: false,
        }
    }

    pub fn health_score(&self) -> f64 {
        let mut score = 100.0;
        if self.status != NodeStatus::Online {
            return 0.0;
        }
        score -= self.cpu_usage.min(50.0);
        score -= self.memory_usage.min(30.0);
        if self.tps < 20.0 {
            score -= (20.0 - self.tps) * 2.0;
        }
        score.max(0.0)
    }

    pub fn is_healthy(&self) -> bool {
        self.health_score() >= 50.0 && self.status == NodeStatus::Online
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub max_players: u32,
    pub servers: Vec<ServerEntry>,
    pub force_default_server: Option<String>,
    pub online_mode: bool,
    pub compression_threshold: u32,
    pub ip_forward: bool,
    pub ping_passthrough: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Bungeecord,
    Velocity,
    Waterfall,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            proxy_type: ProxyType::Velocity,
            host: "0.0.0.0".to_string(),
            port: 25577,
            max_players: 1000,
            servers: Vec::new(),
            force_default_server: None,
            online_mode: true,
            compression_threshold: 256,
            ip_forward: true,
            ping_passthrough: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub address: String,
    pub motd: String,
    pub restricted: bool,
    pub hidden: bool,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub strategy: LoadBalanceStrategy,
    pub health_check_interval_secs: u64,
    pub health_check_timeout_secs: u64,
    pub failover_enabled: bool,
    pub max_failover_attempts: u32,
    pub sticky_sessions: bool,
    pub sticky_session_ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalanceStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    WeightedLeastConnections,
    Hash,
    Random,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalanceStrategy::LeastConnections,
            health_check_interval_secs: 10,
            health_check_timeout_secs: 5,
            failover_enabled: true,
            max_failover_attempts: 3,
            sticky_sessions: true,
            sticky_session_ttl_secs: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSyncConfig {
    pub enabled: bool,
    pub sync_to_proxy: bool,
    pub prefix_global: bool,
    pub private_messages_enabled: bool,
    pub channel_switching: bool,
    pub chat_radius_blocks: Option<u32>,
    pub proxy_chat_format: String,
    pub cooldown_ms: u64,
}

impl Default for ChatSyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_to_proxy: true,
            prefix_global: true,
            private_messages_enabled: true,
            channel_switching: true,
            chat_radius_blocks: None,
            proxy_chat_format: "[{server}] {display_name}: {message}".to_string(),
            cooldown_ms: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingUpdateConfig {
    pub strategy: RollingStrategy,
    pub batch_size: u32,
    pub wait_time_secs: u64,
    pub health_check_grace_period_secs: u64,
    pub max_unavailable_nodes: u32,
    pub auto_rollback_on_failure: bool,
    pub maintenance_mode_first: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RollingStrategy {
    Rolling,
    BlueGreen,
    Canary,
    Recreate,
}

impl Default for RollingUpdateConfig {
    fn default() -> Self {
        Self {
            strategy: RollingStrategy::Rolling,
            batch_size: 1,
            wait_time_secs: 60,
            health_check_grace_period_secs: 30,
            max_unavailable_nodes: 1,
            auto_rollback_on_failure: true,
            maintenance_mode_first: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterEvent {
    pub event_type: ClusterEventType,
    pub node_id: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterEventType {
    NodeAdded,
    NodeRemoved,
    NodeStatusChanged,
    NodeHeartbeat,
    NodeMetricsUpdated,
    ConfigUpdated,
    FailoverTriggered,
    FailoverCompleted,
    RollingUpdateStarted,
    RollingUpdateCompleted,
    ChatMessage,
    DataSync,
}

impl ClusterEvent {
    pub fn new(event_type: ClusterEventType, node_id: Option<String>, payload: serde_json::Value) -> Self {
        Self {
            event_type,
            node_id,
            payload,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterTopology {
    pub nodes: Vec<NodeInfo>,
    pub connections: Vec<NodeConnection>,
    pub proxy_nodes: Vec<NodeInfo>,
    pub game_nodes: Vec<NodeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub from: String,
    pub to: String,
    pub connection_type: ConnectionType,
    pub latency_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Proxy,
    Direct,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub total_nodes: u32,
    pub online_nodes: u32,
    pub total_players: u32,
    pub avg_cpu: f64,
    pub avg_memory: f64,
    pub avg_tps: f64,
    pub total_cpu: f64,
    pub total_memory: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub node_id: String,
    pub healthy: bool,
    pub latency_ms: Option<u32>,
    pub error: Option<String>,
    pub checked_at: DateTime<Utc>,
}

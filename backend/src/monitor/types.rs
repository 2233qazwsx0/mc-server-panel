use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertLevel::Info => write!(f, "info"),
            AlertLevel::Warning => write!(f, "warning"),
            AlertLevel::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub level: AlertLevel,
    pub metric_type: String,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

impl Alert {
    pub fn new(
        id: String,
        level: AlertLevel,
        metric_type: String,
        message: String,
        value: f64,
        threshold: f64,
    ) -> Self {
        Self {
            id,
            level,
            metric_type,
            message,
            value,
            threshold,
            timestamp: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    pub id: String,
    pub name: String,
    pub metric_type: MetricType,
    pub operator: ThresholdOperator,
    pub threshold_value: f64,
    pub alert_level: AlertLevel,
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    CpuUsage,
    MemoryUsage,
    MemoryPercent,
    DiskRead,
    DiskWrite,
    NetworkRx,
    NetworkTx,
    ThreadCount,
    GcPause,
    GcCount,
    HeapUsed,
    HeapMax,
    Tps,
    PlayerCount,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::CpuUsage => "cpu_usage",
            MetricType::MemoryUsage => "memory_usage",
            MetricType::MemoryPercent => "memory_percent",
            MetricType::DiskRead => "disk_read",
            MetricType::DiskWrite => "disk_write",
            MetricType::NetworkRx => "network_rx",
            MetricType::NetworkTx => "network_tx",
            MetricType::ThreadCount => "thread_count",
            MetricType::GcPause => "gc_pause",
            MetricType::GcCount => "gc_count",
            MetricType::HeapUsed => "heap_used",
            MetricType::HeapMax => "heap_max",
            MetricType::Tps => "tps",
            MetricType::PlayerCount => "player_count",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: DateTime<Utc>,
    pub metric_type: MetricType,
    pub value: f64,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub metrics: HashMap<MetricType, f64>,
}

impl PerformanceSnapshot {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            metrics: HashMap::new(),
        }
    }

    pub fn with_metric(mut self, metric_type: MetricType, value: f64) -> Self {
        self.metrics.insert(metric_type, value);
        self
    }
}

impl Default for PerformanceSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcMetrics {
    pub gc_type: String,
    pub pause_ms: f64,
    pub before_heap: u64,
    pub after_heap: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub interface: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStats {
    pub device: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_count: u64,
    pub write_count: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub metric_type: MetricType,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub sample_count: usize,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub metric_type: MetricType,
    pub current_value: f64,
    pub expected_range: (f64, f64),
    pub deviation_score: f64,
    pub is_anomaly: bool,
    pub timestamp: DateTime<Utc>,
}

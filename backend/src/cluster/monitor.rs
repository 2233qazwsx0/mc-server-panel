use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
pub struct ClusterMonitor {
    state: Arc<ClusterMonitorState>,
    retention_hours: u64,
}

struct ClusterMonitorState {
    node_metrics: RwLock<HashMap<String, VecDeque<NodeMetrics>>>,
    aggregate_metrics: RwLock<AggregateMetrics>,
    alerts: RwLock<Vec<Alert>>,
    baselines: RwLock<HashMap<String, MetricBaseline>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_in_mbps: f64,
    pub network_out_mbps: f64,
    pub player_count: u32,
    pub tps: f64,
    pub active_connections: u32,
}

impl NodeMetrics {
    pub fn health_score(&self) -> f64 {
        let mut score = 100.0;
        score -= self.cpu_usage.min(50.0);
        score -= self.memory_usage.min(30.0);
        if self.tps < 20.0 {
            score -= (20.0 - self.tps) * 2.0;
        }
        score.max(0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregateMetrics {
    pub total_cpu: f64,
    pub total_memory: f64,
    pub total_players: u32,
    pub avg_tps: f64,
    pub online_nodes: u32,
    pub offline_nodes: u32,
    pub total_nodes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub node_id: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    CpuHigh,
    MemoryHigh,
    TpsLow,
    NodeOffline,
    DiskFull,
    NetworkLatency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricBaseline {
    pub metric_name: String,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub std_deviation: f64,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub cpu_threshold: f64,
    pub memory_threshold: f64,
    pub tps_threshold: f64,
    pub disk_threshold: f64,
    pub alert_retention_hours: u64,
    pub metrics_retention_hours: u64,
    pub enable_baseline_calculation: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            cpu_threshold: 90.0,
            memory_threshold: 85.0,
            tps_threshold: 18.0,
            disk_threshold: 90.0,
            alert_retention_hours: 168,
            metrics_retention_hours: 24,
            enable_baseline_calculation: true,
        }
    }
}

impl ClusterMonitor {
    pub fn new(retention_hours: u64) -> Self {
        Self {
            state: Arc::new(ClusterMonitorState {
                node_metrics: RwLock::new(HashMap::new()),
                aggregate_metrics: RwLock::new(AggregateMetrics::default()),
                alerts: RwLock::new(Vec::new()),
                baselines: RwLock::new(HashMap::new()),
            }),
            retention_hours,
        }
    }

    pub fn record_metrics(&self, metrics: NodeMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let node_id = metrics.node_id.clone();

        let mut all_metrics = self.state.node_metrics.write();
        let node_history = all_metrics.entry(node_id.clone()).or_insert_with(|| VecDeque::with_capacity(1000));
        node_history.push_back(metrics.clone());

        let cutoff = Utc::now() - Duration::hours(self.retention_hours as i64);
        node_history.retain(|m| m.timestamp > cutoff);

        if metrics.cpu_usage > 90.0 {
            alerts.push(self.create_alert(AlertType::CpuHigh, AlertSeverity::Warning, &node_id, format!("CPU usage at {:.1}%", metrics.cpu_usage)));
        }

        if metrics.memory_usage > 85.0 {
            alerts.push(self.create_alert(AlertType::MemoryHigh, AlertSeverity::Warning, &node_id, format!("Memory usage at {:.1}%", metrics.memory_usage)));
        }

        if metrics.tps < 18.0 {
            alerts.push(self.create_alert(AlertType::TpsLow, AlertSeverity::Critical, &node_id, format!("TPS dropped to {:.1}", metrics.tps)));
        }

        if !alerts.is_empty() {
            let mut existing_alerts = self.state.alerts.write();
            existing_alerts.extend(alerts.clone());
        }

        drop(all_metrics);

        self.update_aggregate_metrics();
        alerts
    }

    fn create_alert(&self, alert_type: AlertType, severity: AlertSeverity, node_id: &str, message: String) -> Alert {
        Alert {
            id: uuid::Uuid::new_v4().to_string(),
            alert_type,
            severity,
            node_id: node_id.to_string(),
            message,
            created_at: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
        }
    }

    pub fn get_node_metrics(&self, node_id: &str, hours: u64) -> Vec<NodeMetrics> {
        let cutoff = Utc::now() - Duration::hours(hours as i64);
        self.state.node_metrics.read()
            .get(node_id)
            .map(|metrics| {
                metrics.iter()
                    .filter(|m| m.timestamp > cutoff)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_all_metrics(&self) -> HashMap<String, Vec<NodeMetrics>> {
        self.state.node_metrics.read()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone().into_iter().collect()))
            .collect()
    }

    pub fn get_aggregate_metrics(&self) -> AggregateMetrics {
        self.state.aggregate_metrics.read().clone()
    }

    fn update_aggregate_metrics(&self) {
        let all_metrics = self.state.node_metrics.read();

        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;
        let mut total_players = 0u32;
        let mut total_tps = 0.0;
        let mut node_count = 0;
        let mut online_count = 0u32;

        for metrics in all_metrics.values() {
            if let Some(latest) = metrics.back() {
                total_cpu += latest.cpu_usage;
                total_memory += latest.memory_usage;
                total_players += latest.player_count;
                total_tps += latest.tps;
                node_count += 1;
                if latest.health_score() > 50.0 {
                    online_count += 1;
                }
            }
        }

        let mut aggregate = AggregateMetrics {
            total_cpu,
            total_memory,
            total_players,
            avg_tps: if node_count > 0 { total_tps / node_count as f64 } else { 20.0 },
            online_nodes: online_count,
            offline_nodes: (node_count as u32).saturating_sub(online_count),
            total_nodes: node_count as u32,
        };

        *self.state.aggregate_metrics.write() = aggregate;
    }

    pub fn get_unacknowledged_alerts(&self) -> Vec<Alert> {
        self.state.alerts.read()
            .iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    pub fn acknowledge_alert(&self, alert_id: &str, acknowledged_by: &str) -> Result<(), MonitorError> {
        let mut alerts = self.state.alerts.write();
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(acknowledged_by.to_string());
            Ok(())
        } else {
            Err(MonitorError::AlertNotFound(alert_id.to_string()))
        }
    }

    pub fn clear_old_alerts(&self, older_than_hours: u64) {
        let cutoff = Utc::now() - Duration::hours(older_than_hours as i64);
        self.state.alerts.write().retain(|a| a.created_at > cutoff || !a.acknowledged);
    }

    pub fn calculate_baseline(&self, node_id: &str) -> Option<MetricBaseline> {
        let metrics = self.get_node_metrics(node_id, 24);
        if metrics.is_empty() {
            return None;
        }

        let cpu_values: Vec<f64> = metrics.iter().map(|m| m.cpu_usage).collect();
        let memory_values: Vec<f64> = metrics.iter().map(|m| m.memory_usage).collect();

        let calc_baseline = |values: &[f64]| -> MetricBaseline {
            let sum: f64 = values.iter().sum();
            let avg = sum / values.len() as f64;
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let variance: f64 = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
            let std_dev = variance.sqrt();

            MetricBaseline {
                metric_name: node_id.to_string(),
                avg_value: avg,
                min_value: min,
                max_value: max,
                std_deviation: std_dev,
                calculated_at: Utc::now(),
            }
        };

        let baseline = calc_baseline(&cpu_values);
        self.state.baselines.write().insert(node_id.to_string(), baseline.clone());
        Some(baseline)
    }

    pub fn get_baseline(&self, node_id: &str) -> Option<MetricBaseline> {
        self.state.baselines.read().get(node_id).cloned()
    }

    pub fn compare_to_baseline(&self, metrics: &NodeMetrics) -> HashMap<String, f64> {
        let mut comparisons = HashMap::new();

        if let Some(baseline) = self.get_baseline(&metrics.node_id) {
            comparisons.insert("cpu_deviation".to_string(), metrics.cpu_usage - baseline.avg_value);
            comparisons.insert("memory_deviation".to_string(), metrics.memory_usage - baseline.avg_value);
        }

        comparisons
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Alert {0} not found")]
    AlertNotFound(String),
}

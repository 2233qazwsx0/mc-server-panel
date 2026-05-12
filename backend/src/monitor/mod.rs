pub mod alert;
pub mod api;
pub mod baseline;
pub mod dashboard;
pub mod deadlock;
pub mod disk_io;
pub mod history;
pub mod jvm_gc;
pub mod network;
pub mod silence;
pub mod system_monitor;
pub mod types;
pub mod webhook;

pub use alert::AlertManager;
pub use baseline::BaselineLearner;
pub use dashboard::DashboardManager;
pub use deadlock::ThreadDeadlockDetector;
pub use disk_io::DiskIoMonitor;
pub use history::MetricsHistory;
pub use jvm_gc::JvmGcMonitor;
pub use network::NetworkMonitor;
pub use silence::{EscalationManager, SilenceManager};
pub use system_monitor::{MetricsSnapshot, ProcessMetrics, ServerStatus, SystemInfo, SystemMetrics, SystemMonitor};
pub use types::*;
pub use webhook::WebhookNotifier;

use crate::monitor::{
    GcMetrics, MetricType,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Monitor error: {0}")]
    General(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[derive(Clone)]
pub struct MonitorOrchestrator {
    pub system_monitor: Arc<SystemMonitor>,
    pub gc_monitor: Arc<JvmGcMonitor>,
    pub deadlock_detector: Arc<ThreadDeadlockDetector>,
    pub alert_manager: Arc<AlertManager>,
    pub webhook_notifier: Arc<WebhookNotifier>,
    pub disk_io_monitor: Arc<DiskIoMonitor>,
    pub network_monitor: Arc<NetworkMonitor>,
    pub metrics_history: Arc<MetricsHistory>,
    pub dashboard_manager: Arc<DashboardManager>,
    pub silence_manager: Arc<SilenceManager>,
    pub escalation_manager: Arc<EscalationManager>,
    pub baseline_learner: Arc<BaselineLearner>,
}

impl MonitorOrchestrator {
    pub fn new() -> Self {
        let system_monitor = Arc::new(SystemMonitor::new(1000));
        let gc_monitor = Arc::new(JvmGcMonitor::with_default_config());
        let deadlock_detector = Arc::new(ThreadDeadlockDetector::with_default());
        let alert_manager = Arc::new(AlertManager::with_default());
        let webhook_notifier = Arc::new(WebhookNotifier::new());
        let disk_io_monitor = Arc::new(DiskIoMonitor::with_default());
        let network_monitor = Arc::new(NetworkMonitor::with_default());
        let metrics_history = Arc::new(MetricsHistory::with_default());
        let dashboard_manager = Arc::new(DashboardManager::with_default());
        let silence_manager = Arc::new(SilenceManager::with_default());
        let escalation_manager = Arc::new(EscalationManager::with_default());
        let baseline_learner = Arc::new(BaselineLearner::with_default());

        alert_manager.create_default_thresholds();
        dashboard_manager.init_default_templates();
        escalation_manager.create_default_policies();
        baseline_learner.setup_default_configs();

        Self {
            system_monitor,
            gc_monitor,
            deadlock_detector,
            alert_manager,
            webhook_notifier,
            disk_io_monitor,
            network_monitor,
            metrics_history,
            dashboard_manager,
            silence_manager,
            escalation_manager,
            baseline_learner,
        }
    }

    pub async fn collect_all_metrics(&self, server_pid: Option<u32>) -> ComprehensiveMetrics {
        let snapshot = self.system_monitor.collect(server_pid).await;

        let mut metrics_map = HashMap::new();
        metrics_map.insert(MetricType::CpuUsage, snapshot.system.cpu_usage as f64);
        metrics_map.insert(MetricType::MemoryPercent, snapshot.system.memory_percent as f64);
        metrics_map.insert(MetricType::MemoryUsage, snapshot.system.memory_used as f64);

        if let Some(process) = &snapshot.process {
            metrics_map.insert(MetricType::MemoryUsage, process.memory_used as f64);
        }

        self.metrics_history.record_batch(&metrics_map);

        for (metric_type, value) in &metrics_map {
            self.baseline_learner.add_sample(*metric_type, *value);
        }

        let gc_history = self.gc_monitor.get_gc_history(100);
        let deadlock_info = self.deadlock_detector.get_deadlock_history(10);
        let disk_stats = self.disk_io_monitor.get_io_history(None, 100);
        let network_summary = self.network_monitor.get_network_summary();

        let active_alerts = self.alert_manager.get_active_alerts();
        let silenced_alerts: Vec<_> = active_alerts
            .iter()
            .filter(|a| self.silence_manager.is_silenced(a))
            .map(|a| a.id.clone())
            .collect();

        ComprehensiveMetrics {
            snapshot,
            gc_metrics: gc_history,
            deadlocks: deadlock_info,
            disk_io: disk_stats,
            network: network_summary,
            alerts: active_alerts,
            silenced_alert_ids: silenced_alerts,
            anomalies: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub async fn process_alerts(&self, metrics: &ComprehensiveMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();

        let mut current_metrics = HashMap::new();
        current_metrics.insert(MetricType::CpuUsage, metrics.snapshot.system.cpu_usage as f64);
        current_metrics.insert(MetricType::MemoryPercent, metrics.snapshot.system.memory_percent as f64);

        let threshold_alerts = self.alert_manager.check_all_thresholds(&current_metrics);

        for mut alert in threshold_alerts {
            if !self.silence_manager.is_silenced(&alert) {
                self.alert_manager.record_alert(alert.clone());
                alerts.push(alert);
            }
        }

        for alert in &alerts {
            let _ = self.escalation_manager.start_escalation(alert, "critical_escalation");
        }

        alerts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveMetrics {
    pub snapshot: MetricsSnapshot,
    pub gc_metrics: Vec<GcMetrics>,
    pub deadlocks: Vec<crate::monitor::deadlock::DeadlockInfo>,
    pub disk_io: Vec<DiskIoStats>,
    pub network: NetworkSummary,
    pub alerts: Vec<Alert>,
    pub silenced_alert_ids: Vec<String>,
    pub anomalies: Vec<AnomalyAlert>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIoStats {
    pub device: String,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub utilization_percent: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub current_rx_bps: u64,
    pub current_tx_bps: u64,
    pub formatted_rx: String,
    pub formatted_tx: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub id: String,
    pub metric_type: String,
    pub current_value: f64,
    pub deviation_score: f64,
    pub severity: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for MonitorOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

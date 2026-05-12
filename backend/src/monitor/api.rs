use crate::monitor::{
    alert::{AlertCondition, AlertRule, ThresholdConfig},
    dashboard::{Dashboard, DashboardManager, Widget},
    history::{MetricsExport, MetricsQuery},
    silence::{EscalationPolicy, SilenceRule},
    types::{Alert, AlertLevel, AlertThreshold, MetricType, ThresholdOperator},
    webhook::{WebhookConfig, WebhookType},
    ComprehensiveMetrics, MonitorOrchestrator,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct MonitorState {
    pub orchestrator: Arc<MonitorOrchestrator>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct TimeRangeParams {
    pub duration_secs: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateThresholdRequest {
    pub name: String,
    pub metric_type: MetricType,
    pub operator: ThresholdOperator,
    pub threshold_value: f64,
    pub alert_level: AlertLevel,
    pub cooldown_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    pub webhook_type: WebhookType,
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSilenceRequest {
    pub name: String,
    pub description: String,
    pub duration_hours: u32,
    pub alert_levels: Option<Vec<AlertLevel>>,
    pub metric_types: Option<Vec<String>>,
}

pub fn create_monitor_routes(state: MonitorState) -> Router {
    Router::new()
        .route("/metrics", get(get_all_metrics))
        .route("/metrics/realtime", get(get_realtime_metrics))
        .route("/metrics/history", get(get_metrics_history))
        .route("/metrics/export", post(export_metrics))
        .route("/gc", get(get_gc_metrics))
        .route("/gc/history", get(get_gc_history))
        .route("/deadlock", get(get_deadlock_status))
        .route("/deadlock/history", get(get_deadlock_history))
        .route("/disk-io", get(get_disk_io_stats))
        .route("/disk-io/history", get(get_disk_io_history))
        .route("/network", get(get_network_stats))
        .route("/network/summary", get(get_network_summary))
        .route("/alerts", get(get_alerts))
        .route("/alerts/active", get(get_active_alerts))
        .route("/alerts/:id/acknowledge", post(acknowledge_alert))
        .route("/thresholds", get(get_thresholds))
        .route("/thresholds", post(create_threshold))
        .route("/thresholds/:id", put(update_threshold))
        .route("/thresholds/:id", delete(delete_threshold))
        .route("/thresholds/:id/enable", post(enable_threshold))
        .route("/thresholds/:id/disable", post(disable_threshold))
        .route("/webhooks", get(get_webhooks))
        .route("/webhooks", post(create_webhook))
        .route("/webhooks/:id", delete(delete_webhook))
        .route("/webhooks/:id/enable", post(enable_webhook))
        .route("/webhooks/:id/disable", post(disable_webhook))
        .route("/dashboards", get(get_dashboards))
        .route("/dashboards", post(create_dashboard))
        .route("/dashboards/:id", get(get_dashboard))
        .route("/dashboards/:id", put(update_dashboard))
        .route("/dashboards/:id", delete(delete_dashboard))
        .route("/dashboards/:id/widgets", post(add_widget))
        .route("/dashboards/:id/widgets/:widget_id", delete(remove_widget))
        .route("/dashboards/templates", get(get_dashboard_templates))
        .route("/dashboards/:id/snapshot", get(get_dashboard_snapshot))
        .route("/silence", get(get_silence_rules))
        .route("/silence", post(create_silence_rule))
        .route("/silence/:id", delete(delete_silence_rule))
        .route("/silence/maintenance", post(create_maintenance_window))
        .route("/escalation", get(get_escalation_policies))
        .route("/escalation", post(create_escalation_policy))
        .route("/escalation/:id", delete(delete_escalation_policy))
        .route("/escalation/:alert_id/escalate", post(manual_escalate))
        .route("/baseline", get(get_baseline_profiles))
        .route("/baseline/:metric_type", get(get_baseline_stats))
        .route("/baseline/:metric_type/samples", post(add_baseline_samples))
        .with_state(state)
}

async fn get_all_metrics(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<ComprehensiveMetrics>> {
    let metrics = state.orchestrator.collect_all_metrics(None).await;
    Json(ApiResponse::success(metrics))
}

async fn get_realtime_metrics(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<crate::monitor::MetricsSnapshot>> {
    let snapshot = state.orchestrator.system_monitor.collect(None).await;
    Json(ApiResponse::success(snapshot))
}

async fn get_metrics_history(
    State(state): State<MonitorState>,
    Query(params): Query<TimeRangeParams>,
) -> Json<ApiResponse<Vec<crate::monitor::types::MetricDataPoint>>> {
    let duration = params.duration_secs.unwrap_or(3600);
    let history = state.orchestrator.metrics_history.get_metric_history(
        MetricType::CpuUsage,
        &chrono::Utc::now() - chrono::Duration::seconds(duration as i64),
        &chrono::Utc::now(),
    );
    Json(ApiResponse::success(history))
}

async fn export_metrics(
    State(state): State<MonitorState>,
    Json(export): Json<MetricsExport>,
) -> Json<ApiResponse<String>> {
    match state.orchestrator.metrics_history.export(export) {
        Ok(data) => Json(ApiResponse::success(data)),
        Err(e) => Json(ApiResponse::error(&e)),
    }
}

async fn get_gc_metrics(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<crate::monitor::GcMetrics>>> {
    let gc = state.orchestrator.gc_monitor.get_gc_history(100);
    Json(ApiResponse::success(gc))
}

async fn get_gc_history(
    State(state): State<MonitorState>,
    Query(params): Query<PaginationParams>,
) -> Json<ApiResponse<Vec<crate::monitor::GcMetrics>>> {
    let limit = params.limit.unwrap_or(100);
    let history = state.orchestrator.gc_monitor.get_gc_history(limit);
    Json(ApiResponse::success(history))
}

async fn get_deadlock_status(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<bool>> {
    let is_deadlocked = state.orchestrator.deadlock_detector.is_deadlock_detected();
    Json(ApiResponse::success(is_deadlocked))
}

async fn get_deadlock_history(
    State(state): State<MonitorState>,
    Query(params): Query<PaginationParams>,
) -> Json<ApiResponse<Vec<crate::monitor::deadlock::DeadlockInfo>>> {
    let limit = params.limit.unwrap_or(50);
    let history = state.orchestrator.deadlock_detector.get_deadlock_history(limit);
    Json(ApiResponse::success(history))
}

async fn get_disk_io_stats(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<crate::monitor::disk_io::DiskIoStats>>> {
    let stats = state.orchestrator.disk_io_monitor.get_io_history(None, 10);
    Json(ApiResponse::success(stats))
}

async fn get_disk_io_history(
    State(state): State<MonitorState>,
    Query(params): Query<TimeRangeParams>,
) -> Json<ApiResponse<Vec<crate::monitor::disk_io::DiskIoStats>>> {
    let duration = params.duration_secs.unwrap_or(3600);
    let (total_read, total_write, _, _) = state.orchestrator.disk_io_monitor.get_total_io_stats(duration);
    let stats = vec![crate::monitor::disk_io::DiskIoStats {
        device: "total".to_string(),
        read_bytes_per_sec: total_read,
        write_bytes_per_sec: total_write,
        read_ops_per_sec: 0,
        write_ops_per_sec: 0,
        utilization_percent: 0.0,
        queue_depth: 0,
        timestamp: chrono::Utc::now(),
    }];
    Json(ApiResponse::success(stats))
}

async fn get_network_stats(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<crate::monitor::network::NetworkInterfaceStats>>> {
    use crate::monitor::network::NetworkMonitor;
    let system = sysinfo::System::new_all();
    let stats = state.orchestrator.network_monitor.collect_network_stats(&system);
    Json(ApiResponse::success(stats.interfaces))
}

async fn get_network_summary(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<crate::monitor::network::NetworkSummary>> {
    let summary = state.orchestrator.network_monitor.get_network_summary();
    Json(ApiResponse::success(summary))
}

async fn get_alerts(
    State(state): State<MonitorState>,
    Query(params): Query<PaginationParams>,
) -> Json<ApiResponse<Vec<Alert>>> {
    let limit = params.limit.unwrap_or(100);
    let alerts = state.orchestrator.alert_manager.get_alert_history(limit);
    Json(ApiResponse::success(alerts))
}

async fn get_active_alerts(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<Alert>>> {
    let alerts = state.orchestrator.alert_manager.get_active_alerts();
    Json(ApiResponse::success(alerts))
}

async fn acknowledge_alert(
    State(state): State<MonitorState>,
    Path(alert_id): Path<String>,
) -> Json<ApiResponse<Alert>> {
    match state.orchestrator.alert_manager.acknowledge_alert(&alert_id, "api") {
        Ok(alert) => Json(ApiResponse::success(alert)),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn get_thresholds(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<AlertThreshold>>> {
    let thresholds = state.orchestrator.alert_manager.get_all_thresholds();
    Json(ApiResponse::success(thresholds))
}

async fn create_threshold(
    State(state): State<MonitorState>,
    Json(req): Json<CreateThresholdRequest>,
) -> Json<ApiResponse<AlertThreshold>> {
    let threshold = AlertThreshold {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        metric_type: req.metric_type,
        operator: req.operator,
        threshold_value: req.threshold_value,
        alert_level: req.alert_level,
        enabled: true,
        cooldown_seconds: req.cooldown_seconds.unwrap_or(300),
        created_at: chrono::Utc::now(),
    };

    if let Err(e) = state.orchestrator.alert_manager.add_threshold(threshold.clone()) {
        return Json(ApiResponse::error(&e.to_string()));
    }

    Json(ApiResponse::success(threshold))
}

async fn update_threshold(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
    Json(threshold): Json<AlertThreshold>,
) -> Json<ApiResponse<AlertThreshold>> {
    match state.orchestrator.alert_manager.update_threshold(threshold.clone()) {
        Ok(_) => Json(ApiResponse::success(threshold)),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn delete_threshold(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.orchestrator.alert_manager.remove_threshold(&id) {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn enable_threshold(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.orchestrator.alert_manager.enable_threshold(&id) {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn disable_threshold(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.orchestrator.alert_manager.disable_threshold(&id) {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn get_webhooks(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<WebhookConfig>>> {
    let webhooks = state.orchestrator.webhook_notifier.get_all_webhooks();
    Json(ApiResponse::success(webhooks))
}

async fn create_webhook(
    State(state): State<MonitorState>,
    Json(req): Json<CreateWebhookRequest>,
) -> Json<ApiResponse<WebhookConfig>> {
    let webhook = match req.webhook_type {
        WebhookType::Discord => WebhookNotifier::create_discord_webhook(&req.name, &req.url),
        WebhookType::Slack => WebhookNotifier::create_slack_webhook(&req.name, &req.url),
        _ => WebhookConfig {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            url: req.url,
            webhook_type: req.webhook_type,
            enabled: true,
            secret: req.secret,
            headers: std::collections::HashMap::new(),
            retry_count: 3,
            retry_delay_ms: 1000,
            timeout_secs: 30,
            filters: crate::monitor::webhook::WebhookFilters::default(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    };

    state.orchestrator.webhook_notifier.add_webhook(webhook.clone());
    Json(ApiResponse::success(webhook))
}

use crate::monitor::webhook::WebhookNotifier;

async fn delete_webhook(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    state.orchestrator.webhook_notifier.remove_webhook(&id);
    Json(ApiResponse::success(()))
}

async fn enable_webhook(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.orchestrator.webhook_notifier.enable_webhook(&id) {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn disable_webhook(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.orchestrator.webhook_notifier.disable_webhook(&id) {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn get_dashboards(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<Dashboard>>> {
    let dashboards = state.orchestrator.dashboard_manager.get_all_dashboards();
    Json(ApiResponse::success(dashboards))
}

async fn create_dashboard(
    State(state): State<MonitorState>,
    Json(req): Json<CreateDashboardRequest>,
) -> Json<ApiResponse<Dashboard>> {
    let dashboard = state.orchestrator.dashboard_manager.create_dashboard(
        &req.name,
        &req.description,
        "api",
    );
    Json(ApiResponse::success(dashboard))
}

async fn get_dashboard(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Dashboard>> {
    match state.orchestrator.dashboard_manager.get_dashboard(&id) {
        Some(d) => Json(ApiResponse::success(d)),
        None => Json(ApiResponse::error("Dashboard not found")),
    }
}

async fn update_dashboard(
    State(state): State<MonitorState>,
    Json(dashboard): Json<Dashboard>,
) -> Json<ApiResponse<Dashboard>> {
    match state.orchestrator.dashboard_manager.update_dashboard(dashboard.clone()) {
        Ok(_) => Json(ApiResponse::success(dashboard)),
        Err(e) => Json(ApiResponse::error(&e)),
    }
}

async fn delete_dashboard(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.orchestrator.dashboard_manager.delete_dashboard(&id) {
        Ok(_) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(&e)),
    }
}

async fn add_widget(
    State(state): State<MonitorState>,
    Path(dashboard_id): Path<String>,
    Json(widget): Json<Widget>,
) -> Json<ApiResponse<Dashboard>> {
    match state.orchestrator.dashboard_manager.add_widget(&dashboard_id, widget) {
        Ok(d) => Json(ApiResponse::success(d)),
        Err(e) => Json(ApiResponse::error(&e)),
    }
}

async fn remove_widget(
    State(state): State<MonitorState>,
    Path((dashboard_id, widget_id)): Path<(String, String)>,
) -> Json<ApiResponse<Dashboard>> {
    match state.orchestrator.dashboard_manager.remove_widget(&dashboard_id, &widget_id) {
        Ok(d) => Json(ApiResponse::success(d)),
        Err(e) => Json(ApiResponse::error(&e)),
    }
}

async fn get_dashboard_templates(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<crate::monitor::dashboard::DashboardTemplate>>> {
    let templates = state.orchestrator.dashboard_manager.get_all_templates();
    Json(ApiResponse::success(templates))
}

async fn get_dashboard_snapshot(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Vec<crate::monitor::dashboard::DashboardSnapshot>>> {
    let snapshots = state.orchestrator.dashboard_manager.get_snapshot_history(&id, 10);
    Json(ApiResponse::success(snapshots))
}

async fn get_silence_rules(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<SilenceRule>>> {
    let rules = state.orchestrator.silence_manager.get_all_rules();
    Json(ApiResponse::success(rules))
}

async fn create_silence_rule(
    State(state): State<MonitorState>,
    Json(rule): Json<SilenceRule>,
) -> Json<ApiResponse<SilenceRule>> {
    state.orchestrator.silence_manager.add_rule(rule.clone());
    Json(ApiResponse::success(rule))
}

async fn delete_silence_rule(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    state.orchestrator.silence_manager.remove_rule(&id);
    Json(ApiResponse::success(()))
}

async fn create_maintenance_window(
    State(state): State<MonitorState>,
    Json(req): Json<CreateSilenceRequest>,
) -> Json<ApiResponse<SilenceRule>> {
    let rule = state.orchestrator.silence_manager.create_maintenance_window(
        &req.name,
        &req.description,
        req.duration_hours,
        "api",
    );
    Json(ApiResponse::success(rule))
}

async fn get_escalation_policies(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<EscalationPolicy>>> {
    let policies = state.orchestrator.escalation_manager.get_all_policies();
    Json(ApiResponse::success(policies))
}

async fn create_escalation_policy(
    State(state): State<MonitorState>,
    Json(policy): Json<EscalationPolicy>,
) -> Json<ApiResponse<EscalationPolicy>> {
    state.orchestrator.escalation_manager.add_policy(policy.clone());
    Json(ApiResponse::success(policy))
}

async fn delete_escalation_policy(
    State(state): State<MonitorState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    state.orchestrator.escalation_manager.remove_policy(&id);
    Json(ApiResponse::success(()))
}

async fn manual_escalate(
    State(state): State<MonitorState>,
    Path(alert_id): Path<String>,
) -> Json<ApiResponse<crate::monitor::silence::EscalationEvent>> {
    match state.orchestrator.escalation_manager.escalate(&alert_id) {
        Ok(event) => Json(ApiResponse::success(event)),
        Err(e) => Json(ApiResponse::error(&e)),
    }
}

async fn get_baseline_profiles(
    State(state): State<MonitorState>,
) -> Json<ApiResponse<Vec<crate::monitor::baseline::BaselineProfile>>> {
    let profiles = state.orchestrator.baseline_learner.get_all_profiles();
    Json(ApiResponse::success(profiles))
}

async fn get_baseline_stats(
    State(state): State<MonitorState>,
    Path(metric_type): Path<MetricType>,
) -> Json<ApiResponse<Option<crate::monitor::baseline::LearningStats>>> {
    let stats = state.orchestrator.baseline_learner.get_learning_stats(metric_type);
    Json(ApiResponse::success(stats))
}

#[derive(Debug, Deserialize)]
pub struct AddSamplesRequest {
    pub values: Vec<f64>,
}

async fn add_baseline_samples(
    State(state): State<MonitorState>,
    Path(metric_type): Path<MetricType>,
    Json(req): Json<AddSamplesRequest>,
) -> Json<ApiResponse<usize>> {
    let count = req.values.len();
    state.orchestrator.baseline_learner.add_samples(metric_type, &req.values);
    Json(ApiResponse::success(count))
}

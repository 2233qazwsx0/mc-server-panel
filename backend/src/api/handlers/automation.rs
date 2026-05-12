use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::automation::{
    BackupConfig, BackupInfo, DiskInfo,
    TaskResult, TaskStatus,
    TestResult, TestSuite, VersionInfo,
};
use crate::automation::{
    backup, config_version, cron_scheduler, disk_monitor, log_cleanup, migration,
    restart_strategy, test_suite, update_checker, warmup,
};
use config_version::{ConfigVersionManager, ConfigVersion};
use migration::{MigrationConfig, MigrationPlan};
use warmup::{WarmupConfig, WarmupResult, WarmupStep};
use update_checker::UpdateCheckerConfig;
use disk_monitor::DiskMonitorConfig;
use restart_strategy::RestartStrategyConfig;

#[derive(Clone)]
pub struct AutomationState {
    pub backup_manager: Arc<RwLock<backup::BackupManager>>,
    pub log_cleaner: Arc<RwLock<log_cleanup::LogCleaner>>,
    pub restart_strategy: Arc<RwLock<restart_strategy::RestartStrategy>>,
    pub cron_scheduler: Arc<RwLock<cron_scheduler::CronScheduler>>,
    pub warmup_script: Arc<RwLock<warmup::WarmupScript>>,
    pub update_checker: Arc<RwLock<update_checker::UpdateChecker>>,
    pub disk_monitor: Arc<RwLock<disk_monitor::DiskMonitor>>,
    pub test_suite: Arc<RwLock<test_suite::AutomationTestSuite>>,
    pub config_version_manager: Arc<RwLock<config_version::ConfigVersionManager>>,
    pub migration_tool: Arc<RwLock<migration::MigrationTool>>,
}

impl AutomationState {
    pub fn new() -> Self {
        Self {
            backup_manager: Arc::new(RwLock::new(backup::BackupManager::new(BackupConfig::default()))),
            log_cleaner: Arc::new(RwLock::new(log_cleanup::LogCleaner::new(LogCleanupConfig::default()))),
            restart_strategy: Arc::new(RwLock::new(restart_strategy::RestartStrategy::new(RestartStrategyConfig::default()))),
            cron_scheduler: Arc::new(RwLock::new(cron_scheduler::CronScheduler::new())),
            warmup_script: Arc::new(RwLock::new(warmup::WarmupScript::new(WarmupConfig::default()))),
            update_checker: Arc::new(RwLock::new(update_checker::UpdateChecker::new(UpdateCheckerConfig::default()))),
            disk_monitor: Arc::new(RwLock::new(disk_monitor::DiskMonitor::new(DiskMonitorConfig::default()))),
            test_suite: Arc::new(RwLock::new(test_suite::AutomationTestSuite::new())),
            config_version_manager: Arc::new(RwLock::new(config_version::ConfigVersionManager::new())),
            migration_tool: Arc::new(RwLock::new(migration::MigrationTool::new())),
        }
    }
}

impl Default for AutomationState {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BackupRequest {
    pub server_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub backup_id: String,
    pub target_path: Option<String>,
}

pub async fn get_all_task_status(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Vec<TaskStatus>>> {
    let mut statuses = Vec::new();

    statuses.push(state.backup_manager.read().await.get_status());
    statuses.push(state.log_cleaner.read().await.get_status());
    statuses.push(state.restart_strategy.read().await.get_status());
    statuses.push(state.warmup_script.read().await.get_status());
    statuses.push(state.update_checker.read().await.get_status());
    statuses.push(state.disk_monitor.read().await.get_status());

    Json(ApiResponse::success(statuses))
}

pub async fn create_backup(
    State(state): State<Arc<AutomationState>>,
    Json(req): Json<BackupRequest>,
) -> Json<ApiResponse<BackupInfo>> {
    let server_path = std::path::PathBuf::from(req.server_path.unwrap_or_else(|| ".".to_string()));

    match state.backup_manager.read().await.create_backup(&server_path).await {
        Ok(backup) => Json(ApiResponse::success(backup)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn list_backups(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Vec<BackupInfo>>> {
    let backups = state.backup_manager.read().await.list_backups().await;
    Json(ApiResponse::success(backups))
}

pub async fn delete_backup(
    State(state): State<Arc<AutomationState>>,
    Path(backup_id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.backup_manager.read().await.delete_backup(&backup_id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn restore_backup(
    State(state): State<Arc<AutomationState>>,
    Json(req): Json<RestoreRequest>,
) -> Json<ApiResponse<()>> {
    let target_path = std::path::PathBuf::from(req.target_path.unwrap_or_else(|| ".".to_string()));

    match state.backup_manager.read().await.restore_backup(&req.backup_id, &target_path).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn run_log_cleanup(
    State(state): State<Arc<AutomationState>>,
    Json(req): Json<BackupRequest>,
) -> Json<ApiResponse<TaskResult>> {
    let server_path = std::path::PathBuf::from(req.server_path.unwrap_or_else(|| ".".to_string()));

    match state.log_cleaner.read().await.cleanup(&server_path).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn get_log_cleanup_stats(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<log_cleanup::LogCleanupStats>> {
    let stats = state.log_cleaner.read().await.get_stats().await;
    Json(ApiResponse::success(stats))
}

pub async fn check_disk_space(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Vec<DiskInfo>>> {
    match state.disk_monitor.read().await.check_disk_space().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn get_disk_alerts(
    State(state): State<Arc<AutomationState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<disk_monitor::DiskAlert>>> {
    let unacknowledged_only = params.get("unacknowledged").map(|s| s == "true").unwrap_or(false);
    let alerts = state.disk_monitor.read().await.get_alerts(unacknowledged_only);
    Json(ApiResponse::success(alerts))
}

pub async fn acknowledge_disk_alert(
    State(state): State<Arc<AutomationState>>,
    Path(alert_id): Path<String>,
) -> Json<ApiResponse<bool>> {
    let result = state.disk_monitor.read().await.acknowledge_alert(&alert_id);
    Json(ApiResponse::success(result))
}

pub async fn check_for_updates(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<VersionInfo>> {
    match state.update_checker.read().await.check_for_updates().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn get_cached_version(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Option<VersionInfo>>> {
    let version = state.update_checker.read().await.get_cached_version();
    Json(ApiResponse::success(version))
}

pub async fn list_test_suites(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Vec<TestSuite>>> {
    let suites = state.test_suite.read().await.list_suites();
    Json(ApiResponse::success(suites))
}

pub async fn run_test(
    State(state): State<Arc<AutomationState>>,
    Path(test_id): Path<String>,
) -> Json<ApiResponse<TestResult>> {
    let result = state.test_suite.read().await.run_test(&test_id).await;
    Json(ApiResponse::success(result))
}

pub async fn run_all_tests(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<std::collections::HashMap<String, Vec<TestResult>>>> {
    let results = state.test_suite.read().await.run_all_tests().await;
    Json(ApiResponse::success(results))
}

pub async fn create_config_version(
    State(state): State<Arc<AutomationState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<ApiResponse<ConfigVersion>> {
    let config_content = body["config_content"].as_str().unwrap_or("");
    let description = body["description"].as_str().unwrap_or("Manual snapshot");

    match state.config_version_manager.read().await.create_version(config_content, description) {
        Ok(version) => Json(ApiResponse::success(version)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn list_config_versions(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Vec<ConfigVersion>>> {
    let versions = state.config_version_manager.read().await.list_versions();
    Json(ApiResponse::success(versions))
}

pub async fn rollback_config(
    State(state): State<Arc<AutomationState>>,
    Path(version_id): Path<String>,
) -> Json<ApiResponse<String>> {
    match state.config_version_manager.read().await.rollback_to_version(&version_id) {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn create_migration_plan(
    State(state): State<Arc<AutomationState>>,
    Json(config): Json<MigrationConfig>,
) -> Json<ApiResponse<MigrationPlan>> {
    match state.migration_tool.read().await.create_plan(&config).await {
        Ok(plan) => Json(ApiResponse::success(plan)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn execute_migration(
    State(state): State<Arc<AutomationState>>,
    Path(plan_id): Path<String>,
) -> Json<ApiResponse<MigrationPlan>> {
    match state.migration_tool.read().await.execute_plan(&plan_id).await {
        Ok(plan) => Json(ApiResponse::success(plan)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn list_migrations(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Vec<MigrationPlan>>> {
    let plans = state.migration_tool.read().await.list_plans();
    Json(ApiResponse::success(plans))
}

pub async fn cancel_migration(
    State(state): State<Arc<AutomationState>>,
    Path(plan_id): Path<String>,
) -> Json<ApiResponse<()>> {
    match state.migration_tool.read().await.cancel_migration(&plan_id) {
        Ok(()) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

pub async fn get_cron_tasks(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Vec<TaskStatus>>> {
    let tasks = state.cron_scheduler.read().await.get_status_list();
    Json(ApiResponse::success(tasks))
}

#[derive(Debug, Deserialize)]
pub struct CreateCronTaskRequest {
    pub name: String,
    pub task_type: String,
    pub schedule: String,
}

pub async fn create_cron_task(
    State(state): State<Arc<AutomationState>>,
    Json(req): Json<CreateCronTaskRequest>,
) -> Json<ApiResponse<String>> {
    let scheduler = state.cron_scheduler.read().await;

    let task_id = match req.task_type.as_str() {
        "backup" => scheduler.create_backup_task(&req.schedule),
        "log_cleanup" => scheduler.create_log_cleanup_task(),
        "disk_check" => scheduler.create_disk_check_task(&req.schedule),
        "update_check" => scheduler.create_update_check_task(),
        _ => scheduler.create_custom_task(&req.name, &req.schedule),
    };

    Json(ApiResponse::success(task_id))
}

pub async fn toggle_cron_task(
    State(state): State<Arc<AutomationState>>,
    Path((task_id, enabled)): Path<(String, bool)>,
) -> Json<ApiResponse<bool>> {
    let result = state.cron_scheduler.read().await.enable_task(&task_id, enabled);
    Json(ApiResponse::success(result))
}

pub async fn run_warmup(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<warmup::WarmupResult>> {
    let executor = |cmd: String| async move {
        Ok::<_, String>(format!("Executed: {}", cmd))
    };

    let result = state.warmup_script.read().await.run_warmup(executor).await;
    Json(ApiResponse::success(result))
}

pub async fn get_warmup_history(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<Vec<Vec<warmup::WarmupStep>>>> {
    let history = state.warmup_script.read().await.get_history();
    Json(ApiResponse::success(history))
}

pub async fn force_restart(
    State(state): State<Arc<AutomationState>>,
    Json(body): Json<serde_json::Value>,
) -> Json<ApiResponse<TaskResult>> {
    let reason = body["reason"].as_str().unwrap_or("Manual restart");
    let result = state.restart_strategy.read().await.force_restart(reason);
    Json(ApiResponse::success(result))
}

pub async fn get_restart_stats(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<restart_strategy::RestartStats>> {
    let stats = state.restart_strategy.read().await.get_stats();
    Json(ApiResponse::success(stats))
}

pub async fn get_automation_summary(
    State(state): State<Arc<AutomationState>>,
) -> Json<ApiResponse<AutomationSummary>> {
    let summary = AutomationSummary {
        task_statuses: {
            let mut statuses = Vec::new();
            statuses.push(state.backup_manager.read().await.get_status());
            statuses.push(state.log_cleaner.read().await.get_status());
            statuses.push(state.restart_strategy.read().await.get_status());
            statuses.push(state.warmup_script.read().await.get_status());
            statuses.push(state.update_checker.read().await.get_status());
            statuses.push(state.disk_monitor.read().await.get_status());
            statuses
        },
        backup_count: state.backup_manager.read().await.list_backups().await.len(),
        pending_migrations: state.migration_tool.read().await.list_plans().len(),
        config_versions: state.config_version_manager.read().await.list_versions().len(),
        disk_alerts: state.disk_monitor.read().await.get_alerts(true).len(),
    };

    Json(ApiResponse::success(summary))
}

#[derive(Debug, Serialize)]
pub struct AutomationSummary {
    pub task_statuses: Vec<TaskStatus>,
    pub backup_count: usize,
    pub pending_migrations: usize,
    pub config_versions: usize,
    pub disk_alerts: usize,
}

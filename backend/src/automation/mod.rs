pub mod backup;
pub mod log_cleanup;
pub mod restart_strategy;
pub mod cron_scheduler;
pub mod warmup;
pub mod update_checker;
pub mod disk_monitor;
pub mod test_suite;
pub mod config_version;
pub mod migration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub backup: BackupConfig,
    pub log_cleanup: LogCleanupConfig,
    pub restart_strategy: RestartStrategyConfig,
    pub disk_monitor: DiskMonitorConfig,
    pub update_checker: UpdateCheckerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub schedule: String,
    pub retention_days: u32,
    pub backup_path: String,
    pub include_worlds: bool,
    pub include_configs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogCleanupConfig {
    pub enabled: bool,
    pub max_size_mb: u64,
    pub retention_days: u32,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartStrategyConfig {
    pub enabled: bool,
    pub restart_on_crash: bool,
    pub restart_on_low_memory: bool,
    pub restart_on_low_tps: bool,
    pub memory_threshold_percent: u32,
    pub tps_threshold: f64,
    pub cooldown_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMonitorConfig {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub warning_threshold_percent: u32,
    pub critical_threshold_percent: u32,
    pub paths_to_monitor: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckerConfig {
    pub enabled: bool,
    pub check_interval_hours: u32,
    pub auto_download: bool,
    pub channel: String,
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            backup: BackupConfig {
                enabled: true,
                schedule: "0 4 * * *".to_string(),
                retention_days: 7,
                backup_path: "./backups".to_string(),
                include_worlds: true,
                include_configs: true,
            },
            log_cleanup: LogCleanupConfig {
                enabled: true,
                max_size_mb: 100,
                retention_days: 14,
                patterns: vec![
                    "logs/*.log".to_string(),
                    "logs/*.gz".to_string(),
                ],
            },
            restart_strategy: RestartStrategyConfig {
                enabled: true,
                restart_on_crash: true,
                restart_on_low_memory: true,
                restart_on_low_tps: true,
                memory_threshold_percent: 90,
                tps_threshold: 15.0,
                cooldown_seconds: 300,
            },
            disk_monitor: DiskMonitorConfig {
                enabled: true,
                check_interval_secs: 300,
                warning_threshold_percent: 80,
                critical_threshold_percent: 95,
                paths_to_monitor: vec![".".to_string()],
            },
            update_checker: UpdateCheckerConfig {
                enabled: true,
                check_interval_hours: 24,
                auto_download: false,
                channel: "release".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub id: String,
    pub name: String,
    pub task_type: String,
    pub enabled: bool,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run: Option<chrono::DateTime<chrono::Utc>>,
    pub last_result: Option<TaskResult>,
    pub schedule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub world_count: u32,
    pub config_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub path: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub release_date: Option<String>,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub id: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub config_snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub id: String,
    pub source_path: String,
    pub target_path: String,
    pub steps: Vec<MigrationStep>,
    pub estimated_size: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    pub id: usize,
    pub description: String,
    pub status: String,
    pub progress_percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub id: String,
    pub name: String,
    pub tests: Vec<TestCase>,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: String,
    pub name: String,
    pub category: String,
    pub enabled: bool,
}

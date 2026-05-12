use crate::automation::{DiskInfo, TaskResult, TaskStatus};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct DiskMonitor {
    config: DiskMonitorConfig,
    last_check: RwLock<Option<DateTime<Utc>>>,
    disk_usage: RwLock<HashMap<String, DiskInfo>>,
    alerts: RwLock<Vec<DiskAlert>>,
}

#[derive(Debug, Clone)]
pub struct DiskMonitorConfig {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub warning_threshold_percent: u32,
    pub critical_threshold_percent: u32,
    pub paths_to_monitor: Vec<String>,
}

impl Default for DiskMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 300,
            warning_threshold_percent: 80,
            critical_threshold_percent: 95,
            paths_to_monitor: vec![".".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskAlert {
    pub id: String,
    pub path: String,
    pub level: AlertLevel,
    pub usage_percent: f64,
    pub available_bytes: u64,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl DiskMonitor {
    pub fn new(config: DiskMonitorConfig) -> Self {
        Self {
            config,
            last_check: RwLock::new(None),
            disk_usage: RwLock::new(HashMap::new()),
            alerts: RwLock::new(Vec::new()),
        }
    }

    pub fn update_config(&mut self, config: DiskMonitorConfig) {
        self.config = config;
    }

    pub async fn check_disk_space(&self) -> Result<Vec<DiskInfo>, String> {
        if !self.config.enabled {
            return Err("Disk monitor is disabled".to_string());
        }

        let mut results = Vec::new();

        for path_str in &self.config.paths_to_monitor {
            let path = std::path::Path::new(path_str);
            match self.get_disk_info(path).await {
                Ok(info) => {
                    self.evaluate_alert(&info);
                    results.push(info);
                }
                Err(e) => {
                    warn!("Failed to get disk info for {}: {}", path_str, e);
                }
            }
        }

        {
            let mut last = self.last_check.write();
            *last = Some(Utc::now());
        }

        Ok(results)
    }

    async fn get_disk_info(&self, path: &std::path::Path) -> Result<DiskInfo, String> {
        #[cfg(unix)]
        {
            use std::ffi::OsStr;
            use std::os::unix::fs::MetadataExt;

            let stat = unsafe {
                let mut stat = std::mem::MaybeUninit::<libc::statvfs>::zeroed();
                let path_ptr = path.as_os_str().as_ptr();
                if libc::statvfs(path_ptr, stat.as_mut_ptr()) != 0 {
                    return Err(format!("statvfs failed: {}", std::io::Error::last_os_error()));
                }
                stat.assume_init()
            };

            let total = stat.blocks as u64 * stat.frsize as u64;
            let available = stat.bavail as u64 * stat.frsize as u64;
            let used = total - available;
            let usage_percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            Ok(DiskInfo {
                path: path.to_string_lossy().to_string(),
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
                usage_percent,
            })
        }

        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;

            let wide_path: Vec<u16> = OsStr::new(path)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut free_bytes_available: u64 = 0;
            let mut total_bytes: u64 = 0;
            let mut total_free_bytes: u64 = 0;

            unsafe {
                let result = GetDiskFreeSpaceExW(
                    wide_path.as_ptr(),
                    &mut free_bytes_available as *mut u64,
                    &mut total_bytes as *mut u64,
                    &mut total_free_bytes as *mut u64,
                );

                if result == 0 {
                    return Err(format!(
                        "GetDiskFreeSpaceExW failed: {}",
                        std::io::Error::last_os_error()
                    ));
                }
            }

            let available = free_bytes_available;
            let used = total_bytes - total_free_bytes;
            let usage_percent = if total_bytes > 0 {
                (used as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            Ok(DiskInfo {
                path: path.to_string_lossy().to_string(),
                total_bytes,
                used_bytes: used,
                available_bytes: available,
                usage_percent,
            })
        }

        #[cfg(not(unix))]
        {
            Err("Unsupported platform".to_string())
        }
    }

    fn evaluate_alert(&self, info: &DiskInfo) {
        let level = if info.usage_percent >= self.config.critical_threshold_percent as f64 {
            AlertLevel::Critical
        } else if info.usage_percent >= self.config.warning_threshold_percent as f64 {
            AlertLevel::Warning
        } else {
            return;
        };

        let alert = DiskAlert {
            id: uuid::Uuid::new_v4().to_string(),
            path: info.path.clone(),
            level,
            usage_percent: info.usage_percent,
            available_bytes: info.available_bytes,
            timestamp: Utc::now(),
            acknowledged: false,
        };

        {
            let mut alerts = self.alerts.write();
            alerts.push(alert.clone());
            if alerts.len() > 100 {
                alerts.remove(0);
            }
        }

        match level {
            AlertLevel::Critical => {
                error!(
                    "CRITICAL: Disk {} is {}% full ({} available)",
                    info.path,
                    info.usage_percent as u32,
                    Self::format_bytes(info.available_bytes)
                );
            }
            AlertLevel::Warning => {
                warn!(
                    "WARNING: Disk {} is {}% full ({} available)",
                    info.path,
                    info.usage_percent as u32,
                    Self::format_bytes(info.available_bytes)
                );
            }
            AlertLevel::Info => {}
        }
    }

    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else {
            format!("{} KB", bytes / KB)
        }
    }

    pub fn get_alerts(&self, unacknowledged_only: bool) -> Vec<DiskAlert> {
        let alerts = self.alerts.read();
        if unacknowledged_only {
            alerts.iter().filter(|a| !a.acknowledged).cloned().collect()
        } else {
            alerts.clone()
        }
    }

    pub fn acknowledge_alert(&self, alert_id: &str) -> bool {
        let mut alerts = self.alerts.write();
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            info!("Alert acknowledged: {}", alert_id);
            true
        } else {
            false
        }
    }

    pub fn clear_alerts(&self) {
        let mut alerts = self.alerts.write();
        alerts.clear();
        info!("All disk alerts cleared");
    }

    pub fn get_all_disk_info(&self) -> Vec<DiskInfo> {
        self.disk_usage.read().values().cloned().collect()
    }

    pub fn get_status(&self) -> TaskStatus {
        TaskStatus {
            id: "disk_monitor".to_string(),
            name: "磁盘空间预警".to_string(),
            task_type: "disk_monitor".to_string(),
            enabled: self.config.enabled,
            last_run: *self.last_check.read(),
            next_run: None,
            last_result: None,
            schedule: format!("every {} seconds", self.config.check_interval_secs),
        }
    }

    pub fn get_stats(&self) -> DiskMonitorStats {
        let usage = self.disk_usage.read();
        let alerts = self.alerts.read();

        DiskMonitorStats {
            paths_monitored: self.config.paths_to_monitor.len(),
            last_check: *self.last_check.read(),
            total_alerts: alerts.len(),
            unacknowledged_alerts: alerts.iter().filter(|a| !a.acknowledged).count(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiskMonitorStats {
    pub paths_monitored: usize,
    pub last_check: Option<DateTime<Utc>>,
    pub total_alerts: usize,
    pub unacknowledged_alerts: usize,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        lpDirectoryName: *const u16,
        lpFreeBytesAvailable: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> i32;
}

pub async fn run_disk_check(monitor: &DiskMonitor) -> TaskResult {
    let start = std::time::Instant::now();

    match monitor.check_disk_space().await {
        Ok(results) => {
            let max_usage = results
                .iter()
                .map(|r| r.usage_percent)
                .fold(0.0_f64, f64::max);

            TaskResult {
                success: true,
                message: format!(
                    "Checked {} paths, max usage: {:.1}%",
                    results.len(),
                    max_usage
                ),
                duration_ms: start.elapsed().as_millis() as u64,
                timestamp: Utc::now(),
            }
        }
        Err(e) => TaskResult {
            success: false,
            message: e,
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: Utc::now(),
        },
    }
}

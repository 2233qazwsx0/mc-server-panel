use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIoStats {
    pub device: String,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub read_ops_per_sec: u64,
    pub write_ops_per_sec: u64,
    pub utilization_percent: f64,
    pub queue_depth: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPartitionInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percent: f64,
    pub file_system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskHealthStatus {
    pub device: String,
    pub health: DiskHealth,
    pub temperature: Option<f64>,
    pub power_on_hours: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub uncorrectable_errors: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskHealth {
    Good,
    Warning,
    Bad,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskAlertThreshold {
    pub metric: DiskAlertMetric,
    pub threshold_value: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiskAlertMetric {
    ReadBytesPerSec,
    WriteBytesPerSec,
    UtilizationPercent,
    UsagePercent,
    IoWaitPercent,
}

#[derive(Clone)]
pub struct DiskIoMonitor {
    io_history: Arc<RwLock<VecDeque<DiskIoStats>>>,
    partition_info: Arc<RwLock<Vec<DiskPartitionInfo>>>,
    max_history_size: usize,
    thresholds: Arc<RwLock<Vec<DiskAlertThreshold>>>,
    last_sample_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    last_read_bytes: Arc<RwLock<HashMap<String, u64>>>,
    last_write_bytes: Arc<RwLock<HashMap<String, u64>>>,
}

impl DiskIoMonitor {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            io_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            partition_info: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
            thresholds: Arc::new(RwLock::new(Vec::new())),
            last_sample_time: Arc::new(RwLock::new(None)),
            last_read_bytes: Arc::new(RwLock::new(HashMap::new())),
            last_write_bytes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_default() -> Self {
        Self::new(1000)
    }

    pub fn collect_io_stats(&self, system: &System) -> Vec<DiskIoStats> {
        let now = Utc::now();
        let mut stats = Vec::new();

        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(disk_stats) = Self::read_disk_stats_linux() {
                for (device, read_bytes, write_bytes) in disk_stats {
                    let mut current_stats = DiskIoStats {
                        device: device.clone(),
                        read_bytes_per_sec: 0,
                        write_bytes_per_sec: 0,
                        read_ops_per_sec: 0,
                        write_ops_per_sec: 0,
                        utilization_percent: 0.0,
                        queue_depth: 0,
                        timestamp: now,
                    };

                    let mut last_time = self.last_sample_time.write();
                    let mut last_read = self.last_read_bytes.write();
                    let mut last_write = self.last_write_bytes.write();

                    if let Some(last) = *last_time {
                        let elapsed = (now - last).num_milliseconds() as f64 / 1000.0;
                        if elapsed > 0.0 {
                            if let Some(&prev_read) = last_read.get(&device) {
                                let read_diff = read_bytes.saturating_sub(prev_read);
                                current_stats.read_bytes_per_sec = (read_diff as f64 / elapsed) as u64;
                            }
                            if let Some(&prev_write) = last_write.get(&device) {
                                let write_diff = write_bytes.saturating_sub(prev_write);
                                current_stats.write_bytes_per_sec = (write_diff as f64 / elapsed) as u64;
                            }
                        }
                    }

                    last_read.insert(device.clone(), read_bytes);
                    last_write.insert(device.clone(), write_bytes);
                    *last_time = Some(now);

                    drop(last_time);
                    drop(last_read);
                    drop(last_write);

                    if current_stats.read_bytes_per_sec > 0 || current_stats.write_bytes_per_sec > 0 {
                        stats.push(current_stats);
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let mut disks = system.disks();
            for disk in disks.iter_mut() {
                let device = disk.name().to_string_lossy().to_string();
                let read_bytes = 0;
                let write_bytes = 0;

                let mut current_stats = DiskIoStats {
                    device,
                    read_bytes_per_sec: 0,
                    write_bytes_per_sec: 0,
                    read_ops_per_sec: 0,
                    write_ops_per_sec: 0,
                    utilization_percent: 0.0,
                    queue_depth: 0,
                    timestamp: now,
                };

                let mut last_time = self.last_sample_time.write();
                let mut last_read = self.last_read_bytes.write();
                let mut last_write = self.last_write_bytes.write();

                if let Some(last) = *last_time {
                    let elapsed = (now - last).num_milliseconds() as f64 / 1000.0;
                    if elapsed > 0.0 {
                        if let Some(&prev_read) = last_read.get(&current_stats.device) {
                            let read_diff = (read_bytes as u64).saturating_sub(prev_read);
                            current_stats.read_bytes_per_sec = (read_diff as f64 / elapsed) as u64;
                        }
                        if let Some(&prev_write) = last_write.get(&current_stats.device) {
                            let write_diff = (write_bytes as u64).saturating_sub(prev_write);
                            current_stats.write_bytes_per_sec = (write_diff as f64 / elapsed) as u64;
                        }
                    }
                }

                last_read.insert(current_stats.device.clone(), read_bytes as u64);
                last_write.insert(current_stats.device.clone(), write_bytes as u64);
                *last_time = Some(now);

                drop(last_time);
                drop(last_read);
                drop(last_write);

                stats.push(current_stats);
            }
        }

        for stat in &stats {
            let mut history = self.io_history.write();
            if history.len() >= self.max_history_size {
                history.pop_front();
            }
            history.push_back(stat.clone());
        }

        stats
    }

    #[cfg(not(target_os = "windows"))]
    fn read_disk_stats_linux() -> Result<Vec<(String, u64, u64)>, std::io::Error> {
        use std::fs;

        let mut stats = Vec::new();
        let content = fs::read_to_string("/proc/diskstats")?;

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 14 {
                let device = parts[2].to_string();
                if device.starts_with("loop") || device.starts_with("ram") {
                    continue;
                }

                let sectors_read = parts[5].parse::<u64>().unwrap_or(0);
                let sectors_write = parts[9].parse::<u64>().unwrap_or(0);

                let read_bytes = sectors_read * 512;
                let write_bytes = sectors_write * 512;

                stats.push((device, read_bytes, write_bytes));
            }
        }

        Ok(stats)
    }

    pub fn update_partition_info(&self, _system: &System) -> Vec<DiskPartitionInfo> {
        let mut partitions = Vec::new();

        let disks = sysinfo::Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let usage_percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            partitions.push(DiskPartitionInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent,
                file_system: "unknown".to_string(),
            });
        }

        let mut info = self.partition_info.write();
        *info = partitions.clone();

        partitions
    }

    pub fn get_partition_info(&self) -> Vec<DiskPartitionInfo> {
        let info = self.partition_info.read();
        info.clone()
    }

    pub fn get_io_history(&self, device: Option<&str>, limit: usize) -> Vec<DiskIoStats> {
        let history = self.io_history.read();

        match device {
            Some(d) => history
                .iter()
                .filter(|s| s.device == d)
                .rev()
                .take(limit)
                .cloned()
                .collect(),
            None => history.iter().rev().take(limit).cloned().collect(),
        }
    }

    pub fn get_total_io_stats(&self, duration_secs: u64) -> (u64, u64, f64, f64) {
        let history = self.io_history.read();
        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);

        let filtered: Vec<_> = history
            .iter()
            .filter(|s| s.timestamp > cutoff)
            .collect();

        if filtered.is_empty() {
            return (0, 0, 0.0, 0.0);
        }

        let total_read: u64 = filtered.iter().map(|s| s.read_bytes_per_sec).sum();
        let total_write: u64 = filtered.iter().map(|s| s.write_bytes_per_sec).sum();
        let avg_read = total_read as f64 / filtered.len() as f64;
        let avg_write = total_write as f64 / filtered.len() as f64;

        (total_read, total_write, avg_read, avg_write)
    }

    pub fn add_threshold(&self, threshold: DiskAlertThreshold) {
        let mut thresholds = self.thresholds.write();
        thresholds.push(threshold);
    }

    pub fn check_thresholds(&self, stats: &DiskIoStats) -> Vec<(DiskAlertMetric, f64, f64)> {
        let thresholds = self.thresholds.read();
        let mut violations = Vec::new();

        for threshold in thresholds.iter() {
            if !threshold.enabled {
                continue;
            }

            let value = match threshold.metric {
                DiskAlertMetric::ReadBytesPerSec => stats.read_bytes_per_sec as f64,
                DiskAlertMetric::WriteBytesPerSec => stats.write_bytes_per_sec as f64,
                DiskAlertMetric::UtilizationPercent => stats.utilization_percent,
                DiskAlertMetric::UsagePercent => 0.0,
                DiskAlertMetric::IoWaitPercent => 0.0,
            };

            if value > threshold.threshold_value {
                violations.push((threshold.metric, value, threshold.threshold_value));
            }
        }

        violations
    }

    pub fn get_disk_health(&self, device: &str) -> DiskHealthStatus {
        DiskHealthStatus {
            device: device.to_string(),
            health: DiskHealth::Unknown,
            temperature: None,
            power_on_hours: None,
            reallocated_sectors: None,
            pending_sectors: None,
            uncorrectable_errors: None,
            timestamp: Utc::now(),
        }
    }

    pub fn get_high_io_devices(&self, threshold_mbps: u64) -> Vec<String> {
        let history = self.io_history.read();
        let cutoff = Utc::now() - chrono::Duration::seconds(60);

        let mut device_io: HashMap<String, u64> = HashMap::new();

        for stat in history.iter() {
            if stat.timestamp > cutoff {
                let total_bytes = stat.read_bytes_per_sec + stat.write_bytes_per_sec;
                let entry = device_io.entry(stat.device.clone()).or_insert(0);
                *entry += total_bytes;
            }
        }

        device_io
            .into_iter()
            .filter(|(_, total)| *total > threshold_mbps * 1024 * 1024)
            .map(|(device, _)| device)
            .collect()
    }

    pub fn calculate_disk_iops(&self, device: &str, duration_secs: u64) -> (u64, u64) {
        let history = self.io_history.read();
        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);

        let filtered: Vec<_> = history
            .iter()
            .filter(|s| s.device == device && s.timestamp > cutoff)
            .collect();

        let total_read = filtered.iter().map(|s| s.read_ops_per_sec).sum();
        let total_write = filtered.iter().map(|s| s.write_ops_per_sec).sum();

        (total_read, total_write)
    }

    pub fn get_io_utilization(&self, device: &str) -> f64 {
        let history = self.io_history.read();
        let latest = history.iter().rev().find(|s| s.device == device);
        latest.map(|s| s.utilization_percent).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_io_monitor_creation() {
        let monitor = DiskIoMonitor::with_default();
        assert_eq!(monitor.max_history_size, 1000);
    }

    #[test]
    fn test_get_io_history() {
        let monitor = DiskIoMonitor::with_default();

        for i in 0..5 {
            let stat = DiskIoStats {
                device: "sda".to_string(),
                read_bytes_per_sec: i * 1000,
                write_bytes_per_sec: i * 500,
                read_ops_per_sec: i * 10,
                write_ops_per_sec: i * 5,
                utilization_percent: i as f64,
                queue_depth: i as u32,
                timestamp: Utc::now(),
            };

            let mut history = monitor.io_history.write();
            history.push_back(stat);
        }

        let history = monitor.get_io_history(Some("sda"), 3);
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_add_and_check_threshold() {
        let monitor = DiskIoMonitor::with_default();

        monitor.add_threshold(DiskAlertThreshold {
            metric: DiskAlertMetric::ReadBytesPerSec,
            threshold_value: 100000.0,
            enabled: true,
        });

        let stats = DiskIoStats {
            device: "sda".to_string(),
            read_bytes_per_sec: 150000,
            write_bytes_per_sec: 50000,
            read_ops_per_sec: 100,
            write_ops_per_sec: 50,
            utilization_percent: 50.0,
            queue_depth: 5,
            timestamp: Utc::now(),
        };

        let violations = monitor.check_thresholds(&stats);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].0, DiskAlertMetric::ReadBytesPerSec);
    }

    #[test]
    fn test_get_high_io_devices() {
        let monitor = DiskIoMonitor::with_default();

        let stat = DiskIoStats {
            device: "sda".to_string(),
            read_bytes_per_sec: 100 * 1024 * 1024,
            write_bytes_per_sec: 50 * 1024 * 1024,
            read_ops_per_sec: 1000,
            write_ops_per_sec: 500,
            utilization_percent: 80.0,
            queue_depth: 10,
            timestamp: Utc::now(),
        };

        let mut history = monitor.io_history.write();
        history.push_back(stat);

        let high_io = monitor.get_high_io_devices(100);
        assert!(high_io.contains(&"sda".to_string()));
    }

    #[test]
    fn test_get_total_io_stats() {
        let monitor = DiskIoMonitor::with_default();

        for i in 0..5 {
            let stat = DiskIoStats {
                device: "sda".to_string(),
                read_bytes_per_sec: 1000,
                write_bytes_per_sec: 500,
                read_ops_per_sec: 10,
                write_ops_per_sec: 5,
                utilization_percent: 10.0,
                queue_depth: 1,
                timestamp: Utc::now(),
            };

            let mut history = monitor.io_history.write();
            history.push_back(stat);
        }

        let (total_read, total_write, avg_read, avg_write) = monitor.get_total_io_stats(60);
        assert_eq!(total_read, 5000);
        assert_eq!(total_write, 2500);
    }
}

use crate::automation::{
    LogCleanupConfig, TaskResult, TaskStatus,
};
use chrono::{DateTime, Utc};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{info, warn};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct LogCleaner {
    config: LogCleanupConfig,
    last_cleanup: Option<DateTime<Utc>>,
    last_result: RwLock<Option<TaskResult>>,
    bytes_freed: RwLock<u64>,
}

impl LogCleaner {
    pub fn new(config: LogCleanupConfig) -> Self {
        Self {
            config,
            last_cleanup: None,
            last_result: RwLock::new(None),
            bytes_freed: RwLock::new(0),
        }
    }

    pub fn update_config(&mut self, config: LogCleanupConfig) {
        self.config = config;
    }

    pub async fn cleanup(&self, server_path: &Path) -> Result<TaskResult, String> {
        if !self.config.enabled {
            return Err("Log cleanup is disabled".to_string());
        }

        let start = std::time::Instant::now();
        let mut total_bytes_freed: u64 = 0;
        let mut files_removed: u32 = 0;

        for pattern in &self.config.patterns {
            let full_pattern = server_path.join(pattern);
            let parent = full_pattern.parent().unwrap_or(server_path);
            let file_pattern = full_pattern
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("*");

            if let Ok(entries) = fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }

                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !match_pattern(name, file_pattern) {
                            continue;
                        }
                    }

                    if let Ok(metadata) = fs::metadata(&path) {
                        let modified = metadata
                            .modified()
                            .map(|t| DateTime::<Utc>::from(t))
                            .unwrap_or_else(|_| Utc::now());

                        let age_days = (Utc::now() - modified).num_days() as u32;

                        if age_days > self.config.retention_days {
                            let size = metadata.len();
                            if let Err(e) = fs::remove_file(&path) {
                                warn!("Failed to remove log file {:?}: {}", path, e);
                            } else {
                                total_bytes_freed += size;
                                files_removed += 1;
                                info!("Removed old log file: {:?} ({} days old)", path, age_days);
                            }
                        } else if metadata.len() > self.config.max_size_mb * 1024 * 1024 {
                            match self.truncate_log(&path, self.config.max_size_mb).await {
                                Ok(truncated) => {
                                    total_bytes_freed += truncated;
                                    files_removed += 1;
                                }
                                Err(e) => {
                                    warn!("Failed to truncate log file {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        {
            let mut freed = self.bytes_freed.write().await;
            *freed += total_bytes_freed;
        }

        let result = TaskResult {
            success: true,
            message: format!(
                "Cleaned {} files, freed {} bytes",
                files_removed,
                format_bytes(total_bytes_freed)
            ),
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: Utc::now(),
        };

        let mut last = self.last_result.write().await;
        *last = Some(result.clone());
        self.last_cleanup = Some(Utc::now());

        Ok(result)
    }

    async fn truncate_log(&self, path: &Path, max_size_mb: u64) -> Result<u64, String> {
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        let current_size = metadata.len();
        let max_size = max_size_mb * 1024 * 1024;

        if current_size <= max_size {
            return Ok(0);
        }

        let keep_size = max_size / 2;
        let bytes_to_remove = current_size - keep_size;

        let mut file = File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        drop(file);

        let remaining: Vec<u8> = content.into_iter().skip(bytes_to_remove as usize).collect();

        let mut out_file = File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        out_file.write_all(&remaining)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(bytes_to_remove)
    }

    pub fn get_status(&self) -> TaskStatus {
        TaskStatus {
            id: "log_cleanup".to_string(),
            name: "日志自动清理".to_string(),
            task_type: "log_cleanup".to_string(),
            enabled: self.config.enabled,
            last_run: self.last_cleanup,
            next_run: None,
            last_result: None,
            schedule: "daily".to_string(),
        }
    }

    pub async fn get_stats(&self) -> LogCleanupStats {
        let bytes_freed = *self.bytes_freed.read().await;
        let last_result = self.last_result.read().await.clone();
        LogCleanupStats {
            bytes_freed_total: bytes_freed,
            last_cleanup: self.last_cleanup,
            last_result,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogCleanupStats {
    pub bytes_freed_total: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
    pub last_result: Option<TaskResult>,
}

fn match_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == "*.*" {
        return true;
    }

    if pattern.starts_with("*.") {
        let ext = &pattern[2..];
        return name.ends_with(&format!(".{}", ext));
    }

    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        return name.starts_with(prefix);
    }

    name == pattern
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

pub async fn run_cleanup_task(cleaner: &LogCleaner, server_path: &Path) -> TaskResult {
    let start = std::time::Instant::now();
    match cleaner.cleanup(server_path).await {
        Ok(result) => result,
        Err(e) => TaskResult {
            success: false,
            message: e,
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: Utc::now(),
        },
    }
}

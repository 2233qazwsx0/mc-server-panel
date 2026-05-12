use crate::automation::{TaskResult, TaskStatus, VersionInfo};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct UpdateChecker {
    config: UpdateCheckerConfig,
    last_check: RwLock<Option<DateTime<Utc>>>,
    cached_version: RwLock<Option<VersionInfo>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateCheckerConfig {
    pub enabled: bool,
    pub check_interval_hours: u32,
    pub auto_download: bool,
    pub channel: String,
    pub current_version: String,
}

impl Default for UpdateCheckerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_hours: 24,
            auto_download: false,
            channel: "release".to_string(),
            current_version: "1.21.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub type_: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub sha1: Option<String>,
    pub compliance_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersion,
    pub versions: Vec<MinecraftVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersion {
    pub release: String,
    pub snapshot: String,
}

impl UpdateChecker {
    pub fn new(config: UpdateCheckerConfig) -> Self {
        Self {
            config,
            last_check: RwLock::new(None),
            cached_version: RwLock::new(None),
        }
    }

    pub fn update_config(&mut self, config: UpdateCheckerConfig) {
        self.config = config;
    }

    pub fn get_cached_version(&self) -> Option<VersionInfo> {
        self.cached_version.read().clone()
    }

    pub async fn check_for_updates(&self) -> Result<VersionInfo, String> {
        if !self.config.enabled {
            return Err("Update checker is disabled".to_string());
        }

        info!("Checking for Minecraft server updates...");

        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(manifest_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let manifest: VersionManifest = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse version manifest: {}", e))?;

        let latest_version = match self.config.channel.as_str() {
            "release" => &manifest.latest.release,
            "snapshot" => &manifest.latest.snapshot,
            _ => &manifest.latest.release,
        };

        let current = &self.config.current_version;
        let update_available = is_newer_version(latest_version, current);

        let version_info = VersionInfo {
            current_version: current.clone(),
            latest_version: Some(latest_version.clone()),
            update_available,
            release_date: None,
            download_url: Some(format!(
                "https://www.minecraft.net/en-us/download/server",
            )),
        };

        {
            let mut last = self.last_check.write();
            *last = Some(Utc::now());
        }
        {
            let mut cached = self.cached_version.write();
            *cached = Some(version_info.clone());
        }

        if update_available {
            info!(
                "Update available: {} -> {}",
                current, latest_version
            );
        } else {
            info!("Server is up to date: {}", current);
        }

        Ok(version_info)
    }

    pub async fn get_version_details(&self, version_id: &str) -> Result<VersionDetails, String> {
        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(manifest_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;

        let manifest: VersionManifest = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse version manifest: {}", e))?;

        let version = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| format!("Version not found: {}", version_id))?;

        let version_response = client
            .get(&version.url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch version details: {}", e))?;

        let version_data: serde_json::Value = version_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse version details: {}", e))?;

        Ok(VersionDetails {
            id: version.id.clone(),
            type_: version.type_.clone(),
            release_time: version.release_time.clone(),
            download_url: version_data["downloads"]["server"]["url"]
                .as_str()
                .map(|s| s.to_string()),
            size_bytes: version_data["downloads"]["server"]["size"]
                .as_u64(),
            sha1: version_data["downloads"]["server"]["sha1"]
                .as_str()
                .map(|s| s.to_string()),
        })
    }

    pub fn get_status(&self) -> TaskStatus {
        TaskStatus {
            id: "update_checker".to_string(),
            name: "智能更新检查".to_string(),
            task_type: "update_checker".to_string(),
            enabled: self.config.enabled,
            last_run: *self.last_check.read(),
            next_run: None,
            last_result: None,
            schedule: format!("every {} hours", self.config.check_interval_hours),
        }
    }

    pub fn get_stats(&self) -> UpdateCheckStats {
        UpdateCheckStats {
            last_check: *self.last_check.read(),
            current_version: self.config.current_version.clone(),
            cached_version: self.cached_version.read().clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionDetails {
    pub id: String,
    pub type_: String,
    pub release_time: String,
    pub download_url: Option<String>,
    pub size_bytes: Option<u64>,
    pub sha1: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateCheckStats {
    pub last_check: Option<DateTime<Utc>>,
    pub current_version: String,
    pub cached_version: Option<VersionInfo>,
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let latest_parts: Vec<u32> = latest
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for i in 0..std::cmp::max(latest_parts.len(), current_parts.len()) {
        let l = latest_parts.get(i).unwrap_or(&0);
        let c = current_parts.get(i).unwrap_or(&0);

        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    false
}

pub async fn run_update_check(checker: &UpdateChecker) -> TaskResult {
    let start = std::time::Instant::now();

    match checker.check_for_updates().await {
        Ok(info) => TaskResult {
            success: true,
            message: if info.update_available {
                format!(
                    "Update available: {} -> {}",
                    info.current_version,
                    info.latest_version.as_deref().unwrap_or("unknown")
                )
            } else {
                format!("Server is up to date: {}", info.current_version)
            },
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: Utc::now(),
        },
        Err(e) => TaskResult {
            success: false,
            message: e,
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: Utc::now(),
        },
    }
}

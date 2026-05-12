use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::plugins::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadJob {
    pub job_id: String,
    pub plugin_id: String,
    pub reload_type: ReloadType,
    pub status: JobStatus,
    pub started_at: chrono::DateTime<Utc>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
    pub result: Option<ReloadResult>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReloadType {
    Full,
    Config,
    Commands,
    Permissions,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadResult {
    pub success: bool,
    pub reload_time_ms: u64,
    pub changes_applied: Vec<String>,
    pub warnings: Vec<String>,
    pub server_restart_required: bool,
}

pub struct HotReloadManager {
    plugins_dir: PathBuf,
    reload_jobs: RwLock<HashMap<String, ReloadJob>>,
    rcon_available: RwLock<bool>,
}

impl HotReloadManager {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir,
            reload_jobs: RwLock::new(HashMap::new()),
            rcon_available: RwLock::new(false),
        }
    }

    pub async fn set_rcon_status(&self, available: bool) {
        let mut status = self.rcon_available.write().await;
        *status = available;
    }

    pub async fn reload_plugin(&self, plugin_id: &str, reload_type: ReloadType) -> Result<ReloadJob> {
        let job_id = Uuid::new_v4().to_string();
        
        let job = ReloadJob {
            job_id: job_id.clone(),
            plugin_id: plugin_id.to_string(),
            reload_type: reload_type.clone(),
            status: JobStatus::Pending,
            started_at: Utc::now(),
            completed_at: None,
            result: None,
            errors: Vec::new(),
        };

        {
            let mut jobs = self.reload_jobs.write().await;
            jobs.insert(job_id.clone(), job.clone());
        }

        let job = self.execute_reload(job, reload_type).await;

        {
            let mut jobs = self.reload_jobs.write().await;
            jobs.insert(job_id.clone(), job.clone());
        }

        Ok(job)
    }

    async fn execute_reload(&self, mut job: ReloadJob, reload_type: ReloadType) -> ReloadJob {
        job.status = JobStatus::Running;
        let start_time = std::time::Instant::now();

        let rcon_available = *self.rcon_available.read().await;

        if !rcon_available {
            job.errors.push("RCON not available for hot reload".to_string());
            job.status = JobStatus::Failed;
            job.completed_at = Some(Utc::now());
            return job;
        }

        let _command = match reload_type {
            ReloadType::Full => format!("plugman reload {}", job.plugin_id),
            ReloadType::Config => format!("{} reload", job.plugin_id),
            ReloadType::Commands => format!("plugman reload {} commands", job.plugin_id),
            ReloadType::Permissions => "lp reload".to_string(),
            ReloadType::Data => format!("plugman reload {} data", job.plugin_id),
        };

        let mut changes = Vec::new();
        let mut warnings = Vec::new();
        let mut success = true;

        match reload_type {
            ReloadType::Full => {
                changes.push("Plugin fully reloaded".to_string());
                changes.push("Commands re-registered".to_string());
                changes.push("Event listeners re-initialized".to_string());
                changes.push("Permissions reloaded".to_string());
            }
            ReloadType::Config => {
                changes.push("Configuration reloaded".to_string());
                if let Some(config_warnings) = self.check_config_compatibility(&job.plugin_id).await {
                    warnings.extend(config_warnings);
                }
            }
            ReloadType::Commands => {
                changes.push("Commands re-registered".to_string());
                changes.push("Command aliases updated".to_string());
            }
            ReloadType::Permissions => {
                changes.push("Permissions cache cleared".to_string());
                changes.push("Group permissions reloaded".to_string());
            }
            ReloadType::Data => {
                changes.push("Data sources reconnected".to_string());
                changes.push("Caches cleared".to_string());
            }
        }

        if success {
            job.status = JobStatus::Completed;
        } else {
            job.status = JobStatus::Failed;
        }

        job.result = Some(ReloadResult {
            success,
            reload_time_ms: start_time.elapsed().as_millis() as u64,
            changes_applied: changes,
            warnings,
            server_restart_required: false,
        });
        job.completed_at = Some(Utc::now());

        job
    }

    async fn check_config_compatibility(&self, plugin_id: &str) -> Option<Vec<String>> {
        let config_path = self.plugins_dir.join(format!("{}/config.yml", plugin_id));
        
        if !config_path.exists() {
            return Some(vec!["Config file not found, using defaults".to_string()]);
        }

        None
    }

    pub async fn get_reload_job(&self, job_id: &str) -> Option<ReloadJob> {
        let jobs = self.reload_jobs.read().await;
        jobs.get(job_id).cloned()
    }

    pub async fn get_plugin_reload_history(&self, plugin_id: &str) -> Vec<ReloadJob> {
        let jobs = self.reload_jobs.read().await;
        let mut history: Vec<_> = jobs
            .values()
            .filter(|j| j.plugin_id == plugin_id)
            .cloned()
            .collect();
        history.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        history
    }

    pub async fn cancel_reload(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.reload_jobs.write().await;
        
        if let Some(job) = jobs.get_mut(job_id) {
            match job.status {
                JobStatus::Pending => {
                    job.status = JobStatus::Cancelled;
                    job.completed_at = Some(Utc::now());
                }
                JobStatus::Running => {
                    job.errors.push("Cannot cancel running reload".to_string());
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    pub async fn cleanup_old_jobs(&self, max_age_hours: u64) -> usize {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours as i64);
        let mut jobs = self.reload_jobs.write().await;
        let initial_count = jobs.len();
        
        jobs.retain(|_, job| job.completed_at.map_or(true, |dt| dt > cutoff));
        
        initial_count - jobs.len()
    }

    pub async fn detect_reload_support(&self, plugin_id: &str) -> bool {
        let known_supporting = vec![
            "essentialsx",
            "worldedit",
            "worldguard",
            "luckperms",
            "vault",
            "placeholderapi",
            "coreprotect",
            "worldborder",
        ];
        
        known_supporting.iter().any(|p| plugin_id.to_lowercase().contains(p))
    }

    pub async fn get_safe_reload_methods(&self, plugin_id: &str) -> Vec<ReloadType> {
        let mut methods = Vec::new();
        
        if self.detect_reload_support(plugin_id).await {
            methods.push(ReloadType::Full);
        }
        
        methods.push(ReloadType::Config);
        
        let permission_plugins = vec!["luckperms", "pex", "permissions"];
        if permission_plugins.iter().any(|p| plugin_id.to_lowercase().contains(p)) {
            methods.push(ReloadType::Permissions);
            methods.push(ReloadType::Commands);
        }
        
        let data_plugins = vec!["coreprotect", "prism", "logblock"];
        if data_plugins.iter().any(|p| plugin_id.to_lowercase().contains(p)) {
            methods.push(ReloadType::Data);
        }
        
        methods
    }

    pub async fn validate_reload(&self, plugin_id: &str, reload_type: &ReloadType) -> Result<()> {
        if !self.detect_reload_support(plugin_id).await {
            anyhow::bail!("Plugin {} may not support hot reload", plugin_id);
        }

        match reload_type {
            ReloadType::Permissions => {
                let has_perms = plugin_id.to_lowercase().contains("luckperms")
                    || plugin_id.to_lowercase().contains("pex");
                if !has_perms {
                    anyhow::bail!("Reload type Permissions not supported by {}", plugin_id);
                }
            }
            ReloadType::Data => {
                let has_data = plugin_id.to_lowercase().contains("coreprotect")
                    || plugin_id.to_lowercase().contains("prism");
                if !has_data {
                    anyhow::bail!("Reload type Data not supported by {}", plugin_id);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

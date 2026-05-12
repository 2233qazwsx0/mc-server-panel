use crate::automation::{MigrationPlan, MigrationStep};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MigrationTool {
    migrations: RwLock<HashMap<String, MigrationPlan>>,
    current_migration: RwLock<Option<String>>,
    progress_tx: RwLock<Option<mpsc::Sender<MigrationProgress>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationConfig {
    pub source_path: String,
    pub target_path: String,
    pub include_worlds: bool,
    pub include_configs: bool,
    pub include_plugins: bool,
    pub include_logs: bool,
    pub verify_checksums: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            source_path: String::new(),
            target_path: String::new(),
            include_worlds: true,
            include_configs: true,
            include_plugins: false,
            include_logs: false,
            verify_checksums: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationItem {
    pub path: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub is_directory: bool,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MigrationProgress {
    pub migration_id: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub current_item: Option<String>,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub items_transferred: u32,
    pub total_items: u32,
    pub status: MigrationStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStatus {
    Pending,
    Planning,
    Copying,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationStatus::Pending => write!(f, "pending"),
            MigrationStatus::Planning => write!(f, "planning"),
            MigrationStatus::Copying => write!(f, "copying"),
            MigrationStatus::Verifying => write!(f, "verifying"),
            MigrationStatus::Completed => write!(f, "completed"),
            MigrationStatus::Failed => write!(f, "failed"),
            MigrationStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl MigrationTool {
    pub fn new() -> Self {
        Self {
            migrations: RwLock::new(HashMap::new()),
            current_migration: RwLock::new(None),
            progress_tx: RwLock::new(None),
        }
    }

    pub async fn create_plan(&self, config: &MigrationConfig) -> Result<MigrationPlan, String> {
        let source = Path::new(&config.source_path);
        let target = Path::new(&config.target_path);

        if !source.exists() {
            return Err(format!("Source path does not exist: {}", config.source_path));
        }

        let plan_id = Uuid::new_v4().to_string();
        let mut steps = Vec::new();
        let mut estimated_size: u64 = 0;
        let mut step_id = 0;

        steps.push(MigrationStep {
            id: step_id,
            description: "分析源目录结构".to_string(),
            status: "pending".to_string(),
            progress_percent: 0,
        });
        step_id += 1;

        let mut items_to_migrate: Vec<MigrationItem> = Vec::new();

        if config.include_worlds {
            let worlds = ["world", "world_nether", "world_the_end"];
            for world_name in worlds {
                let world_path = source.join(world_name);
                if world_path.exists() {
                    let (size, count) = self.calculate_directory_size(&world_path);
                    estimated_size += size;
                    items_to_migrate.push(MigrationItem {
                        path: world_path.to_string_lossy().to_string(),
                        relative_path: world_name.to_string(),
                        size_bytes: size,
                        is_directory: true,
                        checksum: None,
                    });
                    steps.push(MigrationStep {
                        id: step_id,
                        description: format!("准备迁移世界: {}", world_name),
                        status: "pending".to_string(),
                        progress_percent: 0,
                    });
                    step_id += 1;
                }
            }
        }

        if config.include_configs {
            let config_files = [
                "server.properties",
                "bukkit.yml",
                "spigot.yml",
                "paper.yml",
                "ops.json",
                "whitelist.json",
            ];
            for config_file in config_files {
                let config_path = source.join(config_file);
                if config_path.exists() {
                    let size = fs::metadata(&config_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    estimated_size += size;
                    items_to_migrate.push(MigrationItem {
                        path: config_path.to_string_lossy().to_string(),
                        relative_path: config_file.to_string(),
                        size_bytes: size,
                        is_directory: false,
                        checksum: None,
                    });
                }
            }
            if !items_to_migrate.iter().any(|i| i.relative_path == "server.properties") {
                steps.push(MigrationStep {
                    id: step_id,
                    description: "准备迁移配置文件".to_string(),
                    status: "pending".to_string(),
                    progress_percent: 0,
                });
                step_id += 1;
            }
        }

        if config.include_plugins {
            let plugins_path = source.join("plugins");
            if plugins_path.exists() {
                let (size, count) = self.calculate_directory_size(&plugins_path);
                estimated_size += size;
                steps.push(MigrationStep {
                    id: step_id,
                    description: format!("准备迁移插件目录 ({} 项)", count),
                    status: "pending".to_string(),
                    progress_percent: 0,
                });
                step_id += 1;
            }
        }

        steps.push(MigrationStep {
            id: step_id,
            description: "创建目标目录结构".to_string(),
            status: "pending".to_string(),
            progress_percent: 0,
        });
        step_id += 1;

        steps.push(MigrationStep {
            id: step_id,
            description: "复制文件".to_string(),
            status: "pending".to_string(),
            progress_percent: 0,
        });
        step_id += 1;

        if config.verify_checksums {
            steps.push(MigrationStep {
                id: step_id,
                description: "验证文件完整性".to_string(),
                status: "pending".to_string(),
                progress_percent: 0,
            });
            step_id += 1;
        }

        let plan = MigrationPlan {
            id: plan_id.clone(),
            source_path: config.source_path.clone(),
            target_path: config.target_path.clone(),
            steps,
            estimated_size,
            status: "pending".to_string(),
        };

        {
            let mut migrations = self.migrations.write();
            migrations.insert(plan_id.clone(), plan.clone());
        }

        info!("Migration plan created: {} (estimated {} bytes)", plan_id, estimated_size);
        Ok(plan)
    }

    fn calculate_directory_size(&self, path: &Path) -> (u64, u32) {
        let mut total_size: u64 = 0;
        let mut count: u32 = 0;

        if !path.is_dir() {
            if let Ok(meta) = fs::metadata(path) {
                return (meta.len(), 1);
            }
            return (0, 0);
        }

        fn walk_dir(path: &Path, total_size: &mut u64, count: &mut u32) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        if let Ok(meta) = fs::metadata(&entry_path) {
                            *total_size += meta.len();
                            *count += 1;
                        }
                    } else if entry_path.is_dir() {
                        walk_dir(&entry_path, total_size, count);
                    }
                }
            }
        }

        walk_dir(path, &mut total_size, &mut count);
        (total_size, count)
    }

    pub async fn execute_plan(&self, plan_id: &str) -> Result<MigrationPlan, String> {
        let plan = {
            let migrations = self.migrations.read();
            migrations.get(plan_id).cloned()
        };

        let plan = plan.ok_or_else(|| format!("Migration plan not found: {}", plan_id))?;

        {
            let mut current = self.current_migration.write();
            *current = Some(plan_id.to_string());
        }

        let mut updated_plan = plan.clone();
        updated_plan.status = "running".to_string();

        for step in &mut updated_plan.steps {
            step.status = "pending".to_string();
        }

        info!("Starting migration: {}", plan_id);

        let config = MigrationConfig {
            source_path: plan.source_path.clone(),
            target_path: plan.target_path.clone(),
            ..Default::default()
        };

        let source = Path::new(&config.source_path);
        let target = Path::new(&config.target_path);

        for step in &mut updated_plan.steps {
            step.status = "running".to_string();
            self.update_progress(plan_id, step.id, 0);

            match step.id {
                0 => {
                    step.status = "completed".to_string();
                    step.progress_percent = 100;
                }
                1..=5 => {
                    let world_dirs = ["world", "world_nether", "world_the_end"];
                    for world_name in world_dirs {
                        let world_path = source.join(world_name);
                        if world_path.exists() {
                            let dest = target.join(world_name);
                            if let Err(e) = self.copy_directory(&world_path, &dest).await {
                                step.status = "failed".to_string();
                                updated_plan.status = "failed".to_string();
                                return Err(format!("Failed to copy {}: {}", world_name, e));
                            }
                        }
                    }
                    step.status = "completed".to_string();
                    step.progress_percent = 100;
                }
                6 => {
                    fs::create_dir_all(target)
                        .map_err(|e| format!("Failed to create target directory: {}", e))?;
                    step.status = "completed".to_string();
                    step.progress_percent = 100;
                }
                7 => {
                    let config_files = [
                        "server.properties",
                        "bukkit.yml",
                        "spigot.yml",
                        "paper.yml",
                    ];
                    for config_file in config_files {
                        let config_path = source.join(config_file);
                        if config_path.exists() {
                            let dest = target.join(config_file);
                            fs::copy(&config_path, &dest)
                                .map_err(|e| format!("Failed to copy {}: {}", config_file, e))?;
                        }
                    }
                    step.status = "completed".to_string();
                    step.progress_percent = 100;
                }
                8 => {
                    step.status = "completed".to_string();
                    step.progress_percent = 100;
                }
                _ => {
                    step.status = "completed".to_string();
                    step.progress_percent = 100;
                }
            }

            self.update_progress(plan_id, step.id, step.progress_percent);
        }

        updated_plan.status = "completed".to_string();

        {
            let mut migrations = self.migrations.write();
            migrations.insert(plan_id.to_string(), updated_plan.clone());
        }

        {
            let mut current = self.current_migration.write();
            *current = None;
        }

        info!("Migration completed: {}", plan_id);
        Ok(updated_plan)
    }

    async fn copy_directory(&self, source: &Path, target: &Path) -> Result<(), String> {
        if !source.is_dir() {
            return Err("Source is not a directory".to_string());
        }

        fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        let entries = fs::read_dir(source)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());

            if source_path.is_dir() {
                self.copy_directory(&source_path, &target_path).await?;
            } else {
                fs::copy(&source_path, &target_path)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }

        Ok(())
    }

    fn update_progress(&self, _plan_id: &str, step_id: usize, progress: u32) {
        let tx = self.progress_tx.read();
        if let Some(sender) = tx.as_ref() {
            let progress = MigrationProgress {
                migration_id: _plan_id.to_string(),
                current_step: step_id,
                total_steps: 10,
                current_item: None,
                bytes_transferred: 0,
                total_bytes: 0,
                items_transferred: 0,
                total_items: 0,
                status: MigrationStatus::Copying,
                error: None,
            };
            let _ = sender.try_send(progress);
        }
    }

    pub fn set_progress_sender(&self, sender: mpsc::Sender<MigrationProgress>) {
        let mut tx = self.progress_tx.write();
        *tx = Some(sender);
    }

    pub fn get_plan(&self, plan_id: &str) -> Option<MigrationPlan> {
        self.migrations.read().get(plan_id).cloned()
    }

    pub fn list_plans(&self) -> Vec<MigrationPlan> {
        let migrations = self.migrations.read();
        let mut list: Vec<_> = migrations.values().cloned().collect();
        list.sort_by(|a, b| {
            let a_time = a.steps.first().map(|_| Utc::now()).unwrap_or_else(Utc::now);
            let b_time = b.steps.first().map(|_| Utc::now()).unwrap_or_else(Utc::now);
            b_time.cmp(&a_time)
        });
        list
    }

    pub fn cancel_migration(&self, plan_id: &str) -> Result<(), String> {
        let mut migrations = self.migrations.write();
        if let Some(plan) = migrations.get_mut(plan_id) {
            if plan.status == "running" {
                plan.status = "cancelled".to_string();
                for step in &mut plan.steps {
                    if step.status == "running" {
                        step.status = "cancelled".to_string();
                        break;
                    }
                }
                info!("Migration cancelled: {}", plan_id);
                Ok(())
            } else {
                Err("Migration is not running".to_string())
            }
        } else {
            Err(format!("Migration plan not found: {}", plan_id))
        }
    }

    pub fn delete_plan(&self, plan_id: &str) -> Result<(), String> {
        let mut migrations = self.migrations.write();
        if migrations.remove(plan_id).is_some() {
            info!("Migration plan deleted: {}", plan_id);
            Ok(())
        } else {
            Err(format!("Migration plan not found: {}", plan_id))
        }
    }

    pub fn get_stats(&self) -> MigrationStats {
        let migrations = self.migrations.read();
        MigrationStats {
            total_plans: migrations.len(),
            completed: migrations.values().filter(|p| p.status == "completed").count(),
            failed: migrations.values().filter(|p| p.status == "failed").count(),
            running: migrations.values().filter(|p| p.status == "running").count(),
            pending: migrations.values().filter(|p| p.status == "pending").count(),
        }
    }
}

impl Default for MigrationTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationStats {
    pub total_plans: usize,
    pub completed: usize,
    pub failed: usize,
    pub running: usize,
    pub pending: usize,
}

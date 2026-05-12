use crate::automation::{
    BackupConfig, BackupInfo, TaskResult, TaskStatus,
};
use chrono::{DateTime, Utc};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Debug, Clone)]
pub struct BackupManager {
    config: BackupConfig,
    last_backup: Option<DateTime<Utc>>,
    backups: RwLock<Vec<BackupInfo>>,
}

impl BackupManager {
    pub fn new(config: BackupConfig) -> Self {
        Self {
            config,
            last_backup: None,
            backups: RwLock::new(Vec::new()),
        }
    }

    pub fn update_config(&mut self, config: BackupConfig) {
        self.config = config;
    }

    pub async fn create_backup(&self, server_path: &Path) -> Result<BackupInfo, String> {
        if !self.config.enabled {
            return Err("Backup is disabled".to_string());
        }

        let backup_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        let backup_name = format!(
            "backup_{}_{}.zip",
            timestamp.format("%Y%m%d_%H%M%S"),
            &backup_id[..8]
        );
        let backup_path = PathBuf::from(&self.config.backup_path).join(&backup_name);

        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create backup dir: {}", e))?;
        }

        let mut zip_file = File::create(&backup_path)
            .map_err(|e| format!("Failed to create zip file: {}", e))?;
        let mut zip = ZipWriter::new(&mut zip_file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        let mut world_count = 0u32;
        let mut config_count = 0u32;

        if self.config.include_worlds {
            let worlds_path = server_path.join("world");
            if worlds_path.exists() {
                world_count = self.add_directory_to_zip(
                    &mut zip,
                    &worlds_path,
                    server_path,
                    "world",
                    &options,
                )?;
            }

            let world_nether = server_path.join("world_nether");
            if world_nether.exists() {
                world_count += self.add_directory_to_zip(
                    &mut zip,
                    &world_nether,
                    server_path,
                    "world_nether",
                    &options,
                )?;
            }

            let world_the_end = server_path.join("world_the_end");
            if world_the_end.exists() {
                world_count += self.add_directory_to_zip(
                    &mut zip,
                    &world_the_end,
                    server_path,
                    "world_the_end",
                    &options,
                )?;
            }
        }

        if self.config.include_configs {
            let config_files = ["server.properties", "bukkit.yml", "spigot.yml", "paper.yml"];
            for config_file in config_files {
                let config_path = server_path.join(config_file);
                if config_path.exists() {
                    if let Err(e) = self.add_file_to_zip(
                        &mut zip,
                        &config_path,
                        server_path,
                        config_file,
                        &options,
                    ) {
                        warn!("Failed to add config {}: {}", config_file, e);
                    } else {
                        config_count += 1;
                    }
                }
            }
        }

        let icons = server_path.join("server-icon.png");
        if icons.exists() {
            let _ = self.add_file_to_zip(
                &mut zip,
                &icons,
                server_path,
                "server-icon.png",
                &options,
            );
        }

        zip.finish().map_err(|e| format!("Failed to finish zip: {}", e))?;

        let metadata = fs::metadata(&backup_path)
            .map_err(|e| format!("Failed to get backup metadata: {}", e))?;

        let backup_info = BackupInfo {
            id: backup_id,
            name: backup_name,
            path: backup_path.to_string_lossy().to_string(),
            size_bytes: metadata.len(),
            created_at: timestamp,
            world_count,
            config_count,
        };

        let mut backups = self.backups.write().await;
        backups.push(backup_info.clone());
        drop(backups);

        self.clean_old_backups().await;

        info!(
            "Backup created: {} ({} bytes, {} worlds)",
            backup_info.name,
            backup_info.size_bytes,
            backup_info.world_count
        );

        Ok(backup_info)
    }

    fn add_directory_to_zip(
        &self,
        zip: &mut ZipWriter<&mut File>,
        dir_path: &Path,
        base_path: &Path,
        archive_name: &str,
        options: &SimpleFileOptions,
    ) -> Result<u32, String> {
        let mut count = 0u32;
        if !dir_path.is_dir() {
            return Ok(0);
        }

        fn walk_dir(
            zip: &mut ZipWriter<&mut File>,
            dir_path: &Path,
            base_path: &Path,
            archive_base: &str,
            options: &SimpleFileOptions,
            count: &mut u32,
        ) -> Result<(), String> {
            let entries = fs::read_dir(dir_path)
                .map_err(|e| format!("Failed to read dir: {}", e))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();
                let relative = path.strip_prefix(base_path)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                let name = relative.to_string_lossy();

                if path.is_file() {
                    let mut file = File::open(&path)
                        .map_err(|e| format!("Failed to open file: {}", e))?;
                    zip.start_file(name.to_string(), *options)
                        .map_err(|e| format!("Failed to start zip entry: {}", e))?;
                    io::copy(&mut file, zip)
                        .map_err(|e| format!("Failed to write file to zip: {}", e))?;
                    *count += 1;
                } else if path.is_dir() {
                    let dir_name = format!("{}/", name);
                    zip.add_directory(&dir_name, *options)
                        .map_err(|e| format!("Failed to add directory: {}", e))?;
                    walk_dir(zip, &path, base_path, archive_base, options, count)?;
                }
            }
            Ok(())
        }

        let mut dir_zip = ZipWriter::new(
            zip.into_inner().try_clone().map_err(|e| format!("Clone error: {}", e))?,
        );
        walk_dir(&mut dir_zip, dir_path, base_path, archive_name, options, &mut count)?;
        Ok(count)
    }

    fn add_file_to_zip(
        &self,
        zip: &mut ZipWriter<&mut File>,
        file_path: &Path,
        base_path: &Path,
        archive_name: &str,
        options: &SimpleFileOptions,
    ) -> Result<(), String> {
        let relative = file_path.strip_prefix(base_path)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        let mut file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        zip.start_file(relative.to_string_lossy().to_string(), *options)
            .map_err(|e| format!("Failed to start zip entry: {}", e))?;
        io::copy(&mut file, zip)
            .map_err(|e| format!("Failed to write file to zip: {}", e))?;
        Ok(())
    }

    pub async fn restore_backup(&self, backup_id: &str, target_path: &Path) -> Result<(), String> {
        let backups = self.backups.read().await;
        let backup = backups
            .iter()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| format!("Backup not found: {}", backup_id))?;
        drop(backups);

        let backup_file = File::open(&backup.path)
            .map_err(|e| format!("Failed to open backup file: {}", e))?;
        let mut archive = zip::ZipArchive::new(backup_file)
            .map_err(|e| format!("Failed to read zip archive: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {}", e))?;
            let outpath = target_path.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
                let mut outfile = File::create(&outpath)
                    .map_err(|e| format!("Failed to create output file: {}", e))?;
                io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to extract file: {}", e))?;
            }
        }

        info!("Backup restored from {} to {:?}", backup.name, target_path);
        Ok(())
    }

    pub async fn list_backups(&self) -> Vec<BackupInfo> {
        self.backups.read().await.clone()
    }

    pub async fn delete_backup(&self, backup_id: &str) -> Result<(), String> {
        let mut backups = self.backups.write().await;
        if let Some(pos) = backups.iter().position(|b| b.id == backup_id) {
            let backup = backups.remove(pos);
            fs::remove_file(&backup.path)
                .map_err(|e| format!("Failed to delete backup file: {}", e))?;
            info!("Deleted backup: {}", backup.name);
            Ok(())
        } else {
            Err(format!("Backup not found: {}", backup_id))
        }
    }

    async fn clean_old_backups(&self) {
        let mut backups = self.backups.write().await;
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        let to_delete: Vec<_> = backups
            .iter()
            .filter(|b| b.created_at < cutoff)
            .map(|b| b.id.clone())
            .collect();

        for id in to_delete {
            if let Some(pos) = backups.iter().position(|b| b.id == id) {
                let backup = backups.remove(pos);
                if let Err(e) = fs::remove_file(&backup.path) {
                    warn!("Failed to delete old backup {}: {}", backup.name, e);
                } else {
                    info!("Cleaned up old backup: {}", backup.name);
                }
            }
        }
    }

    pub fn get_status(&self) -> TaskStatus {
        TaskStatus {
            id: "backup".to_string(),
            name: "定时自动备份".to_string(),
            task_type: "backup".to_string(),
            enabled: self.config.enabled,
            last_run: self.last_backup,
            next_run: None,
            last_result: None,
            schedule: self.config.schedule.clone(),
        }
    }

    pub fn set_last_result(&mut self, result: TaskResult) {
        self.last_backup = Some(result.timestamp);
    }
}

pub async fn run_backup_task(
    manager: &BackupManager,
    server_path: &Path,
) -> TaskResult {
    let start = std::time::Instant::now();
    match manager.create_backup(server_path).await {
        Ok(info) => TaskResult {
            success: true,
            message: format!(
                "Backup created: {} ({} bytes)",
                info.name,
                info.size_bytes
            ),
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

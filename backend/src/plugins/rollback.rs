use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::plugins::types::*;

pub struct RollbackManager {
    backups_dir: PathBuf,
    plugins_dir: PathBuf,
}

impl RollbackManager {
    pub fn new(plugins_dir: PathBuf) -> Self {
        let backups_dir = plugins_dir.join("backups");
        
        Self {
            backups_dir,
            plugins_dir,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.backups_dir).await?;
        Ok(())
    }

    pub async fn create_backup(
        &self,
        plugin: &InstalledPlugin,
        reason: &str,
    ) -> Result<PluginBackup> {
        let source_path = self.plugins_dir.join(&plugin.file_name);
        
        if !source_path.exists() {
            anyhow::bail!("Plugin file not found: {:?}", source_path);
        }

        let backup_id = Uuid::new_v4().to_string();
        let backup_filename = format!(
            "{}-{}-{}.jar",
            plugin.name.replace(" ", "_"),
            plugin.version.replace(".", "_"),
            &backup_id[..8]
        );
        let backup_path = self.backups_dir.join(&backup_filename);

        fs::copy(&source_path, &backup_path).await?;

        let metadata = fs::metadata(&backup_path).await?;
        let checksum = self.calculate_checksum(&backup_path).await?;

        let backup = PluginBackup {
            id: backup_id,
            plugin_id: plugin.id.clone(),
            version: plugin.version.clone(),
            backup_date: Utc::now(),
            file_path: backup_path.to_string_lossy().to_string(),
            file_size: metadata.len() as i64,
            config_included: false,
            checksum,
            reason: reason.to_string(),
        };

        self.save_backup_metadata(&backup).await?;

        Ok(backup)
    }

    pub async fn rollback(
        &self,
        plugin_id: &str,
        backup_id: &str,
    ) -> Result<InstalledPlugin> {
        let backup = self.get_backup(backup_id).await?;

        if backup.plugin_id != plugin_id {
            anyhow::bail!("Backup does not match plugin ID");
        }

        let current_plugin_path = self.plugins_dir.join(&backup.version);
        let current_exists = current_plugin_path.exists().await;

        if current_exists {
            let emergency_backup_id = Uuid::new_v4().to_string();
            let emergency_filename = format!(
                "emergency-{}-{}.jar",
                plugin_id,
                &emergency_backup_id[..8]
            );
            let emergency_path = self.backups_dir.join(&emergency_filename);
            fs::copy(&current_plugin_path, &emergency_path).await?;
        }

        let backup_path = PathBuf::from(&backup.file_path);
        let plugin_files = self.find_plugin_files(&backup.plugin_id).await?;
        
        for file in &plugin_files {
            if file != &backup_path {
                fs::remove_file(file).await.ok();
            }
        }

        let restored_name = self.get_plugin_jar_name(&backup.plugin_id)?;
        let restored_path = self.plugins_dir.join(&restored_name);
        fs::copy(&backup_path, &restored_path).await?;

        let restored = InstalledPlugin {
            id: plugin_id.to_string(),
            plugin_id: backup.plugin_id.clone(),
            name: backup.plugin_id.clone(),
            version: backup.version.clone(),
            file_name: restored_name,
            install_date: backup.backup_date,
            last_updated: Utc::now(),
            enabled: true,
            config: PluginConfig {
                config_file: None,
                config_hash: None,
                custom_settings: serde_json::json!({}),
            },
            dependencies: Vec::new(),
            performance_stats: None,
            status: PluginStatus {
                is_loaded: false,
                errors: Vec::new(),
                warnings: Vec::new(),
                last_error: None,
            },
        };

        Ok(restored)
    }

    pub async fn get_backups(&self, plugin_id: &str) -> Result<Vec<PluginBackup>> {
        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&self.backups_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Ok(backup) = serde_json::from_str::<PluginBackup>(&content) {
                        if backup.plugin_id == plugin_id {
                            backups.push(backup);
                        }
                    }
                }
            }
        }

        backups.sort_by(|a, b| b.backup_date.cmp(&a.backup_date));
        Ok(backups)
    }

    pub async fn get_backup(&self, backup_id: &str) -> Result<PluginBackup> {
        let mut entries = fs::read_dir(&self.backups_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Ok(backup) = serde_json::from_str::<PluginBackup>(&content) {
                        if backup.id == backup_id {
                            return Ok(backup);
                        }
                    }
                }
            }
        }

        anyhow::bail!("Backup not found: {}", backup_id)
    }

    pub async fn delete_backup(&self, backup_id: &str) -> Result<()> {
        let backup = self.get_backup(backup_id).await?;
        let backup_path = PathBuf::from(&backup.file_path);

        if backup_path.exists() {
            fs::remove_file(&backup_path).await?;
        }

        let metadata_path = self.backups_dir.join(format!("{}.json", backup_id));
        if metadata_path.exists() {
            fs::remove_file(&metadata_path).await?;
        }

        Ok(())
    }

    pub async fn cleanup_old_backups(&self, keep_count: usize) -> Result<Vec<String>> {
        let mut all_backups = Vec::new();
        let mut entries = fs::read_dir(&self.backups_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Ok(backup) = serde_json::from_str::<PluginBackup>(&content) {
                        all_backups.push(backup);
                    }
                }
            }
        }

        all_backups.sort_by(|a, b| b.backup_date.cmp(&a.backup_date));

        let mut deleted = Vec::new();
        for (i, backup) in all_backups.iter().enumerate() {
            if i >= keep_count {
                self.delete_backup(&backup.id).await?;
                deleted.push(backup.id.clone());
            }
        }

        Ok(deleted)
    }

    async fn save_backup_metadata(&self, backup: &PluginBackup) -> Result<()> {
        let metadata_path = self.backups_dir.join(format!("{}.json", backup.id));
        let content = serde_json::to_string_pretty(backup)?;
        fs::write(&metadata_path, content).await?;
        Ok(())
    }

    async fn calculate_checksum(&self, path: &PathBuf) -> Result<String> {
        use sha2::{Sha256, Digest};
        let contents = fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    async fn find_plugin_files(&self, plugin_name: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(&self.plugins_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "jar" || ext == "yml" || ext == "yaml" {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.to_lowercase().contains(&plugin_name.to_lowercase()) {
                            files.push(path);
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    fn get_plugin_jar_name(&self, plugin_name: &str) -> Result<String> {
        Ok(format!("{}.jar", plugin_name))
    }

    pub async fn verify_backup_integrity(&self, backup_id: &str) -> Result<bool> {
        let backup = self.get_backup(backup_id).await?;
        let backup_path = PathBuf::from(&backup.file_path);

        if !backup_path.exists() {
            return Ok(false);
        }

        let calculated_checksum = self.calculate_checksum(&backup_path).await?;
        Ok(calculated_checksum == backup.checksum)
    }

    pub async fn export_backup(&self, backup_id: &str, target_path: &str) -> Result<()> {
        let backup = self.get_backup(backup_id).await?;
        let source_path = PathBuf::from(&backup.file_path);
        let target = PathBuf::from(target_path);

        if !source_path.exists() {
            anyhow::bail!("Backup file not found");
        }

        fs::copy(&source_path, &target).await?;
        Ok(())
    }
}

use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::error::AppError;
use crate::plugins::types::*;

pub struct PluginInstaller {
    plugins_dir: PathBuf,
    backups_dir: PathBuf,
    cache_dir: PathBuf,
}

impl PluginInstaller {
    pub fn new(plugins_dir: PathBuf) -> Self {
        let backups_dir = plugins_dir.join("backups");
        let cache_dir = plugins_dir.join("cache");
        
        Self {
            plugins_dir,
            backups_dir,
            cache_dir,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.backups_dir).await?;
        fs::create_dir_all(&self.cache_dir).await?;
        Ok(())
    }

    pub async fn install_plugin(
        &self,
        plugin: &Plugin,
        backup: bool,
    ) -> Result<InstalledPlugin> {
        if backup {
            self.ensure_backup_directory().await?;
        }

        let download_path = self.download_plugin(plugin).await?;

        if let Some(existing) = self.find_existing_plugin(&plugin.name).await? {
            if backup {
                self.create_backup(&existing, "Pre-install backup").await?;
            }
            self.remove_plugin(&existing).await?;
        }

        let target_path = self.plugins_dir.join(&plugin.file_name);
        fs::copy(&download_path, &target_path).await?;
        
        fs::remove_file(&download_path).await.ok();

        let installed = InstalledPlugin {
            id: Uuid::new_v4().to_string(),
            plugin_id: plugin.plugin_id.to_string(),
            name: plugin.name.clone(),
            version: plugin.version.clone(),
            file_name: plugin.file_name.clone(),
            install_date: Utc::now(),
            last_updated: Utc::now(),
            enabled: true,
            config: PluginConfig {
                config_file: None,
                config_hash: None,
                custom_settings: json!({}),
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

        Ok(installed)
    }

    pub async fn update_plugin(
        &self,
        plugin_id: &str,
        new_plugin: &Plugin,
        backup: bool,
    ) -> Result<InstalledPlugin> {
        let existing = self.find_installed_plugin(plugin_id)
            .await?
            .ok_or_else(|| AppError::Internal(format!("Plugin {} not found", plugin_id)))?;

        if backup {
            self.create_backup(&existing, "Pre-update backup").await?;
        }

        let download_path = self.download_plugin(new_plugin).await?;
        let target_path = self.plugins_dir.join(&new_plugin.file_name);
        fs::copy(&download_path, &target_path).await?;
        fs::remove_file(&download_path).await.ok();

        let updated = InstalledPlugin {
            id: existing.id,
            plugin_id: new_plugin.plugin_id.to_string(),
            name: existing.name.clone(),
            version: new_plugin.version.clone(),
            file_name: new_plugin.file_name.clone(),
            install_date: existing.install_date,
            last_updated: Utc::now(),
            enabled: existing.enabled,
            config: existing.config,
            dependencies: existing.dependencies,
            performance_stats: existing.performance_stats,
            status: PluginStatus {
                is_loaded: false,
                errors: Vec::new(),
                warnings: Vec::new(),
                last_error: None,
            },
        };

        Ok(updated)
    }

    pub async fn download_plugin(&self, plugin: &Plugin) -> Result<PathBuf> {
        let cache_path = self.cache_dir.join(format!("{}-{}.jar", plugin.name, plugin.version));
        
        if cache_path.exists() {
            return Ok(cache_path);
        }

        let response = reqwest::get(&plugin.download_url).await?;
        let bytes = response.bytes().await?;
        
        let mut file = fs::File::create(&cache_path).await?;
        file.write_all(&bytes).await?;

        Ok(cache_path)
    }

    pub async fn find_existing_plugin(&self, name: &str) -> Result<Option<InstalledPlugin>> {
        let mut entries = fs::read_dir(&self.plugins_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "jar").unwrap_or(false) {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                    
                if file_name.to_lowercase().contains(&name.to_lowercase()) {
                    let metadata = entry.metadata().await?;
                    let modified = metadata.modified()
                        .ok()
                        .map(|t| chrono::DateTime::<Utc>::from(t))
                        .unwrap_or_else(Utc::now);
                    
                    let version = self.extract_version_from_filename(file_name);
                    
                    return Ok(Some(InstalledPlugin {
                        id: Uuid::new_v4().to_string(),
                        plugin_id: String::new(),
                        name: name.to_string(),
                        version,
                        file_name: file_name.to_string(),
                        install_date: modified,
                        last_updated: modified,
                        enabled: true,
                        config: PluginConfig {
                            config_file: None,
                            config_hash: None,
                            custom_settings: json!({}),
                        },
                        dependencies: Vec::new(),
                        performance_stats: None,
                        status: PluginStatus {
                            is_loaded: false,
                            errors: Vec::new(),
                            warnings: Vec::new(),
                            last_error: None,
                        },
                    }));
                }
            }
        }
        
        Ok(None)
    }

    pub async fn find_installed_plugin(&self, plugin_id: &str) -> Result<Option<InstalledPlugin>> {
        let mut entries = fs::read_dir(&self.plugins_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map(|e| e == "jar").unwrap_or(false) {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                    
                let metadata = entry.metadata().await?;
                let modified = metadata.modified()
                    .ok()
                    .map(|t| chrono::DateTime::<Utc>::from(t))
                    .unwrap_or_else(Utc::now);
                
                let version = self.extract_version_from_filename(file_name);
                let name = self.extract_name_from_filename(file_name);
                
                return Ok(Some(InstalledPlugin {
                    id: plugin_id.to_string(),
                    plugin_id: plugin_id.to_string(),
                    name,
                    version,
                    file_name: file_name.to_string(),
                    install_date: modified,
                    last_updated: modified,
                    enabled: true,
                    config: PluginConfig {
                        config_file: None,
                        config_hash: None,
                        custom_settings: json!({}),
                    },
                    dependencies: Vec::new(),
                    performance_stats: None,
                    status: PluginStatus {
                        is_loaded: false,
                        errors: Vec::new(),
                        warnings: Vec::new(),
                        last_error: None,
                    },
                }));
            }
        }
        
        Ok(None)
    }

    fn extract_version_from_filename(&self, filename: &str) -> String {
        let version_patterns = [
            r"v?(\d+\.\d+\.\d+)",
            r"-(\d+\.\d+\.\d+)",
            r"_(\d+\.\d+\.\d+)",
        ];
        
        for pattern in version_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(filename) {
                    if let Some(v) = caps.get(1) {
                        return v.as_str().to_string();
                    }
                }
            }
        }
        
        "1.0.0".to_string()
    }

    fn extract_name_from_filename(&self, filename: &str) -> String {
        let name = filename
            .replace(".jar", "")
            .replace(".JAR", "");
        
        let version_patterns = [
            r"v?(\d+\.\d+\.\d+)",
            r"-(\d+\.\d+\.\d+)",
            r"_(\d+\.\d+\.\d+)",
        ];
        
        let mut result = name.clone();
        for pattern in version_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                result = re.replace(&result, "").to_string();
            }
        }
        
        result.trim_matches(&['-', '_', ' '][..]).to_string()
    }

    async fn create_backup(
        &self,
        plugin: &InstalledPlugin,
        reason: &str,
    ) -> Result<PluginBackup> {
        let source_path = self.plugins_dir.join(&plugin.file_name);
        let backup_id = Uuid::new_v4().to_string();
        let backup_filename = format!(
            "{}-{}-{}.jar",
            plugin.name,
            plugin.version,
            backup_id
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

        Ok(backup)
    }

    async fn calculate_checksum(&self, path: &PathBuf) -> Result<String> {
        use sha2::{Sha256, Digest};
        let contents = fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    async fn remove_plugin(&self, plugin: &InstalledPlugin) -> Result<()> {
        let plugin_path = self.plugins_dir.join(&plugin.file_name);
        if plugin_path.exists() {
            fs::remove_file(&plugin_path).await?;
        }
        Ok(())
    }

    async fn ensure_backup_directory(&self) -> Result<()> {
        fs::create_dir_all(&self.backups_dir).await
    }

    pub async fn get_plugin_versions(&self, plugin_id: i64) -> Result<Vec<PluginVersion>> {
        Ok(vec![
            PluginVersion {
                version: "1.0.0".to_string(),
                file_id: 1001,
                download_url: format!("https://cdn.spigotmc.org/resources/plugin-{}/versions/1.0.0.jar", plugin_id),
                release_date: Utc::now(),
                release_type: ReleaseType::Release,
                changelog: "Initial release".to_string(),
                supported_versions: vec!["1.20.4".to_string(), "1.20.2".to_string()],
            }
        ])
    }

    pub async fn uninstall_plugin(&self, plugin_id: &str) -> Result<()> {
        if let Some(plugin) = self.find_installed_plugin(plugin_id).await? {
            self.create_backup(&plugin, "Pre-uninstall backup").await?;
            self.remove_plugin(&plugin).await?;
        }
        Ok(())
    }
}

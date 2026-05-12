use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::plugins::types::*;

pub struct RepositoryManager {
    repositories_dir: PathBuf,
    repositories: RwLock<HashMap<String, CustomRepository>>,
    sync_status: RwLock<HashMap<String, SyncStatus>>,
}

#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub repo_id: String,
    pub status: SyncState,
    pub last_sync: Option<chrono::DateTime<Utc>>,
    pub plugins_count: i32,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SyncState {
    Idle,
    Syncing,
    Success,
    Failed,
}

impl RepositoryManager {
    pub fn new(repositories_dir: PathBuf) -> Self {
        Self {
            repositories_dir,
            repositories: RwLock::new(HashMap::new()),
            sync_status: RwLock::new(HashMap::new()),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.repositories_dir).await?;
        self.load_repositories().await?;
        self.setup_default_repositories().await?;
        Ok(())
    }

    async fn load_repositories(&mut self) -> Result<()> {
        let path = self.repositories_dir.join("repositories.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(path).await?;
            let repos: Vec<CustomRepository> = serde_json::from_str(&content)?;
            let mut r = self.repositories.write().await;
            for repo in repos {
                r.insert(repo.id.clone(), repo);
            }
        }
        Ok(())
    }

    async fn save_repositories(&self) -> Result<()> {
        let repos = self.repositories.read().await;
        let values: Vec<_> = repos.values().cloned().collect();
        let content = serde_json::to_string_pretty(&values)?;
        let path = self.repositories_dir.join("repositories.json");
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    async fn setup_default_repositories(&self) -> Result<()> {
        let repos = self.repositories.read().await;
        if repos.is_empty() {
            drop(repos);
            
            let spigot = CustomRepository {
                id: Uuid::new_v4().to_string(),
                name: "SpigotMC".to_string(),
                url: "https://hub.spigotmc.org".to_string(),
                repo_type: RepositoryType::SpigotMC,
                enabled: true,
                priority: 1,
                last_sync: Some(Utc::now()),
                plugins_count: 50000,
                auth_required: false,
                auth_token: None,
                metadata: json!({
                    "api_endpoint": "/resources/items",
                    "download_endpoint": "/builds/downloads"
                }),
            };

            let bukkit = CustomRepository {
                id: Uuid::new_v4().to_string(),
                name: "Bukkit".to_string(),
                url: "https://dev.bukkit.org".to_string(),
                repo_type: RepositoryType::Bukkit,
                enabled: true,
                priority: 2,
                last_sync: Some(Utc::now()),
                plugins_count: 30000,
                auth_required: false,
                auth_token: None,
                metadata: json!({}),
            };

            let paper = CustomRepository {
                id: Uuid::new_v4().to_string(),
                name: "PaperMC".to_string(),
                url: "https://papermc.io".to_string(),
                repo_type: RepositoryType::Paper,
                enabled: true,
                priority: 3,
                last_sync: Some(Utc::now()),
                plugins_count: 10000,
                auth_required: false,
                auth_token: None,
                metadata: json!({}),
            };

            let mut r = self.repositories.write().await;
            r.insert(spigot.id.clone(), spigot);
            r.insert(bukkit.id.clone(), bukkit);
            r.insert(paper.id.clone(), paper);
            
            drop(r);
            self.save_repositories().await?;
        }
        Ok(())
    }

    pub async fn add_repository(
        &self,
        name: String,
        url: String,
        repo_type: RepositoryType,
        auth_token: Option<String>,
    ) -> Result<CustomRepository> {
        let repo = CustomRepository {
            id: Uuid::new_v4().to_string(),
            name,
            url: url.clone(),
            repo_type,
            enabled: true,
            priority: 10,
            last_sync: None,
            plugins_count: 0,
            auth_required: auth_token.is_some(),
            auth_token,
            metadata: json!({
                "custom_url": url
            }),
        };

        let mut repos = self.repositories.write().await;
        repos.insert(repo.id.clone(), repo.clone());
        drop(repos);
        
        self.save_repositories().await?;
        Ok(repo)
    }

    pub async fn get_repositories(&self) -> Vec<CustomRepository> {
        let repos = self.repositories.read().await;
        let mut result: Vec<_> = repos.values().cloned().collect();
        result.sort_by(|a, b| a.priority.cmp(&b.priority));
        result
    }

    pub async fn get_repository(&self, repo_id: &str) -> Option<CustomRepository> {
        let repos = self.repositories.read().await;
        repos.get(repo_id).cloned()
    }

    pub async fn update_repository(
        &self,
        repo_id: &str,
        updates: HashMap<String, serde_json::Value>,
    ) -> Result<CustomRepository> {
        let mut repos = self.repositories.write().await;
        
        if let Some(repo) = repos.get_mut(repo_id) {
            if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
                repo.name = name.to_string();
            }
            if let Some(enabled) = updates.get("enabled").and_then(|v| v.as_bool()) {
                repo.enabled = enabled;
            }
            if let Some(priority) = updates.get("priority").and_then(|v| v.as_i64()) {
                repo.priority = priority as i32;
            }
            if let Some(token) = updates.get("auth_token").and_then(|v| v.as_str()) {
                repo.auth_token = Some(token.to_string());
                repo.auth_required = true;
            }
            
            let updated = repo.clone();
            drop(repos);
            self.save_repositories().await?;
            Ok(updated)
        } else {
            anyhow::bail!("Repository not found")
        }
    }

    pub async fn delete_repository(&self, repo_id: &str) -> Result<()> {
        let mut repos = self.repositories.write().await;
        repos.remove(repo_id);
        drop(repos);
        self.save_repositories().await
    }

    pub async fn sync_repository(&self, repo_id: &str) -> Result<SyncStatus> {
        {
            let mut status = self.sync_status.write().await;
            status.insert(repo_id.to_string(), SyncStatus {
                repo_id: repo_id.to_string(),
                status: SyncState::Syncing,
                last_sync: None,
                plugins_count: 0,
                error: None,
            });
        }

        let repos = self.repositories.read().await;
        let repo = repos.get(repo_id).cloned();
        
        let result = match repo {
            Some(r) => self.perform_sync(&r).await,
            None => Err(anyhow::anyhow!("Repository not found")),
        };

        let mut status = self.sync_status.write().await;
        
        match result {
            Ok(plugins_count) => {
                status.insert(repo_id.to_string(), SyncStatus {
                    repo_id: repo_id.to_string(),
                    status: SyncState::Success,
                    last_sync: Some(Utc::now()),
                    plugins_count,
                    error: None,
                });
            }
            Err(e) => {
                status.insert(repo_id.to_string(), SyncStatus {
                    repo_id: repo_id.to_string(),
                    status: SyncState::Failed,
                    last_sync: None,
                    plugins_count: 0,
                    error: Some(e.to_string()),
                });
            }
        }

        Ok(status.get(repo_id).unwrap().clone())
    }

    async fn perform_sync(&self, repo: &CustomRepository) -> Result<i32> {
        match repo.repo_type {
            RepositoryType::SpigotMC => {
                Ok(50000)
            }
            RepositoryType::Bukkit => {
                Ok(30000)
            }
            RepositoryType::Paper => {
                Ok(10000)
            }
            RepositoryType::Jenkins => {
                Ok(5000)
            }
            RepositoryType::GitHub => {
                Ok(1000)
            }
            RepositoryType::Custom => {
                Ok(100)
            }
        }
    }

    pub async fn get_sync_status(&self, repo_id: &str) -> Option<SyncStatus> {
        let status = self.sync_status.read().await;
        status.get(repo_id).cloned()
    }

    pub async fn search_in_repository(
        &self,
        repo_id: &str,
        query: &str,
    ) -> Result<Vec<Plugin>> {
        let repos = self.repositories.read().await;
        let repo = repos.get(repo_id).cloned();
        
        match repo {
            Some(_) => {
                Ok(vec![
                    Plugin {
                        id: Uuid::new_v4().to_string(),
                        name: format!("Example Plugin for {}", query),
                        plugin_id: 1,
                        version: "1.0.0".to_string(),
                        file_id: 1,
                        download_url: format!("https://example.com/plugin.jar"),
                        file_name: "example.jar".to_string(),
                        file_size: 1024000,
                        upload_date: Utc::now(),
                        description: Some("Example plugin".to_string()),
                        authors: vec!["Author".to_string()],
                        category: "General".to_string(),
                        tags: vec!["example".to_string()],
                        source_url: None,
                        issue_tracker: None,
                        website: None,
                        supported_versions: vec!["1.20.4".to_string()],
                        stats: PluginStats {
                            downloads: 1000,
                            likes: 100,
                            reviews_count: 50,
                            average_rating: 4.5,
                        },
                        compatibility: PluginCompatibility {
                            compatibility_score: 85.0,
                            server_version: "1.20.4".to_string(),
                            api_version: "1.20".to_string(),
                            known_issues: vec![],
                            verified: false,
                        },
                        is_premium: false,
                        price: None,
                    }
                ])
            }
            None => anyhow::bail!("Repository not found"),
        }
    }

    pub async fn test_connection(&self, repo: &CustomRepository) -> Result<bool> {
        match reqwest::get(&repo.url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

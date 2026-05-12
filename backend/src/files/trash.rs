use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use uuid::Uuid;

pub struct TrashService {
    server_root: PathBuf,
    trash_dir: PathBuf,
    metadata_file: PathBuf,
    items: RwLock<HashMap<String, TrashItem>>,
    default_retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashItem {
    pub id: String,
    pub original_path: String,
    pub original_name: String,
    pub trash_path: String,
    pub deleted_at: String,
    pub deleted_by: String,
    pub size: u64,
    pub file_type: String,
    pub expires_at: String,
    pub metadata: TrashMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashMetadata {
    pub is_directory: bool,
    pub checksum: Option<String>,
    pub permissions: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashList {
    pub items: Vec<TrashItem>,
    pub total_size: u64,
    pub item_count: usize,
    pub oldest_item: Option<String>,
    pub newest_item: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub success: bool,
    pub restored_path: String,
    pub item_id: String,
    pub warnings: Vec<String>,
}

impl TrashService {
    pub fn new(server_root: PathBuf) -> Self {
        let trash_dir = server_root.join(".trash");
        let metadata_file = trash_dir.join(".trash_metadata.json");
        
        fs::create_dir_all(&trash_dir).ok();

        let service = Self {
            server_root,
            trash_dir,
            metadata_file,
            items: RwLock::new(HashMap::new()),
            default_retention_days: 30,
        };
        
        service.load_metadata().ok();
        service
    }

    fn load_metadata(&self) -> Result<()> {
        if self.metadata_file.exists() {
            let content = fs::read_to_string(&self.metadata_file)?;
            let items: HashMap<String, TrashItem> = serde_json::from_str(&content)?;
            let mut guard = self.items.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            *guard = items;
        }
        Ok(())
    }

    fn save_metadata(&self) -> Result<()> {
        let guard = self.items.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let content = serde_json::to_string_pretty(&*guard)?;
        fs::write(&self.metadata_file, content)?;
        Ok(())
    }

    pub fn delete(&self, path: &str, user_id: Option<&str>) -> Result<TrashItem> {
        let full_path = self.server_root.join(path);
        
        if !full_path.exists() {
            anyhow::bail!("File not found: {}", path);
        }

        let original_path = path.to_string();
        let original_name = full_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let metadata = fs::metadata(&full_path)?;
        let is_directory = metadata.is_dir();
        
        let file_type = if is_directory {
            "directory".to_string()
        } else {
            full_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("file")
                .to_string()
        };

        let item_id = Uuid::new_v4().to_string();
        let trash_path = self.get_trash_path(&item_id, &original_name);
        let trash_full_path = self.trash_dir.join(&trash_path);

        if let Some(parent) = trash_full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::rename(&full_path, &trash_full_path)?;

        let checksum = if !is_directory {
            Some(self.calculate_checksum(&trash_full_path)?)
        } else {
            None
        };

        let now = chrono::Utc::now();
        let deleted_at = now.to_rfc3339();
        let expires_at = (now + chrono::Duration::days(self.default_retention_days as i64))
            .to_rfc3339();

        let item = TrashItem {
            id: item_id.clone(),
            original_path: original_path.clone(),
            original_name: original_name.clone(),
            trash_path: trash_path.clone(),
            deleted_at,
            deleted_by: user_id.unwrap_or("system").to_string(),
            size: metadata.len(),
            file_type,
            expires_at: expires_at.clone(),
            metadata: TrashMetadata {
                is_directory,
                checksum,
                permissions: None,
                content_type: None,
            },
        };

        {
            let mut guard = self.items.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.insert(item_id, item.clone());
        }

        self.save_metadata()?;

        Ok(item)
    }

    pub fn restore(&self, item_id: &str) -> Result<RestoreResult> {
        let item = {
            let guard = self.items.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.get(item_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Trash item not found: {}", item_id))?
        };

        let trash_full_path = self.trash_dir.join(&item.trash_path);
        
        if !trash_full_path.exists() {
            anyhow::bail!("Trash file not found, may have been permanently deleted");
        }

        let mut restored_path = item.original_path.clone();
        let mut warnings = Vec::new();

        let dest_path = self.server_root.join(&restored_path);
        if dest_path.exists() {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let extension = PathBuf::from(&item.original_name)
                .extension()
                .and_then(|e| e.to_str());
            
            let new_name = if let Some(ext) = extension {
                format!("{}.{}.restored", 
                    item.original_name.strip_suffix(&format!(".{}", ext)).unwrap_or(&item.original_name),
                    timestamp)
            } else {
                format!("{}.{}.restored", item.original_name, timestamp)
            };

            restored_path = PathBuf::from(&item.original_path)
                .parent()
                .map(|p| p.join(&new_name).to_string_lossy().to_string())
                .unwrap_or_else(|| new_name.clone());

            warnings.push(format!("Destination exists, restored as '{}'", new_name));
        }

        let final_dest = self.server_root.join(&restored_path);
        
        if let Some(parent) = final_dest.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::rename(&trash_full_path, &final_dest)?;

        {
            let mut guard = self.items.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.remove(item_id);
        }

        self.save_metadata()?;

        Ok(RestoreResult {
            success: true,
            restored_path,
            item_id: item_id.to_string(),
            warnings,
        })
    }

    pub fn permanent_delete(&self, item_id: &str) -> Result<()> {
        let item = {
            let guard = self.items.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.get(item_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Trash item not found: {}", item_id))?
        };

        let trash_full_path = self.trash_dir.join(&item.trash_path);
        
        if trash_full_path.exists() {
            if item.metadata.is_directory {
                fs::remove_dir_all(&trash_full_path)?;
            } else {
                fs::remove_file(&trash_full_path)?;
            }
        }

        {
            let mut guard = self.items.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.remove(item_id);
        }

        self.save_metadata()?;

        Ok(())
    }

    pub fn empty_trash(&self) -> Result<TrashList> {
        let items = self.list_items()?;

        for item in &items.items {
            let trash_full_path = self.trash_dir.join(&item.trash_path);
            
            if trash_full_path.exists() {
                if item.metadata.is_directory {
                    fs::remove_dir_all(&trash_full_path)?;
                } else {
                    fs::remove_file(&trash_full_path)?;
                }
            }
        }

        {
            let mut guard = self.items.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.clear();
        }

        self.save_metadata()?;

        Ok(items)
    }

    pub fn list_items(&self) -> Result<TrashList> {
        let guard = self.items.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let mut items: Vec<TrashItem> = guard.values().cloned().collect();
        items.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));

        let total_size: u64 = items.iter().map(|i| i.size).sum();
        let item_count = items.len();

        let oldest_item = items.last().map(|i| i.deleted_at.clone());
        let newest_item = items.first().map(|i| i.deleted_at.clone());

        Ok(TrashList {
            items,
            total_size,
            item_count,
            oldest_item,
            newest_item,
        })
    }

    pub fn get_item(&self, item_id: &str) -> Result<TrashItem> {
        let guard = self.items.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        guard.get(item_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Trash item not found: {}", item_id))
    }

    pub fn search_items(&self, query: &str) -> Result<Vec<TrashItem>> {
        let guard = self.items.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let query_lower = query.to_lowercase();
        let results: Vec<TrashItem> = guard.values()
            .filter(|item| {
                item.original_name.to_lowercase().contains(&query_lower) ||
                item.original_path.to_lowercase().contains(&query_lower) ||
                item.file_type.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        Ok(results)
    }

    pub fn cleanup_expired(&self) -> Result<CleanupResult> {
        let now = chrono::Utc::now();
        
        let expired_ids: Vec<String> = {
            let guard = self.items.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.values()
                .filter(|item| {
                    chrono::DateTime::parse_from_rfc3339(&item.expires_at)
                        .map(|dt| dt.with_timezone(&chrono::Utc) < now)
                        .unwrap_or(false)
                })
                .map(|item| item.id.clone())
                .collect()
        };

        let mut deleted_count = 0;
        let mut freed_bytes = 0u64;

        for id in expired_ids {
            if let Ok(item) = self.get_item(&id) {
                freed_bytes += item.size;
                if self.permanent_delete(&id).is_ok() {
                    deleted_count += 1;
                }
            }
        }

        Ok(CleanupResult {
            deleted_count,
            freed_bytes,
            deleted_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn restore_batch(&self, item_ids: &[String]) -> Result<Vec<RestoreResult>> {
        let mut results = Vec::new();

        for id in item_ids {
            match self.restore(id) {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(RestoreResult {
                        success: false,
                        restored_path: String::new(),
                        item_id: id.clone(),
                        warnings: vec![e.to_string()],
                    });
                }
            }
        }

        Ok(results)
    }

    pub fn permanent_delete_batch(&self, item_ids: &[String]) -> Result<usize> {
        let mut count = 0;

        for id in item_ids {
            if self.permanent_delete(id).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn get_statistics(&self) -> Result<TrashStatistics> {
        let list = self.list_items()?;
        let mut by_type: HashMap<String, TypeStats> = HashMap::new();

        for item in &list.items {
            let entry = by_type.entry(item.file_type.clone())
                .or_insert_with(|| TypeStats {
                    file_type: item.file_type.clone(),
                    count: 0,
                    total_size: 0,
                });
            entry.count += 1;
            entry.total_size += item.size;
        }

        let storage_info = self.get_storage_info()?;

        Ok(TrashStatistics {
            total_items: list.item_count,
            total_size: list.total_size,
            by_type,
            storage_info,
        })
    }

    fn get_storage_info(&self) -> Result<StorageInfo> {
        let trash_size: u64 = WalkDir::new(&self.trash_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum();

        let available_space = fs2::available_space(&self.trash_dir)
            .unwrap_or(0);

        Ok(StorageInfo {
            used_bytes: trash_size,
            available_bytes: available_space,
            item_count: {
                let guard = self.items.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
                guard.len() as u64
            },
        })
    }

    fn get_trash_path(&self, id: &str, original_name: &str) -> String {
        let date = chrono::Utc::now().format("%Y/%m/%d");
        format!("{}/{}/{}", date, id, original_name)
    }

    fn calculate_checksum(&self, path: &PathBuf) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize()))
    }

    pub fn set_retention_period(&mut self, days: u32) {
        self.default_retention_days = days;
    }

    pub fn get_retention_period(&self) -> u32 {
        self.default_retention_days
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub deleted_count: usize,
    pub freed_bytes: u64,
    pub deleted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashStatistics {
    pub total_items: usize,
    pub total_size: u64,
    pub by_type: HashMap<String, TypeStats>,
    pub storage_info: StorageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStats {
    pub file_type: String,
    pub count: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub item_count: u64,
}

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct EditorService {
    server_root: PathBuf,
    max_file_size: usize,
}

impl EditorService {
    pub fn new(server_root: PathBuf) -> Self {
        Self {
            server_root,
            max_file_size: 10 * 1024 * 1024,
        }
    }

    pub fn read_file(&self, path: &str) -> Result<(String, String)> {
        let full_path = self.resolve_path(path)?;
        
        if !full_path.exists() {
            anyhow::bail!("File not found: {}", path);
        }

        let metadata = fs::metadata(&full_path)?;
        if metadata.len() as usize > self.max_file_size {
            anyhow::bail!("File too large: {} bytes (max: {} bytes)", 
                metadata.len(), self.max_file_size);
        }

        let content = fs::read_to_string(&full_path)?;
        let extension = full_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt")
            .to_string();

        Ok((content, extension))
    }

    pub fn save_file(&self, path: &str, content: &str) -> Result<FileSaveResult> {
        let full_path = self.resolve_path(path)?;
        
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let original_content = if full_path.exists() {
            Some(fs::read_to_string(&full_path)?)
        } else {
            None
        };

        let checksum_before = original_content.as_ref().map(|c| {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(c.as_bytes());
            hex::encode(hasher.finalize())
        });

        fs::write(&full_path, content)?;

        let checksum_after = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            hex::encode(hasher.finalize())
        };

        let backup_path = self.create_backup(&full_path)?;

        Ok(FileSaveResult {
            path: path.to_string(),
            checksum_before,
            checksum_after,
            backup_path: Some(backup_path),
            bytes_written: content.len() as u64,
        })
    }

    pub fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;
        
        if !full_path.exists() {
            anyhow::bail!("File not found: {}", path);
        }

        if full_path.is_dir() {
            fs::remove_dir_all(&full_path)?;
        } else {
            fs::remove_file(&full_path)?;
        }

        Ok(())
    }

    pub fn read_dir(&self, path: &str) -> Result<Vec<FileListEntry>> {
        let full_path = if path.is_empty() || path == "/" {
            self.server_root.clone()
        } else {
            self.resolve_path(path)?
        };

        if !full_path.exists() {
            anyhow::bail!("Directory not found: {}", path);
        }

        let mut entries = Vec::new();

        for entry in fs::read_dir(&full_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            let modified: String = metadata.modified()
                .ok()
                .map(|t| DateTime::<Utc>::from(t).to_rfc3339())
                .unwrap_or_default();

            let permissions = if metadata.permissions().readonly() {
                "r--"
            } else {
                "rw-"
            }.to_string();

            entries.push(FileListEntry {
                name,
                path: entry.path().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified,
                permissions,
            });
        }

        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        Ok(entries)
    }

    pub fn create_file(&self, path: &str, content: &str) -> Result<FileInfo> {
        let full_path = self.resolve_path(path)?;
        
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&full_path, content)?;

        self.get_file_info(path)
    }

    pub fn create_dir(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path)?;
        fs::create_dir_all(&full_path)?;
        Ok(())
    }

    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<String> {
        let old_full = self.resolve_path(old_path)?;
        let new_full = self.resolve_path(new_path)?;

        fs::rename(&old_full, &new_full)?;

        Ok(new_path.to_string())
    }

    pub fn copy(&self, source: &str, destination: &str) -> Result<String> {
        let source_full = self.resolve_path(source)?;
        let dest_full = self.resolve_path(destination)?;

        if source_full.is_dir() {
            copy_dir_recursive(&source_full, &dest_full)?;
        } else {
            fs::copy(&source_full, &dest_full)?;
        }

        Ok(destination.to_string())
    }

    pub fn move_file(&self, source: &str, destination: &str) -> Result<String> {
        self.copy(source, destination)?;
        self.delete(source)?;

        Ok(destination.to_string())
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        if path.is_empty() || path == "/" {
            return Ok(self.server_root.clone());
        }
        
        let requested_path = self.server_root.join(path);
        let canonical = requested_path.canonicalize()?;
        
        if !canonical.starts_with(&self.server_root) {
            anyhow::bail!("Path escape detected: {}", path);
        }

        Ok(canonical)
    }

    fn create_backup(&self, path: &Path) -> Result<PathBuf> {
        let backup_dir = self.server_root.join(".backups");
        fs::create_dir_all(&backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let backup_path = backup_dir.join(format!("{}.{}", filename, timestamp));
        fs::copy(path, &backup_path)?;

        Ok(backup_path)
    }

    pub fn list_backups(&self, filename: &str) -> Result<Vec<BackupInfo>> {
        let backup_dir = self.server_root.join(".backups");
        
        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();
        for entry in fs::read_dir(backup_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            
            if name_str.starts_with(filename) && name_str.contains(".") {
                let metadata = entry.metadata()?;
                let modified = metadata.modified()?;
                
                backups.push(BackupInfo {
                    path: entry.path(),
                    created: DateTime::<Utc>::from(modified).to_rfc3339(),
                    size: metadata.len(),
                });
            }
        }

        backups.sort_by(|a, b| b.created.cmp(&a.created));
        Ok(backups)
    }

    pub fn restore_backup(&self, backup_path: &str) -> Result<String> {
        let backup = PathBuf::from(backup_path);
        
        if !backup.exists() {
            anyhow::bail!("Backup file not found");
        }

        let filename = backup.file_name()
            .and_then(|n| n.to_str())
            .unwrap();
        
        let parts: Vec<&str> = filename.rsplitn(2, '.').collect();
        if parts.len() < 2 {
            anyhow::bail!("Invalid backup filename format");
        }

        let original_name = parts[1];
        let content = fs::read_to_string(&backup)?;
        self.save_file(original_name, &content)?;

        Ok(original_name.to_string())
    }

    pub fn get_file_info(&self, path: &str) -> Result<FileInfo> {
        let full_path = self.resolve_path(path)?;
        
        let metadata = fs::metadata(&full_path)?;
        let content = if metadata.is_file() {
            let size = metadata.len() as usize;
            if size > self.max_file_size {
                None
            } else {
                fs::read_to_string(&full_path).ok()
            }
        } else {
            None
        };

        Ok(FileInfo {
            path: path.to_string(),
            name: full_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            is_directory: metadata.is_dir(),
            size: metadata.len(),
            created: metadata.created()
                .ok()
                .map(|t| DateTime::<Utc>::from(t).to_rfc3339()),
            modified: metadata.modified()
                .ok()
                .map(|t| DateTime::<Utc>::from(t).to_rfc3339()),
            extension: full_path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_string()),
            line_count: content.as_ref().map(|c| c.lines().count()),
            content_hash: content.as_ref().map(|c| {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(c.as_bytes());
                hex::encode(hasher.finalize())
            }),
        })
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSaveResult {
    pub path: String,
    pub checksum_before: Option<String>,
    pub checksum_after: String,
    pub backup_path: Option<PathBuf>,
    pub bytes_written: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub created: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub extension: Option<String>,
    pub line_count: Option<usize>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub permissions: String,
}

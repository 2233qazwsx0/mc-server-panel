use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::RwLock;
use uuid::Uuid;
use sha2::{Sha256, Digest};

pub struct UploadService {
    server_root: PathBuf,
    temp_dir: PathBuf,
    chunk_dir: PathBuf,
    uploads: RwLock<HashMap<String, ChunkUpload>>,
    max_chunk_size: usize,
    max_file_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkUpload {
    pub id: String,
    pub filename: String,
    pub destination: String,
    pub total_chunks: usize,
    pub received_chunks: Vec<usize>,
    pub total_size: u64,
    pub checksum: String,
    pub created_at: String,
    pub last_activity: String,
    pub metadata: UploadMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMetadata {
    pub mime_type: Option<String>,
    pub original_size: u64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub index: usize,
    pub size: usize,
    pub checksum: String,
    pub received: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    pub upload_id: String,
    pub filename: String,
    pub total_chunks: usize,
    pub received_chunks: usize,
    pub received_bytes: u64,
    pub total_bytes: u64,
    pub progress_percent: f32,
    pub is_complete: bool,
    pub chunks: Vec<ChunkInfo>,
}

impl UploadService {
    pub fn new(server_root: PathBuf) -> Self {
        let temp_dir = server_root.join(".temp");
        let chunk_dir = server_root.join(".temp").join("chunks");
        
        fs::create_dir_all(&chunk_dir).ok();

        Self {
            server_root,
            temp_dir,
            chunk_dir,
            uploads: RwLock::new(HashMap::new()),
            max_chunk_size: 5 * 1024 * 1024,
            max_file_size: 1024 * 1024 * 1024,
        }
    }

    pub fn initiate_upload(&self, request: InitUploadRequest) -> Result<ChunkUpload> {
        if request.total_size > self.max_file_size as u64 {
            anyhow::bail!("File size exceeds maximum allowed: {} bytes", self.max_file_size);
        }

        let upload_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let upload = ChunkUpload {
            id: upload_id.clone(),
            filename: request.filename.clone(),
            destination: request.path.clone(),
            total_chunks: request.total_chunks,
            received_chunks: Vec::new(),
            total_size: request.total_size,
            checksum: request.checksum.clone(),
            created_at: now.clone(),
            last_activity: now,
            metadata: request.metadata.unwrap_or_default(),
        };

        let chunk_info_dir = self.chunk_dir.join(&upload_id);
        fs::create_dir_all(&chunk_info_dir)?;

        {
            let mut guard = self.uploads.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.insert(upload_id.clone(), upload.clone());
        }

        Ok(upload)
    }

    pub fn upload_chunk(&self, upload_id: &str, chunk_index: usize, data: Vec<u8>, expected_checksum: &str) -> Result<ChunkUploadResponse> {
        let mut guard = self.uploads.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let upload = guard.get_mut(upload_id)
            .ok_or_else(|| anyhow::anyhow!("Upload not found: {}", upload_id))?;

        if chunk_index >= upload.total_chunks {
            anyhow::bail!("Invalid chunk index: {} (total: {})", chunk_index, upload.total_chunks);
        }

        let chunk_size = data.len();
        let received_checksum = self.calculate_checksum(&data);

        if received_checksum != expected_checksum {
            anyhow::bail!("Checksum mismatch for chunk {}: expected {}, got {}", 
                chunk_index, expected_checksum, received_checksum);
        }

        let chunk_path = self.chunk_dir.join(upload_id).join(format!("chunk_{}", chunk_index));
        let mut file = File::create(&chunk_path)?;
        file.write_all(&data)?;
        drop(file);

        upload.received_chunks.push(chunk_index);
        upload.last_activity = chrono::Utc::now().to_rfc3339();

        let received_bytes: u64 = upload.received_chunks.iter()
            .map(|&idx| {
                let path = self.chunk_dir.join(upload_id).join(format!("chunk_{}", idx));
                fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            })
            .sum();

        let is_complete = upload.received_chunks.len() == upload.total_chunks;

        if is_complete {
            let final_checksum = self.verify_upload(upload_id)?;
            if final_checksum != upload.checksum {
                anyhow::bail!("Final checksum mismatch: expected {}, got {}", upload.checksum, final_checksum);
            }
        }

        Ok(ChunkUploadResponse {
            upload_id: upload_id.to_string(),
            chunk_index,
            received: true,
            total_received: upload.received_chunks.len(),
            is_complete,
            received_bytes,
            total_bytes: upload.total_size,
        })
    }

    pub fn complete_upload(&self, upload_id: &str) -> Result<CompletedUpload> {
        let upload = {
            let guard = self.uploads.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.get(upload_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Upload not found: {}", upload_id))?
        };

        if upload.received_chunks.len() != upload.total_chunks {
            anyhow::bail!("Upload incomplete: {}/{} chunks received", 
                upload.received_chunks.len(), upload.total_chunks);
        }

        let dest_path = self.server_root.join(&upload.destination);
        
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut output_file = File::create(&dest_path)?;

        for i in 0..upload.total_chunks {
            let chunk_path = self.chunk_dir.join(upload_id).join(format!("chunk_{}", i));
            
            if !chunk_path.exists() {
                anyhow::bail!("Missing chunk {} for upload {}", i, upload_id);
            }

            let mut chunk_file = File::open(&chunk_path)?;
            let mut buffer = Vec::new();
            chunk_file.read_to_end(&mut buffer)?;
            output_file.write_all(&buffer)?;
        }

        output_file.flush()?;
        drop(output_file);

        self.cleanup_upload(upload_id)?;

        let metadata = fs::metadata(&dest_path)?;

        Ok(CompletedUpload {
            upload_id: upload_id.to_string(),
            filename: upload.filename,
            path: upload.destination,
            size: metadata.len(),
            completed_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub fn get_upload_progress(&self, upload_id: &str) -> Result<UploadProgress> {
        let upload = {
            let guard = self.uploads.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.get(upload_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Upload not found: {}", upload_id))?
        };

        let received_bytes: u64 = upload.received_chunks.iter()
            .map(|&idx| {
                let path = self.chunk_dir.join(upload_id).join(format!("chunk_{}", idx));
                fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            })
            .sum();

        let progress_percent = (received_bytes as f32 / upload.total_size as f32) * 100.0;

        let chunks: Vec<ChunkInfo> = (0..upload.total_chunks)
            .map(|i| {
                let path = self.chunk_dir.join(upload_id).join(format!("chunk_{}", i));
                ChunkInfo {
                    index: i,
                    size: fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(0),
                    checksum: self.read_chunk_checksum(&path).unwrap_or_default(),
                    received: upload.received_chunks.contains(&i),
                }
            })
            .collect();

        Ok(UploadProgress {
            upload_id: upload.id,
            filename: upload.filename,
            total_chunks: upload.total_chunks,
            received_chunks: upload.received_chunks.len(),
            received_bytes,
            total_bytes: upload.total_size,
            progress_percent,
            is_complete: upload.received_chunks.len() == upload.total_chunks,
            chunks,
        })
    }

    pub fn cancel_upload(&self, upload_id: &str) -> Result<()> {
        self.cleanup_upload(upload_id)?;
        
        let mut guard = self.uploads.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        guard.remove(upload_id);
        
        Ok(())
    }

    pub fn list_uploads(&self) -> Result<Vec<UploadProgress>> {
        let guard = self.uploads.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let uploads: Vec<UploadProgress> = guard.values()
            .map(|upload| {
                let received_bytes: u64 = upload.received_chunks.iter()
                    .map(|&idx| {
                        let path = self.chunk_dir.join(&upload.id).join(format!("chunk_{}", idx));
                        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                    })
                    .sum();

                UploadProgress {
                    upload_id: upload.id.clone(),
                    filename: upload.filename.clone(),
                    total_chunks: upload.total_chunks,
                    received_chunks: upload.received_chunks.len(),
                    received_bytes,
                    total_bytes: upload.total_size,
                    progress_percent: (received_bytes as f32 / upload.total_size as f32) * 100.0,
                    is_complete: false,
                    chunks: vec![],
                }
            })
            .collect();

        Ok(uploads)
    }

    fn cleanup_upload(&self, upload_id: &str) -> Result<()> {
        let chunk_dir = self.chunk_dir.join(upload_id);
        if chunk_dir.exists() {
            fs::remove_dir_all(&chunk_dir)?;
        }
        Ok(())
    }

    fn verify_upload(&self, upload_id: &str) -> Result<String> {
        let upload = {
            let guard = self.uploads.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.get(upload_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Upload not found: {}", upload_id))?
        };

        let mut hasher = Sha256::new();

        for i in 0..upload.total_chunks {
            let chunk_path = self.chunk_dir.join(upload_id).join(format!("chunk_{}", i));
            
            if !chunk_path.exists() {
                anyhow::bail!("Missing chunk {} for verification", i);
            }

            let mut chunk_file = File::open(&chunk_path)?;
            let mut buffer = Vec::new();
            chunk_file.read_to_end(&mut buffer)?;
            hasher.update(&buffer);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    fn read_chunk_checksum(&self, path: &PathBuf) -> Result<String> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(self.calculate_checksum(&buffer))
    }

    pub fn cleanup_stale_uploads(&self, max_age_hours: u64) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours as i64);
        
        let stale_ids: Vec<String> = {
            let guard = self.uploads.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.values()
                .filter(|upload| {
                    chrono::DateTime::parse_from_rfc3339(&upload.last_activity)
                        .map(|dt| dt.with_timezone(&chrono::Utc) < cutoff)
                        .unwrap_or(false)
                })
                .map(|upload| upload.id.clone())
                .collect()
        };

        let mut count = 0;
        for id in stale_ids {
            if self.cancel_upload(&id).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitUploadRequest {
    pub filename: String,
    pub path: String,
    pub total_chunks: usize,
    pub total_size: u64,
    pub checksum: String,
    pub metadata: Option<UploadMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkUploadResponse {
    pub upload_id: String,
    pub chunk_index: usize,
    pub received: bool,
    pub total_received: usize,
    pub is_complete: bool,
    pub received_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedUpload {
    pub upload_id: String,
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub completed_at: String,
}

impl Default for UploadMetadata {
    fn default() -> Self {
        Self {
            mime_type: None,
            original_size: 0,
            description: None,
        }
    }
}

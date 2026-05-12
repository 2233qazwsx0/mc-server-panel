pub mod editor;
pub mod diff;
pub mod archive;
pub mod acl;
pub mod git_integration;
pub mod sync;
pub mod search;
pub mod validator;
pub mod upload;
pub mod trash;
pub mod handlers;
pub mod routes;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub path: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub case_sensitive: bool,
    pub max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file: FileEntry,
    pub line_matches: Vec<LineMatch>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineMatch {
    pub line_number: usize,
    pub content: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub original_file: String,
    pub modified_file: String,
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub original_start: usize,
    pub original_count: usize,
    pub modified_start: usize,
    pub modified_count: usize,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub change_type: String,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRequest {
    pub files: Vec<String>,
    pub output_name: String,
    pub format: ArchiveFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclEntry {
    pub id: String,
    pub path: String,
    pub principal: AclPrincipal,
    pub permissions: Vec<AclPermission>,
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclPrincipal {
    pub kind: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AclPermission {
    Read,
    Write,
    Delete,
    Execute,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkUploadRequest {
    pub upload_id: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub filename: String,
    pub path: String,
    pub content: Vec<u8>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkUploadResponse {
    pub upload_id: String,
    pub chunk_index: usize,
    pub received: bool,
    pub total_received: usize,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    pub id: String,
    pub file_path: String,
    pub version: String,
    pub commit_hash: String,
    pub author: String,
    pub message: String,
    pub timestamp: String,
    pub size: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashItem {
    pub id: String,
    pub original_path: String,
    pub deleted_at: String,
    pub size: u64,
    pub expires_at: String,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub file_path: String,
    pub file_type: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub id: String,
    pub name: String,
    pub source: SyncEndpoint,
    pub target: SyncEndpoint,
    pub direction: SyncDirection,
    pub auto_sync: bool,
    pub sync_interval: u64,
    pub last_sync: Option<String>,
    pub status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEndpoint {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub credentials: Option<SyncCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCredentials {
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error,
    Completed,
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::files::{
    archive::{ArchiveEntry, ArchiveFormat, ArchiveRequest, ArchiveResult, ExtractResult},
    editor::{BackupInfo, EditorService, FileInfo, FileSaveResult},
    diff::{DiffLine, DiffResult, InlineDiffResult},
    acl::{AclRule, AclRuleRequest, AclRuleUpdate},
    search::{IndexStats, SearchQuery, SearchResult, SearchResultFile},
    sync::{
        ConflictResolution, SyncConfig, SyncConfigRequest, SyncConfigUpdate, SyncEndpoint,
        SyncEvent, SyncPreview, SyncProtocol, SyncStatus,
    },
    upload::{
        ChunkUpload, ChunkUploadResponse, CompletedUpload, InitUploadRequest, UploadMetadata,
        UploadProgress,
    },
    trash::{CleanupResult, RestoreResult, TrashItem, TrashList, TrashStatistics},
    validator::{SupportedFileType, ValidationResult},
    git_integration::{GitCommit, GitDiff, GitStatus},
    AppState,
};

pub fn create_files_routes(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/api/files/list", post(list_files))
        .route("/api/files/read", get(read_file))
        .route("/api/files/write", post(write_file))
        .route("/api/files/delete", delete(delete_file))
        .route("/api/files/create", post(create_file))
        .route("/api/files/create-dir", post(create_directory))
        .route("/api/files/rename", put(rename_file))
        .route("/api/files/copy", post(copy_file))
        .route("/api/files/move", put(move_file))
        .route("/api/files/info", get(get_file_info))
        .route("/api/files/download", get(download_file))
        .route("/api/files/upload", post(upload_file))
        .route("/api/files/backups", get(list_backups))
        .route("/api/files/restore-backup", post(restore_backup))
        .route("/api/files/diff", get(compare_files))
        .route("/api/files/diff-inline", get(compare_files_inline))
        .route("/api/files/diff-backup", get(compare_with_backup))
        .route("/api/files/archive/create", post(create_archive))
        .route("/api/files/archive/extract", post(extract_archive))
        .route("/api/files/archive/list", get(list_archive_contents))
        .route("/api/files/acl/rules", get(list_acl_rules))
        .route("/api/files/acl/rules", post(add_acl_rule))
        .route("/api/files/acl/rules/:id", put(update_acl_rule))
        .route("/api/files/acl/rules/:id", delete(remove_acl_rule))
        .route("/api/files/acl/check", get(check_acl_permission))
        .route("/api/files/acl/effective", get(get_effective_permissions))
        .route("/api/files/search", post(search_files))
        .route("/api/files/search/name", get(search_by_name))
        .route("/api/files/search/rebuild-index", post(rebuild_search_index))
        .route("/api/files/search/stats", get(get_search_stats))
        .route("/api/files/validate", post(validate_file))
        .route("/api/files/validate/types", get(get_supported_types))
        .route("/api/files/upload/init", post(init_upload))
        .route("/api/files/upload/chunk", post(upload_chunk))
        .route("/api/files/upload/complete/:id", post(complete_upload))
        .route("/api/files/upload/progress/:id", get(get_upload_progress))
        .route("/api/files/upload/cancel/:id", post(cancel_upload))
        .route("/api/files/upload/list", get(list_uploads))
        .route("/api/files/trash/list", get(list_trash))
        .route("/api/files/trash/delete/:id", post(delete_to_trash))
        .route("/api/files/trash/restore/:id", post(restore_from_trash))
        .route("/api/files/trash/purge/:id", delete(purge_trash_item))
        .route("/api/files/trash/empty", post(empty_trash))
        .route("/api/files/trash/cleanup", post(cleanup_expired))
        .route("/api/files/trash/stats", get(trash_statistics))
        .route("/api/files/sync/configs", get(list_sync_configs))
        .route("/api/files/sync/configs", post(create_sync_config))
        .route("/api/files/sync/configs/:id", put(update_sync_config))
        .route("/api/files/sync/configs/:id", delete(delete_sync_config))
        .route("/api/files/sync/preview/:id", get(preview_sync))
        .route("/api/files/sync/execute/:id", post(execute_sync))
        .route("/api/files/sync/history/:id", get(sync_history))
        .route("/api/files/git/init", post(init_git_repo))
        .route("/api/files/git/status", get(git_status))
        .route("/api/files/git/commit", post(git_commit))
        .route("/api/files/git/log", get(git_log))
        .route("/api/files/git/diff", get(git_diff))
        .route("/api/files/git/branches", get(git_branches))
        .route("/api/files/git/blame", get(git_blame))
        .with_state(state)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListFilesRequest {
    pub path: String,
    pub include_hidden: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub permissions: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

async fn list_files(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<ListFilesRequest>,
) -> Json<ApiResponse<Vec<FileListEntry>>> {
    let state = state.read().await;
    let editor = state.file_editor.as_ref();
    
    match editor {
        Some(editor) => {
            match editor.read_dir(&req.path) {
                Ok(entries) => Json(ApiResponse::success(entries)),
                Err(e) => Json(ApiResponse::error(&e.to_string())),
            }
        }
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

async fn read_file(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<(String, String)>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.read_file(path) {
            Ok(content) => Json(ApiResponse::success(content)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

async fn write_file(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<WriteFileRequest>,
) -> Json<ApiResponse<FileSaveResult>> {
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.save_file(&params.path, &params.content) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub content: String,
}

async fn delete_file(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<bool>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.delete(path) {
            Ok(_) => Json(ApiResponse::success(true)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

async fn create_file(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<CreateFileRequest>,
) -> Json<ApiResponse<FileInfo>> {
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.create_file(&params.path, &params.content) {
            Ok(info) => Json(ApiResponse::success(info)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub path: String,
    pub content: String,
}

async fn create_directory(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<CreateDirRequest>,
) -> Json<ApiResponse<bool>> {
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.create_dir(&params.path) {
            Ok(_) => Json(ApiResponse::success(true)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDirRequest {
    pub path: String,
}

async fn rename_file(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<RenameRequest>,
) -> Json<ApiResponse<String>> {
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.rename(&params.old_path, &params.new_path) {
            Ok(new_path) => Json(ApiResponse::success(new_path)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub old_path: String,
    pub new_path: String,
}

async fn copy_file(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<CopyRequest>,
) -> Json<ApiResponse<String>> {
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.copy(&params.source, &params.destination) {
            Ok(dest) => Json(ApiResponse::success(dest)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct CopyRequest {
    pub source: String,
    pub destination: String,
}

async fn move_file(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<MoveRequest>,
) -> Json<ApiResponse<String>> {
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.move_file(&params.source, &params.destination) {
            Ok(dest) => Json(ApiResponse::success(dest)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct MoveRequest {
    pub source: String,
    pub destination: String,
}

async fn get_file_info(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<FileInfo>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.get_file_info(path) {
            Ok(info) => Json(ApiResponse::success(info)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

async fn download_file() -> Json<ApiResponse<String>> {
    Json(ApiResponse::error("Not implemented"))
}

async fn upload_file() -> Json<ApiResponse<String>> {
    Json(ApiResponse::error("Not implemented"))
}

async fn list_backups(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<BackupInfo>>> {
    let filename = params.get("filename").map(|s| s.as_str()).unwrap_or("");
    let state = state.read().await;
    
    match &state.file_editor {
        Some(editor) => match editor.list_backups(filename) {
            Ok(backups) => Json(ApiResponse::success(backups)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("File service not initialized")),
    }
}

async fn restore_backup(
    Json(params): Json<RestoreBackupRequest>,
) -> Json<ApiResponse<String>> {
    Json(ApiResponse::error("Not implemented"))
}

#[derive(Debug, Deserialize)]
pub struct RestoreBackupRequest {
    pub backup_path: String,
}

async fn compare_files(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<DiffResult>> {
    let path1 = params.get("path1").map(|s| s.as_str()).unwrap_or("");
    let path2 = params.get("path2").map(|s| s.as_str()).unwrap_or("");
    
    let state = state.read().await;
    
    match &state.diff_service {
        Some(diff) => match diff.compare_files(path1, path2) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Diff service not initialized")),
    }
}

async fn compare_files_inline(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<InlineDiffResult>> {
    let path1 = params.get("path1").map(|s| s.as_str()).unwrap_or("");
    let path2 = params.get("path2").map(|s| s.as_str()).unwrap_or("");
    
    let state = state.read().await;
    
    match &state.diff_service {
        Some(diff) => match diff.get_inline_diff(path1, path2) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Diff service not initialized")),
    }
}

async fn compare_with_backup(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<DiffResult>> {
    let file_path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    
    let state = state.read().await;
    
    match &state.diff_service {
        Some(diff) => match diff.compare_with_backup(file_path) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Diff service not initialized")),
    }
}

async fn create_archive(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<ArchiveRequest>,
) -> Json<ApiResponse<ArchiveResult>> {
    let state = state.read().await;
    
    match &state.archive_service {
        Some(archive) => match archive.create_archive(&request) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Archive service not initialized")),
    }
}

async fn extract_archive(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<ExtractArchiveRequest>,
) -> Json<ApiResponse<ExtractResult>> {
    let state = state.read().await;
    
    match &state.archive_service {
        Some(archive) => match archive.extract_archive(&params.archive_path, &params.destination) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Archive service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct ExtractArchiveRequest {
    pub archive_path: String,
    pub destination: String,
}

async fn list_archive_contents(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<ArchiveEntry>>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    
    let state = state.read().await;
    
    match &state.archive_service {
        Some(archive) => match archive.list_archive_contents(path) {
            Ok(entries) => Json(ApiResponse::success(entries)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Archive service not initialized")),
    }
}

async fn list_acl_rules(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<AclRule>>> {
    let path_filter = params.get("path").map(|s| s.as_str());
    
    let state = state.read().await;
    
    match &state.acl_service {
        Some(acl) => match acl.list_rules(path_filter) {
            Ok(rules) => Json(ApiResponse::success(rules)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("ACL service not initialized")),
    }
}

async fn add_acl_rule(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(rule): Json<AclRuleRequest>,
) -> Json<ApiResponse<AclRule>> {
    let state = state.read().await;
    
    match &state.acl_service {
        Some(acl) => match acl.add_rule(rule) {
            Ok(new_rule) => Json(ApiResponse::success(new_rule)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("ACL service not initialized")),
    }
}

async fn update_acl_rule(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
    Json(update): Json<AclRuleUpdate>,
) -> Json<ApiResponse<AclRule>> {
    let state = state.read().await;
    
    match &state.acl_service {
        Some(acl) => match acl.update_rule(&id, update) {
            Ok(rule) => Json(ApiResponse::success(rule)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("ACL service not initialized")),
    }
}

async fn remove_acl_rule(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<bool>> {
    let state = state.read().await;
    
    match &state.acl_service {
        Some(acl) => match acl.remove_rule(&id) {
            Ok(_) => Json(ApiResponse::success(true)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("ACL service not initialized")),
    }
}

async fn check_acl_permission(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<bool>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    let principal = params.get("principal").map(|s| s.as_str()).unwrap_or("");
    let permission = params.get("permission").map(|s| s.as_str()).unwrap_or("");
    
    let state = state.read().await;
    
    match &state.acl_service {
        Some(acl) => match acl.check_permission(path, principal, permission) {
            Ok(allowed) => Json(ApiResponse::success(allowed)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("ACL service not initialized")),
    }
}

async fn get_effective_permissions(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<String>>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    let principal = params.get("principal").map(|s| s.as_str()).unwrap_or("");
    
    let state = state.read().await;
    
    match &state.acl_service {
        Some(acl) => match acl.get_effective_permissions(path, principal) {
            Ok(perms) => Json(ApiResponse::success(perms)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("ACL service not initialized")),
    }
}

async fn search_files(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(query): Json<SearchQuery>,
) -> Json<ApiResponse<Vec<SearchResult>>> {
    let state = state.read().await;
    
    match &state.search_service {
        Some(search) => match search.search(query) {
            Ok(results) => Json(ApiResponse::success(results)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Search service not initialized")),
    }
}

async fn search_by_name(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<SearchResultFile>>> {
    let name = params.get("name").map(|s| s.as_str()).unwrap_or("");
    let path = params.get("path").map(|s| s.as_str());
    
    let state = state.read().await;
    
    match &state.search_service {
        Some(search) => match search.search_by_name(name, path) {
            Ok(results) => Json(ApiResponse::success(results)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Search service not initialized")),
    }
}

async fn rebuild_search_index(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<crate::files::search::FileIndex>> {
    let state = state.read().await;
    
    match &state.search_service {
        Some(search) => match search.rebuild_index() {
            Ok(index) => Json(ApiResponse::success(index)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Search service not initialized")),
    }
}

async fn get_search_stats(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<IndexStats>> {
    let state = state.read().await;
    
    match &state.search_service {
        Some(search) => match search.get_index_stats() {
            Ok(stats) => Json(ApiResponse::success(stats)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Search service not initialized")),
    }
}

async fn validate_file(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<ValidateFileRequest>,
) -> Json<ApiResponse<ValidationResult>> {
    let state = state.read().await;
    
    match &state.validator_service {
        Some(validator) => match validator.validate_file(&params.path) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Validator service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct ValidateFileRequest {
    pub path: String,
}

async fn get_supported_types(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<Vec<SupportedFileType>>> {
    let state = state.read().await;
    
    match &state.validator_service {
        Some(validator) => {
            let types = validator.get_supported_types();
            Json(ApiResponse::success(types))
        }
        None => Json(ApiResponse::error("Validator service not initialized")),
    }
}

async fn init_upload(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(request): Json<InitUploadRequest>,
) -> Json<ApiResponse<ChunkUpload>> {
    let state = state.read().await;
    
    match &state.upload_service {
        Some(upload) => match upload.initiate_upload(request) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Upload service not initialized")),
    }
}

async fn upload_chunk(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<UploadChunkRequest>,
) -> Json<ApiResponse<ChunkUploadResponse>> {
    let state = state.read().await;
    
    match &state.upload_service {
        Some(upload) => match upload.upload_chunk(&params.upload_id, params.chunk_index, params.data, &params.checksum) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Upload service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct UploadChunkRequest {
    pub upload_id: String,
    pub chunk_index: usize,
    pub data: Vec<u8>,
    pub checksum: String,
}

async fn complete_upload(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<CompletedUpload>> {
    let state = state.read().await;
    
    match &state.upload_service {
        Some(upload) => match upload.complete_upload(&id) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Upload service not initialized")),
    }
}

async fn get_upload_progress(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<UploadProgress>> {
    let state = state.read().await;
    
    match &state.upload_service {
        Some(upload) => match upload.get_upload_progress(&id) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Upload service not initialized")),
    }
}

async fn cancel_upload(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<bool>> {
    let state = state.read().await;
    
    match &state.upload_service {
        Some(upload) => match upload.cancel_upload(&id) {
            Ok(_) => Json(ApiResponse::success(true)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Upload service not initialized")),
    }
}

async fn list_uploads(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<Vec<UploadProgress>>> {
    let state = state.read().await;
    
    match &state.upload_service {
        Some(upload) => match upload.list_uploads() {
            Ok(results) => Json(ApiResponse::success(results)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Upload service not initialized")),
    }
}

async fn list_trash(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<TrashList>> {
    let state = state.read().await;
    
    match &state.trash_service {
        Some(trash) => match trash.list_items() {
            Ok(items) => Json(ApiResponse::success(items)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Trash service not initialized")),
    }
}

async fn delete_to_trash(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<DeleteToTrashRequest>,
) -> Json<ApiResponse<TrashItem>> {
    let state = state.read().await;
    
    match &state.trash_service {
        Some(trash) => match trash.delete(&params.path, params.user_id.as_deref()) {
            Ok(item) => Json(ApiResponse::success(item)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Trash service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteToTrashRequest {
    pub path: String,
    pub user_id: Option<String>,
}

async fn restore_from_trash(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<RestoreResult>> {
    let state = state.read().await;
    
    match &state.trash_service {
        Some(trash) => match trash.restore(&id) {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Trash service not initialized")),
    }
}

async fn purge_trash_item(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<bool>> {
    let state = state.read().await;
    
    match &state.trash_service {
        Some(trash) => match trash.permanent_delete(&id) {
            Ok(_) => Json(ApiResponse::success(true)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Trash service not initialized")),
    }
}

async fn empty_trash(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<TrashList>> {
    let state = state.read().await;
    
    match &state.trash_service {
        Some(trash) => match trash.empty_trash() {
            Ok(list) => Json(ApiResponse::success(list)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Trash service not initialized")),
    }
}

async fn cleanup_expired(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<CleanupResult>> {
    let state = state.read().await;
    
    match &state.trash_service {
        Some(trash) => match trash.cleanup_expired() {
            Ok(result) => Json(ApiResponse::success(result)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Trash service not initialized")),
    }
}

async fn trash_statistics(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<TrashStatistics>> {
    let state = state.read().await;
    
    match &state.trash_service {
        Some(trash) => match trash.get_statistics() {
            Ok(stats) => Json(ApiResponse::success(stats)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Trash service not initialized")),
    }
}

async fn list_sync_configs(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<Vec<SyncConfig>>> {
    let state = state.read().await;
    
    match &state.sync_service {
        Some(sync) => match sync.list_configs() {
            Ok(configs) => Json(ApiResponse::success(configs)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Sync service not initialized")),
    }
}

async fn create_sync_config(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(config): Json<SyncConfigRequest>,
) -> Json<ApiResponse<SyncConfig>> {
    let state = state.read().await;
    
    match &state.sync_service {
        Some(sync) => match sync.create_config(config) {
            Ok(new_config) => Json(ApiResponse::success(new_config)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Sync service not initialized")),
    }
}

async fn update_sync_config(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
    Json(update): Json<SyncConfigUpdate>,
) -> Json<ApiResponse<SyncConfig>> {
    let state = state.read().await;
    
    match &state.sync_service {
        Some(sync) => match sync.update_config(&id, update) {
            Ok(config) => Json(ApiResponse::success(config)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Sync service not initialized")),
    }
}

async fn delete_sync_config(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<bool>> {
    let state = state.read().await;
    
    match &state.sync_service {
        Some(sync) => match sync.delete_config(&id) {
            Ok(_) => Json(ApiResponse::success(true)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Sync service not initialized")),
    }
}

async fn preview_sync(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<SyncPreview>> {
    let state = state.read().await;
    
    match &state.sync_service {
        Some(sync) => match sync.preview_sync(&id) {
            Ok(preview) => Json(ApiResponse::success(preview)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Sync service not initialized")),
    }
}

async fn execute_sync(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<SyncEvent>> {
    let state = state.read().await;
    
    match &state.sync_service {
        Some(sync) => match sync.execute_sync(&id) {
            Ok(event) => Json(ApiResponse::success(event)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Sync service not initialized")),
    }
}

async fn sync_history(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<Vec<SyncEvent>>> {
    let state = state.read().await;
    
    match &state.sync_service {
        Some(sync) => match sync.get_sync_history(&id) {
            Ok(history) => Json(ApiResponse::success(history)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Sync service not initialized")),
    }
}

async fn init_git_repo(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<InitGitRequest>,
) -> Json<ApiResponse<GitRepository>> {
    let state = state.read().await;
    
    match &state.git_service {
        Some(git) => match git.init_repo(&params.path, &params.name) {
            Ok(repo) => Json(ApiResponse::success(repo)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Git service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct InitGitRequest {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepository {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote_url: Option<String>,
    pub branch: String,
    pub initialized: bool,
}

async fn git_status(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<GitStatus>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or(".");
    
    let state = state.read().await;
    
    match &state.git_service {
        Some(git) => match git.get_status(path) {
            Ok(status) => Json(ApiResponse::success(status)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Git service not initialized")),
    }
}

async fn git_commit(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(params): Json<GitCommitRequest>,
) -> Json<ApiResponse<GitCommit>> {
    let state = state.read().await;
    
    match &state.git_service {
        Some(git) => match git.commit(&params.path, &params.message) {
            Ok(commit) => Json(ApiResponse::success(commit)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Git service not initialized")),
    }
}

#[derive(Debug, Deserialize)]
pub struct GitCommitRequest {
    pub path: String,
    pub message: String,
}

async fn git_log(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<GitCommit>>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or(".");
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50) as usize;
    
    let state = state.read().await;
    
    match &state.git_service {
        Some(git) => match git.get_log(path, limit) {
            Ok(commits) => Json(ApiResponse::success(commits)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Git service not initialized")),
    }
}

async fn git_diff(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<GitDiff>>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or(".");
    let target = params.get("target").map(|s| s.as_str());
    
    let state = state.read().await;
    
    match &state.git_service {
        Some(git) => match git.diff(path, target) {
            Ok(diffs) => Json(ApiResponse::success(diffs)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Git service not initialized")),
    }
}

async fn git_branches(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<Vec<String>>> {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or(".");
    
    let state = state.read().await;
    
    match &state.git_service {
        Some(git) => match git.get_branches(path) {
            Ok(branches) => Json(ApiResponse::success(branches)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Git service not initialized")),
    }
}

async fn git_blame(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<ApiResponse<GitBlame>> {
    let file_path = params.get("file").map(|s| s.as_str()).unwrap_or("");
    
    let state = state.read().await;
    
    match &state.git_service {
        Some(git) => match git.blame(file_path) {
            Ok(blame) => Json(ApiResponse::success(blame)),
            Err(e) => Json(ApiResponse::error(&e.to_string())),
        },
        None => Json(ApiResponse::error("Git service not initialized")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBlame {
    pub file: String,
    pub lines: Vec<GitBlameLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBlameLine {
    pub line_number: usize,
    pub commit: String,
    pub author: String,
    pub date: String,
    pub content: String,
}

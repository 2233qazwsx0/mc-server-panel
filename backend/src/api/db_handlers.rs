use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::error::AppError;
use crate::state::AppState;
use crate::database::{
    connection::DatabaseManager,
    models::{DatabaseConfig, DatabaseType},
    player_stats::{PlayerStatsQuery, PlayerStatsUpdate, TotalStats},
    economy::{EconomyStats, TransferRequest, TransactionRequest},
    api_keys::{ApiKeyListItem, ApiKeyUpdate, CreateApiKeyRequest},
    export_import::{ExportRequest, ExportResult, ImportRequest, ImportResult},
    optimization::{DatabaseSize, OptimizationResult, OptimizationSchedule},
    performance::{DatabaseStats, QueryStats, QueryStatsSummary},
    archive::{ArchiveRequest, ArchiveResult, ArchivePolicy},
    sync::{SyncRequest, SyncResult, SyncStatusRecord},
    backup::{BackupResult, BackupType, RestoreResult},
};
use crate::database::{
    player_stats::PlayerStatsRepository,
    economy::EconomyRepository,
    api_keys::ApiKeyRepository,
    export_import::ExportImportService,
    optimization::OptimizationService,
    performance::PerformanceAnalyzer,
    archive::ArchiveService,
    sync::SyncService,
    backup::BackupService,
};

pub fn create_db_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/db/status", get(get_db_status))
        .route("/api/db/switch", post(switch_database))
        .route("/api/db/health", get(db_health_check))
        .route("/api/db/player-stats", get(get_all_players).post(create_player))
        .route("/api/db/player-stats/:uuid", get(get_player).put(update_player).delete(delete_player))
        .route("/api/db/player-stats/top/:limit", get(get_top_players))
        .route("/api/db/player-stats/total", get(get_total_stats))
        .route("/api/db/economy", get(get_all_balances))
        .route("/api/db/economy/richest/:limit", get(get_richest_players))
        .route("/api/db/economy/stats", get(get_economy_stats))
        .route("/api/db/economy/account/:uuid", get(get_player_balance))
        .route("/api/db/economy/deposit", post(deposit))
        .route("/api/db/economy/withdraw", post(withdraw))
        .route("/api/db/economy/transfer", post(transfer))
        .route("/api/db/economy/transactions/:uuid", get(get_transaction_history))
        .route("/api/db/api-keys", get(list_api_keys).post(create_api_key))
        .route("/api/db/api-keys/:id", delete(delete_api_key).put(update_api_key))
        .route("/api/db/api-keys/:id/revoke", post(revoke_api_key))
        .route("/api/db/export", post(export_data))
        .route("/api/db/import", post(import_data))
        .route("/api/db/export-all", post(export_all_json))
        .route("/api/db/import-all", post(import_all_json))
        .route("/api/db/optimize", post(run_optimization))
        .route("/api/db/optimize/vacuum", post(run_vacuum))
        .route("/api/db/optimize/analyze", post(run_analyze))
        .route("/api/db/optimize/schedule", get(get_optimization_schedule).put(update_optimization_schedule))
        .route("/api/db/optimize/size", get(get_database_size))
        .route("/api/db/performance/slow-queries", get(get_slow_queries))
        .route("/api/db/performance/stats", get(get_all_query_stats))
        .route("/api/db/performance/:query_type", get(get_query_stats_by_type))
        .route("/api/db/performance/cleanup", post(cleanup_old_metrics))
        .route("/api/db/archive", post(run_archive))
        .route("/api/db/archive/restore", post(restore_from_archive))
        .route("/api/db/archive/list", get(list_archives))
        .route("/api/db/archive/:id", delete(delete_archive_record))
        .route("/api/db/archive/cleanup", post(cleanup_expired_archives))
        .route("/api/db/sync", post(start_sync))
        .route("/api/db/sync/status", get(get_sync_status))
        .route("/api/db/sync/:sync_id", delete(cancel_sync))
        .route("/api/db/backup", get(list_backups).post(create_backup))
        .route("/api/db/backup/:id", delete(delete_backup_record))
        .route("/api/db/backup/:id/restore", post(restore_backup_record))
        .route("/api/db/backup/:id/verify", get(verify_backup_record))
        .route("/api/db/backup/cleanup", post(cleanup_old_backups))
        .with_state(state)
}

#[derive(Debug, Serialize)]
pub struct DbStatus {
    pub db_type: String,
    pub url: String,
    pub connected: bool,
}

async fn get_db_status(State(state): State<AppState>) -> Result<Json<DbStatus>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    
    Ok(Json(DbStatus {
        db_type: format!("{:?}", db.db_type()),
        url: db.url().to_string(),
        connected: db.health_check().unwrap_or(false),
    }))
}

async fn switch_database(
    State(state): State<AppState>,
    Json(config): Json<DatabaseConfig>,
) -> Result<Json<DbStatus>, AppError> {
    let mut db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    db.switch_database(config.clone()).await?;
    
    Ok(Json(DbStatus {
        db_type: format!("{:?}", db.db_type()),
        url: db.url().to_string(),
        connected: db.health_check().unwrap_or(false),
    }))
}

async fn db_health_check(State(state): State<AppState>) -> Result<&'static str, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    db.health_check().map_err(|e| AppError::Database(e.to_string()))?;
    Ok("Database healthy")
}

async fn get_all_players(State(state): State<AppState>) -> Result<Json<Vec<crate::database::models::PlayerStats>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = PlayerStatsRepository::new(Arc::new(db.clone()));
    let players = repo.get_all_players()?;
    Ok(Json(players))
}

async fn create_player(
    State(state): State<AppState>,
    Json(new_player): Json<crate::database::models::NewPlayerStats>,
) -> Result<Json<crate::database::models::PlayerStats>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = PlayerStatsRepository::new(Arc::new(db.clone()));
    let player = repo.create_player(new_player)?;
    Ok(Json(player))
}

async fn get_player(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<Json<crate::database::models::PlayerStats>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = PlayerStatsRepository::new(Arc::new(db.clone()));
    let player = repo.get_player_by_uuid(&uuid)?
        .ok_or_else(|| AppError::NotFound(format!("Player {} not found", uuid)))?;
    Ok(Json(player))
}

async fn update_player(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
    Json(updates): Json<PlayerStatsUpdate>,
) -> Result<Json<crate::database::models::PlayerStats>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = PlayerStatsRepository::new(Arc::new(db.clone()));
    let player = repo.update_player_stats(&uuid, updates)?;
    Ok(Json(player))
}

async fn delete_player(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<StatusCode, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = PlayerStatsRepository::new(Arc::new(db.clone()));
    repo.delete_player(&uuid)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_top_players(
    State(state): State<AppState>,
    Path(limit): Path<i64>,
) -> Result<Json<Vec<crate::database::models::PlayerStats>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = PlayerStatsRepository::new(Arc::new(db.clone()));
    let players = repo.get_top_players(limit)?;
    Ok(Json(players))
}

async fn get_total_stats(State(state): State<AppState>) -> Result<Json<TotalStats>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = PlayerStatsRepository::new(Arc::new(db.clone()));
    let stats = repo.get_total_stats()?;
    Ok(Json(stats))
}

async fn get_all_balances(State(state): State<AppState>) -> Result<Json<Vec<crate::database::models::Economy>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = EconomyRepository::new(Arc::new(db.clone()));
    let accounts = repo.get_all_accounts()?;
    Ok(Json(accounts))
}

async fn get_richest_players(
    State(state): State<AppState>,
    Path(limit): Path<i64>,
) -> Result<Json<Vec<crate::database::models::Economy>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = EconomyRepository::new(Arc::new(db.clone()));
    let accounts = repo.get_richest_players(limit)?;
    Ok(Json(accounts))
}

async fn get_economy_stats(State(state): State<AppState>) -> Result<Json<EconomyStats>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = EconomyRepository::new(Arc::new(db.clone()));
    let stats = repo.get_economy_stats()?;
    Ok(Json(stats))
}

async fn get_player_balance(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<Json<crate::database::models::Economy>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = EconomyRepository::new(Arc::new(db.clone()));
    let balance = repo.get_player_balance(&uuid)?
        .ok_or_else(|| AppError::NotFound(format!("Economy account {} not found", uuid)))?;
    Ok(Json(balance))
}

async fn deposit(
    State(state): State<AppState>,
    Json(request): Json<TransactionRequest>,
) -> Result<Json<crate::database::models::Economy>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = EconomyRepository::new(Arc::new(db.clone()));
    let economy = repo.deposit(&request.player_uuid, request.amount, request.description)?;
    Ok(Json(economy))
}

async fn withdraw(
    State(state): State<AppState>,
    Json(request): Json<TransactionRequest>,
) -> Result<Json<crate::database::models::Economy>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = EconomyRepository::new(Arc::new(db.clone()));
    let economy = repo.withdraw(&request.player_uuid, request.amount, request.description)?;
    Ok(Json(economy))
}

async fn transfer(
    State(state): State<AppState>,
    Json(request): Json<TransferRequest>,
) -> Result<Json<(crate::database::models::Economy, crate::database::models::Economy)>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = EconomyRepository::new(Arc::new(db.clone()));
    let result = repo.transfer(&request.from_player_uuid, &request.to_player_uuid, request.amount, request.description)?;
    Ok(Json(result))
}

async fn get_transaction_history(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<Json<Vec<crate::database::models::Transaction>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = EconomyRepository::new(Arc::new(db.clone()));
    let transactions = repo.get_transaction_history(&uuid, 100)?;
    Ok(Json(transactions))
}

async fn list_api_keys(State(state): State<AppState>) -> Result<Json<Vec<ApiKeyListItem>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = ApiKeyRepository::new(Arc::new(db.clone()));
    let keys = repo.list_keys()?;
    Ok(Json(keys))
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub key: ApiKeyListItem,
    pub raw_key: String,
}

async fn create_api_key(
    State(state): State<AppState>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiKeyResponse>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = ApiKeyRepository::new(Arc::new(db.clone()));
    let (key, raw_key) = repo.create_api_key(
        request.key_name,
        request.permissions,
        request.rate_limit,
        request.expires_in_days,
    )?;
    Ok(Json(ApiKeyResponse {
        key: ApiKeyListItem {
            id: key.id,
            key_name: key.key_name,
            permissions: serde_json::from_str(&key.permissions).unwrap_or_else(|_| vec![]),
            rate_limit: key.rate_limit,
            is_active: key.is_active,
            expires_at: key.expires_at,
            last_used: key.last_used,
            created_at: key.created_at,
        },
        raw_key,
    }))
}

async fn delete_api_key(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = ApiKeyRepository::new(Arc::new(db.clone()));
    repo.delete_key(id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_api_key(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(updates): Json<ApiKeyUpdate>,
) -> Result<Json<crate::database::models::ApiKey>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = ApiKeyRepository::new(Arc::new(db.clone()));
    let key = repo.update_key(id, updates)?;
    Ok(Json(key))
}

async fn revoke_api_key(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let repo = ApiKeyRepository::new(Arc::new(db.clone()));
    repo.revoke_key(id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub table_name: String,
    pub format: Option<String>,
    pub output_path: Option<String>,
}

async fn export_data(
    State(state): State<AppState>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<ExportResult>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ExportImportService::new(Arc::new(db.clone()));
    let output_path = params.output_path.unwrap_or_else(|| format!("/tmp/{}_export.csv", params.table_name));
    let result = service.export_table_to_csv(&params.table_name, &output_path)?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct ImportQuery {
    pub table_name: String,
    pub input_path: String,
}

async fn import_data(
    State(state): State<AppState>,
    Query(params): Query<ImportQuery>,
) -> Result<Json<ImportResult>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ExportImportService::new(Arc::new(db.clone()));
    let result = service.import_csv_to_table(&params.table_name, &params.input_path)?;
    Ok(Json(result))
}

async fn export_all_json(State(state): State<AppState>) -> Result<Json<String>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ExportImportService::new(Arc::new(db.clone()));
    let path = format!("/tmp/all_data_export_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let result = service.export_all_to_json(&path)?;
    Ok(Json(result))
}

async fn import_all_json(
    State(state): State<AppState>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<i32>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ExportImportService::new(Arc::new(db.clone()));
    let json = serde_json::to_string(&data).map_err(|e| AppError::Serialization(e.to_string()))?;
    let temp_path = format!("/tmp/import_{}.json", chrono::Utc::now().timestamp());
    std::fs::write(&temp_path, json).map_err(|e| AppError::Io(e.to_string()))?;
    let count = service.import_all_from_json(&temp_path)?;
    Ok(Json(count))
}

async fn run_optimization(State(state): State<AppState>) -> Result<Json<Vec<OptimizationResult>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = OptimizationService::new(Arc::new(db.clone()));
    let results = service.run_full_optimization().await?;
    Ok(Json(results))
}

async fn run_vacuum(State(state): State<AppState>) -> Result<Json<OptimizationResult>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = OptimizationService::new(Arc::new(db.clone()));
    let result = service.run_vacuum().await?;
    Ok(Json(result))
}

async fn run_analyze(State(state): State<AppState>) -> Result<Json<OptimizationResult>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = OptimizationService::new(Arc::new(db.clone()));
    let result = service.run_analyze().await?;
    Ok(Json(result))
}

async fn get_optimization_schedule(State(state): State<AppState>) -> Result<Json<OptimizationSchedule>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = OptimizationService::new(Arc::new(db.clone()));
    let schedule = service.get_optimization_schedule().await;
    Ok(Json(schedule))
}

async fn update_optimization_schedule(
    State(state): State<AppState>,
    Json(schedule): Json<OptimizationSchedule>,
) -> Result<Json<OptimizationSchedule>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = OptimizationService::new(Arc::new(db.clone()));
    service.update_optimization_schedule(schedule.clone()).await?;
    Ok(Json(schedule))
}

async fn get_database_size(State(state): State<AppState>) -> Result<Json<DatabaseSize>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = OptimizationService::new(Arc::new(db.clone()));
    let size = service.get_database_size()?;
    Ok(Json(size))
}

#[derive(Debug, Deserialize)]
pub struct SlowQueriesQuery {
    pub threshold_ms: Option<i32>,
    pub limit: Option<i64>,
}

async fn get_slow_queries(
    State(state): State<AppState>,
    Query(params): Query<SlowQueriesQuery>,
) -> Result<Json<Vec<crate::database::models::QueryMetric>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let analyzer = PerformanceAnalyzer::new(Arc::new(db.clone()));
    let queries = analyzer.get_slow_queries(params.threshold_ms.unwrap_or(100), params.limit.unwrap_or(50))?;
    Ok(Json(queries))
}

async fn get_all_query_stats(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, i64>>,
) -> Result<Json<Vec<QueryStatsSummary>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let analyzer = PerformanceAnalyzer::new(Arc::new(db.clone()));
    let hours = params.get("hours").copied().unwrap_or(24) as i64;
    let stats = analyzer.get_all_query_stats(hours)?;
    Ok(Json(stats))
}

async fn get_query_stats_by_type(
    State(state): State<AppState>,
    Path(query_type): Path<String>,
    Query(params): Query<HashMap<String, i64>>,
) -> Result<Json<QueryStats>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let analyzer = PerformanceAnalyzer::new(Arc::new(db.clone()));
    let hours = params.get("hours").copied().unwrap_or(24) as i64;
    let stats = analyzer.get_query_stats(&query_type, hours)?;
    Ok(Json(stats))
}

async fn cleanup_old_metrics(
    State(state): State<AppState>,
    Json(params): Json<HashMap<String, i64>>,
) -> Result<Json<i32>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let analyzer = PerformanceAnalyzer::new(Arc::new(db.clone()));
    let retention_days = params.get("retention_days").copied().unwrap_or(7) as i64;
    let deleted = analyzer.cleanup_old_metrics(retention_days)?;
    Ok(Json(deleted))
}

#[derive(Debug, Deserialize)]
pub struct ArchiveQuery {
    pub table_name: String,
    pub older_than_days: Option<i64>,
    pub output_dir: Option<String>,
}

async fn run_archive(
    State(state): State<AppState>,
    Query(params): Query<ArchiveQuery>,
) -> Result<Json<ArchiveResult>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ArchiveService::new(Arc::new(db.clone()));
    let output_dir = params.output_dir.unwrap_or_else(|| "/tmp/archives".to_string());
    let older_than_days = params.older_than_days.unwrap_or(90);
    
    let result = match params.table_name.as_str() {
        "transactions" => service.archive_transactions(older_than_days, &output_dir),
        "player_stats" => service.archive_player_stats(older_than_days, &output_dir),
        _ => return Err(AppError::Validation(format!("Unknown table: {}", params.table_name))),
    }?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct RestoreArchiveQuery {
    pub archive_path: String,
    pub target_table: String,
}

async fn restore_from_archive(
    State(state): State<AppState>,
    Query(params): Query<RestoreArchiveQuery>,
) -> Result<Json<i32>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ArchiveService::new(Arc::new(db.clone()));
    let restored = service.restore_from_archive(&params.archive_path, &params.target_table)?;
    Ok(Json(restored))
}

async fn list_archives(State(state): State<AppState>) -> Result<Json<Vec<crate::database::models::ArchiveMetadata>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ArchiveService::new(Arc::new(db.clone()));
    let archives = service.list_archives()?;
    Ok(Json(archives))
}

async fn delete_archive_record(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ArchiveService::new(Arc::new(db.clone()));
    service.delete_archive(id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn cleanup_expired_archives(State(state): State<AppState>) -> Result<Json<i32>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = ArchiveService::new(Arc::new(db.clone()));
    let deleted = service.cleanup_expired_archives()?;
    Ok(Json(deleted))
}

async fn start_sync(
    State(state): State<AppState>,
    Json(request): Json<SyncRequest>,
) -> Result<Json<SyncResult>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = SyncService::new(Arc::new(db.clone()));
    let result = service.sync_player_stats_to_external(&request.target_url).await?;
    Ok(Json(result))
}

async fn get_sync_status(State(state): State<AppState>) -> Result<Json<Vec<SyncStatusRecord>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = SyncService::new(Arc::new(db.clone()));
    let status = service.get_sync_status().await?;
    Ok(Json(status))
}

async fn cancel_sync(
    State(state): State<AppState>,
    Path(sync_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = SyncService::new(Arc::new(db.clone()));
    service.cancel_sync(&sync_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_backup(
    State(state): State<AppState>,
    Json(params): Json<BackupRequest>,
) -> Result<Json<BackupResult>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = BackupService::new(Arc::new(db.clone()), std::path::PathBuf::from("/tmp/backups"));
    let result = service.create_backup(params.name, params.backup_type)?;
    Ok(Json(result))
}

async fn list_backups(State(state): State<AppState>) -> Result<Json<Vec<crate::database::models::BackupMetadata>>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = BackupService::new(Arc::new(db.clone()), std::path::PathBuf::from("/tmp/backups"));
    let backups = service.list_backups()?;
    Ok(Json(backups))
}

async fn delete_backup_record(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = BackupService::new(Arc::new(db.clone()), std::path::PathBuf::from("/tmp/backups"));
    service.delete_backup(id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn restore_backup_record(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<RestoreResult>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = BackupService::new(Arc::new(db.clone()), std::path::PathBuf::from("/tmp/backups"));
    
    let backups = service.list_backups()?;
    let backup = backups.iter().find(|b| b.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Backup {} not found", id)))?;
    
    let result = service.restore_backup(&backup.backup_path)?;
    Ok(Json(result))
}

async fn verify_backup_record(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<bool>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = BackupService::new(Arc::new(db.clone()), std::path::PathBuf::from("/tmp/backups"));
    
    let backups = service.list_backups()?;
    let backup = backups.iter().find(|b| b.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Backup {} not found", id)))?;
    
    let verified = service.verify_backup(&backup.backup_path)?;
    Ok(Json(verified))
}

async fn cleanup_old_backups(
    State(state): State<AppState>,
    Json(params): Json<HashMap<String, i64>>,
) -> Result<Json<i32>, AppError> {
    let db = state.db_manager.as_ref().ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;
    let service = BackupService::new(Arc::new(db.clone()), std::path::PathBuf::from("/tmp/backups"));
    let retention_days = params.get("retention_days").copied().unwrap_or(30) as i64;
    let deleted = service.cleanup_old_backups(retention_days)?;
    Ok(Json(deleted))
}

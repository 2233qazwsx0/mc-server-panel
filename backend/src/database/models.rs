use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::player_stats)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PlayerStats {
    pub id: i32,
    pub player_uuid: String,
    pub player_name: String,
    pub play_time_seconds: i64,
    pub blocks_placed: i32,
    pub blocks_broken: i32,
    pub deaths: i32,
    pub kills: i32,
    pub last_login: Option<NaiveDateTime>,
    pub first_join: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::player_stats)]
pub struct NewPlayerStats {
    pub player_uuid: String,
    pub player_name: String,
    pub play_time_seconds: Option<i64>,
    pub blocks_placed: Option<i32>,
    pub blocks_broken: Option<i32>,
    pub deaths: Option<i32>,
    pub kills: Option<i32>,
    pub last_login: Option<NaiveDateTime>,
    pub first_join: Option<NaiveDateTime>,
}

impl Default for NewPlayerStats {
    fn default() -> Self {
        Self {
            player_uuid: Uuid::new_v4().to_string(),
            player_name: String::new(),
            play_time_seconds: Some(0),
            blocks_placed: Some(0),
            blocks_broken: Some(0),
            deaths: Some(0),
            kills: Some(0),
            last_login: None,
            first_join: Some(Utc::now().naive_utc()),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::economy)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Economy {
    pub id: i32,
    pub player_uuid: String,
    pub balance: f64,
    pub total_earned: f64,
    pub total_spent: f64,
    pub transaction_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::economy)]
pub struct NewEconomy {
    pub player_uuid: String,
    pub balance: Option<f64>,
    pub total_earned: Option<f64>,
    pub total_spent: Option<f64>,
    pub transaction_count: Option<i32>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::transactions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Transaction {
    pub id: i32,
    pub player_uuid: String,
    pub transaction_type: String,
    pub amount: f64,
    pub balance_before: f64,
    pub balance_after: f64,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::transactions)]
pub struct NewTransaction {
    pub player_uuid: String,
    pub transaction_type: String,
    pub amount: f64,
    pub balance_before: f64,
    pub balance_after: f64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::api_keys)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ApiKey {
    pub id: i32,
    pub key_hash: String,
    pub key_name: String,
    pub permissions: String,
    pub rate_limit: i32,
    pub is_active: bool,
    pub expires_at: Option<NaiveDateTime>,
    pub last_used: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::api_keys)]
pub struct NewApiKey {
    pub key_hash: String,
    pub key_name: String,
    pub permissions: Option<String>,
    pub rate_limit: Option<i32>,
    pub is_active: Option<bool>,
    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::query_metrics)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct QueryMetric {
    pub id: i32,
    pub query_hash: String,
    pub query_type: String,
    pub execution_time_ms: i32,
    pub rows_affected: i32,
    pub table_name: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::query_metrics)]
pub struct NewQueryMetric {
    pub query_hash: String,
    pub query_type: String,
    pub execution_time_ms: i32,
    pub rows_affected: Option<i32>,
    pub table_name: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::archive_metadata)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ArchiveMetadata {
    pub id: i32,
    pub archive_name: String,
    pub archive_type: String,
    pub source_table: String,
    pub record_count: i32,
    pub file_size_bytes: Option<i64>,
    pub compressed: bool,
    pub archived_at: NaiveDateTime,
    pub retention_days: i32,
    pub auto_delete: bool,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::backup_metadata)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BackupMetadata {
    pub id: i32,
    pub backup_name: String,
    pub backup_path: String,
    pub backup_type: String,
    pub file_size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::database::schema::sync_status)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SyncStatus {
    pub id: i32,
    pub sync_type: String,
    pub target_system: Option<String>,
    pub last_sync_at: Option<NaiveDateTime>,
    pub status: String,
    pub records_synced: i32,
    pub errors: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub db_type: DatabaseType,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Sqlite,
    Mysql,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://data.db".to_string(),
            db_type: DatabaseType::Sqlite,
            max_connections: 10,
            connection_timeout: 30,
        }
    }
}

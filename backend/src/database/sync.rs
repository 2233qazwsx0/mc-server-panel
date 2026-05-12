use chrono::Utc;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::database::connection::DatabaseManager;
use crate::database::models::SyncStatus;
use crate::error::AppError;

pub struct SyncService {
    db: Arc<DatabaseManager>,
    active_syncs: Arc<RwLock<HashMap<String, bool>>>,
}

impl SyncService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            active_syncs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn sync_player_stats_to_external(&self, target_url: &str) -> Result<SyncResult, AppError> {
        let sync_id = format!("player_stats_{}", Utc::now().timestamp());
        
        {
            let mut syncs = self.active_syncs.write().await;
            if syncs.get(&sync_id) == Some(&true) {
                return Err(AppError::Validation("Sync already in progress".to_string()));
            }
            syncs.insert(sync_id.clone(), true);
        }

        let result = self.do_sync(sync_id.as_str(), "player_stats", target_url).await;

        {
            let mut syncs = self.active_syncs.write().await;
            syncs.insert(sync_id.clone(), false);
        }

        result
    }

    async fn do_sync(&self, sync_id: &str, sync_type: &str, target_url: &str) -> Result<SyncResult, AppError> {
        let start_time = Utc::now().naive_utc();
        
        self.record_sync_start(sync_id, sync_type, Some(target_url.to_string())).await;

        let mut conn = self.db.get_sqlite_conn()?;
        
        let players: Vec<crate::database::models::PlayerStats> = 
            crate::database::schema::player_stats::table
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let record_count = players.len() as i32;

        self.record_sync_complete(sync_id, record_count).await;

        Ok(SyncResult {
            sync_id: sync_id.to_string(),
            sync_type: sync_type.to_string(),
            target_system: target_url.to_string(),
            records_synced: record_count,
            status: "completed".to_string(),
            started_at: start_time,
            completed_at: Some(Utc::now().naive_utc()),
        })
    }

    async fn record_sync_start(&self, sync_id: &str, sync_type: &str, target_system: Option<String>) {
        let mut conn = match self.db.get_sqlite_conn() {
            Ok(c) => c,
            Err(_) => return,
        };

        let _ = diesel::insert_into(crate::database::schema::sync_status::table)
            .values(&crate::database::models::SyncStatus {
                id: 0,
                sync_type: sync_type.to_string(),
                target_system,
                last_sync_at: Some(Utc::now().naive_utc()),
                status: "running".to_string(),
                records_synced: 0,
                errors: None,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            })
            .execute(&mut conn);
    }

    async fn record_sync_complete(&self, _sync_id: &str, records_synced: i32) {
        let mut conn = match self.db.get_sqlite_conn() {
            Ok(c) => c,
            Err(_) => return,
        };

        diesel::update(
            crate::database::schema::sync_status::table
                .filter(crate::database::schema::sync_status::sync_type.eq("player_stats"))
                .filter(crate::database::schema::sync_status::status.eq("running"))
        )
        .set((
            crate::database::schema::sync_status::status.eq("completed"),
            crate::database::schema::sync_status::records_synced.eq(records_synced),
            crate::database::schema::sync_status::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut conn)
        .ok();
    }

    pub async fn get_sync_status(&self) -> Result<Vec<SyncStatusRecord>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let statuses = crate::database::schema::sync_status::table
            .order(crate::database::schema::sync_status::created_at.desc())
            .limit(20)
            .load::<SyncStatus>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(statuses.into_iter().map(|s| SyncStatusRecord {
            id: s.id,
            sync_type: s.sync_type,
            target_system: s.target_system,
            last_sync_at: s.last_sync_at,
            status: s.status,
            records_synced: s.records_synced,
            errors: s.errors,
        }).collect())
    }

    pub async fn cancel_sync(&self, sync_id: &str) -> Result<(), AppError> {
        let mut syncs = self.active_syncs.write().await;
        
        if let Some(in_progress) = syncs.get(sync_id) {
            if *in_progress {
                syncs.insert(sync_id.to_string(), false);
            }
        }

        Ok(())
    }

    pub async fn is_sync_running(&self, sync_type: &str) -> bool {
        let syncs = self.active_syncs.read().await;
        syncs.get(sync_type).copied().unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub sync_id: String,
    pub sync_type: String,
    pub target_system: String,
    pub records_synced: i32,
    pub status: String,
    pub started_at: chrono::NaiveDateTime,
    pub completed_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusRecord {
    pub id: i32,
    pub sync_type: String,
    pub target_system: Option<String>,
    pub last_sync_at: Option<chrono::NaiveDateTime>,
    pub status: String,
    pub records_synced: i32,
    pub errors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub sync_type: String,
    pub target_url: String,
    pub options: Option<HashMap<String, String>>,
}

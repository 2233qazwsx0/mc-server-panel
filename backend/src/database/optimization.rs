use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::database::connection::DatabaseManager;
use crate::error::AppError;

pub struct OptimizationService {
    db: Arc<DatabaseManager>,
    last_vacuum: Arc<RwLock<Option<chrono::NaiveDateTime>>>,
    last_analyze: Arc<RwLock<Option<chrono::NaiveDateTime>>>,
}

impl OptimizationService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            last_vacuum: Arc::new(RwLock::new(None)),
            last_analyze: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn run_vacuum(&self) -> Result<OptimizationResult, AppError> {
        let start = Instant::now();
        let mut conn = self.db.get_sqlite_conn()?;

        diesel::sql_query("VACUUM")
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as i32;
        let mut last_vacuum = self.last_vacuum.write().await;
        *last_vacuum = Some(chrono::Utc::now().naive_utc());

        Ok(OptimizationResult {
            operation: "vacuum".to_string(),
            duration_ms,
            success: true,
            message: "Database vacuum completed successfully".to_string(),
            executed_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub async fn run_analyze(&self) -> Result<OptimizationResult, AppError> {
        let start = Instant::now();
        let mut conn = self.db.get_sqlite_conn()?;

        diesel::sql_query("ANALYZE")
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as i32;
        let mut last_analyze = self.last_analyze.write().await;
        *last_analyze = Some(chrono::Utc::now().naive_utc());

        Ok(OptimizationResult {
            operation: "analyze".to_string(),
            duration_ms,
            success: true,
            message: "Database analyze completed successfully".to_string(),
            executed_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub async fn run_full_optimization(&self) -> Result<Vec<OptimizationResult>, AppError> {
        let mut results = vec![];
        
        results.push(self.run_vacuum().await?);
        results.push(self.run_analyze().await?);
        
        results.push(self.rebuild_indexes().await?);

        Ok(results)
    }

    pub async fn rebuild_indexes(&self) -> Result<OptimizationResult, AppError> {
        let start = Instant::now();
        let mut conn = self.db.get_sqlite_conn()?;

        diesel::sql_query("REINDEX")
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as i32;

        Ok(OptimizationResult {
            operation: "reindex".to_string(),
            duration_ms,
            success: true,
            message: "Database indexes rebuilt successfully".to_string(),
            executed_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub async fn get_last_vacuum(&self) -> Option<chrono::NaiveDateTime> {
        self.last_vacuum.read().await.clone()
    }

    pub async fn get_last_analyze(&self) -> Option<chrono::NaiveDateTime> {
        self.last_analyze.read().await.clone()
    }

    pub async fn get_optimization_schedule(&self) -> OptimizationSchedule {
        OptimizationSchedule {
            auto_vacuum: true,
            auto_analyze: true,
            vacuum_interval_hours: 24,
            analyze_interval_hours: 6,
            last_vacuum: self.get_last_vacuum().await,
            last_analyze: self.get_last_analyze().await,
        }
    }

    pub async fn update_optimization_schedule(&self, schedule: OptimizationSchedule) -> Result<(), AppError> {
        tracing::info!("Updating optimization schedule: {:?}", schedule);
        Ok(())
    }

    pub fn get_database_size(&self) -> Result<DatabaseSize, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let page_count: (i64,) = diesel::sql_query("PRAGMA page_count")
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?
            .pop()
            .unwrap_or((0,));

        let page_size: (i64,) = diesel::sql_query("PRAGMA page_size")
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?
            .pop()
            .unwrap_or((0,));

        let freelist_count: (i64,) = diesel::sql_query("PRAGMA freelist_count")
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?
            .pop()
            .unwrap_or((0,));

        let db_size_bytes = page_count.0 * page_size.0;
        let freelist_bytes = freelist_count.0 * page_size.0;
        let usable_size_bytes = db_size_bytes - freelist_bytes;

        Ok(DatabaseSize {
            total_bytes: db_size_bytes,
            usable_bytes: usable_size_bytes,
            freelist_bytes,
            page_size: page_size.0,
            page_count: page_count.0,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub operation: String,
    pub duration_ms: i32,
    pub success: bool,
    pub message: String,
    pub executed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSchedule {
    pub auto_vacuum: bool,
    pub auto_analyze: bool,
    pub vacuum_interval_hours: u32,
    pub analyze_interval_hours: u32,
    pub last_vacuum: Option<chrono::NaiveDateTime>,
    pub last_analyze: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSize {
    pub total_bytes: i64,
    pub usable_bytes: i64,
    pub freelist_bytes: i64,
    pub page_size: i64,
    pub page_count: i64,
}

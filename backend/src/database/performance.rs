use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use std::time::Instant;

use crate::database::models::{NewQueryMetric, QueryMetric};
use crate::database::connection::DatabaseManager;
use crate::error::AppError;

pub struct PerformanceAnalyzer {
    db: Arc<DatabaseManager>,
}

impl PerformanceAnalyzer {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub fn record_query<T, F>(&self, query_type: &str, table_name: Option<&str>, f: F) -> Result<T, AppError>
    where
        F: FnOnce() -> Result<T, AppError>,
    {
        let start = Instant::now();
        let result = f()?;
        let duration_ms = start.elapsed().as_millis() as i32;

        let query_hash = format!("{}_{}", query_type, Utc::now().timestamp());

        let _ = self.record_metric(
            &query_hash,
            query_type,
            duration_ms,
            0,
            table_name,
        );

        Ok(result)
    }

    pub fn record_metric(
        &self,
        query_hash: &str,
        query_type: &str,
        execution_time_ms: i32,
        rows_affected: i32,
        table_name: Option<&str>,
    ) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let metric = NewQueryMetric {
            query_hash: query_hash.to_string(),
            query_type: query_type.to_string(),
            execution_time_ms,
            rows_affected: Some(rows_affected),
            table_name: table_name.map(|s| s.to_string()),
        };

        diesel::insert_into(crate::database::schema::query_metrics::table)
            .values(&metric)
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn get_slow_queries(&self, threshold_ms: i32, limit: i64) -> Result<Vec<QueryMetric>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let queries = crate::database::schema::query_metrics::table
            .filter(crate::database::schema::query_metrics::execution_time_ms.gt(threshold_ms))
            .order(crate::database::schema::query_metrics::execution_time_ms.desc())
            .limit(limit)
            .load::<QueryMetric>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(queries)
    }

    pub fn get_query_stats(&self, query_type: &str, hours: i64) -> Result<QueryStats, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let since = Utc::now() - Duration::hours(hours);

        let stats: (Option<f64>, Option<f64>, Option<f64>, Option<i64>, Option<i32>, Option<i32>) = 
            crate::database::schema::query_metrics::table
            .filter(crate::database::schema::query_metrics::query_type.eq(query_type))
            .filter(crate::database::schema::query_metrics::created_at.gt(since.naive_utc()))
            .select((
                diesel::dsl::sql::<diesel::sql_types::Double>("AVG(execution_time_ms)"),
                diesel::dsl::sql::<diesel::sql_types::Double>("MAX(execution_time_ms)"),
                diesel::dsl::sql::<diesel::sql_types::Double>("MIN(execution_time_ms)"),
                diesel::dsl::sql::<diesel::sql_types::BigInt>("SUM(rows_affected)"),
                diesel::dsl::sql::<diesel::sql_types::Integer>("COUNT(*)"),
                diesel::dsl::sql::<diesel::sql_types::Integer>("MAX(execution_time_ms)"),
            ))
            .first(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(QueryStats {
            query_type: query_type.to_string(),
            avg_execution_time_ms: stats.0.unwrap_or(0.0),
            max_execution_time_ms: stats.1.unwrap_or(0.0),
            min_execution_time_ms: stats.2.unwrap_or(0.0),
            total_rows_affected: stats.3.unwrap_or(0),
            total_queries: stats.4.unwrap_or(0),
            p95_execution_time_ms: stats.5.unwrap_or(0),
        })
    }

    pub fn get_all_query_stats(&self, hours: i64) -> Result<Vec<QueryStatsSummary>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let since = Utc::now() - Duration::hours(hours);

        let summaries: Vec<(String, f64, f64, f64, i64, i64)> = diesel::sql_query(&format!(
            r#"SELECT 
                query_type,
                AVG(execution_time_ms) as avg_time,
                MAX(execution_time_ms) as max_time,
                MIN(execution_time_ms) as min_time,
                SUM(rows_affected) as total_rows,
                COUNT(*) as query_count
            FROM query_metrics 
            WHERE created_at > '{}'
            GROUP BY query_type
            ORDER BY avg_time DESC"#,
            since.naive_utc()
        ))
        .load(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(summaries.into_iter().map(|s| QueryStatsSummary {
            query_type: s.0,
            avg_execution_time_ms: s.1,
            max_execution_time_ms: s.2,
            min_execution_time_ms: s.3,
            total_rows_affected: s.4,
            total_queries: s.5,
        }).collect())
    }

    pub fn cleanup_old_metrics(&self, retention_days: i64) -> Result<i32, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let cutoff = Utc::now() - Duration::days(retention_days);

        let deleted = diesel::delete(
            crate::database::schema::query_metrics::table
                .filter(crate::database::schema::query_metrics::created_at.lt(cutoff.naive_utc()))
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(deleted as i32)
    }

    pub fn get_database_stats(&self) -> Result<DatabaseStats, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let total_queries: i64 = crate::database::schema::query_metrics::table
            .count()
            .get_result(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let avg_time: (f64,) = diesel::sql_query(
            "SELECT AVG(execution_time_ms) FROM query_metrics"
        )
        .load(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?
        .pop()
        .unwrap_or((0.0,));

        let slow_queries: i64 = crate::database::schema::query_metrics::table
            .filter(crate::database::schema::query_metrics::execution_time_ms.gt(100))
            .count()
            .get_result(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(DatabaseStats {
            total_queries,
            avg_query_time_ms: avg_time.0,
            slow_query_count: slow_queries,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub query_type: String,
    pub avg_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub min_execution_time_ms: f64,
    pub total_rows_affected: i64,
    pub total_queries: i64,
    pub p95_execution_time_ms: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStatsSummary {
    pub query_type: String,
    pub avg_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub min_execution_time_ms: f64,
    pub total_rows_affected: i64,
    pub total_queries: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_queries: i64,
    pub avg_query_time_ms: f64,
    pub slow_query_count: i64,
}

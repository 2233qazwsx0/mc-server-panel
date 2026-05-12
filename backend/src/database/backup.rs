use chrono::Utc;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::database::connection::DatabaseManager;
use crate::database::models::BackupMetadata;
use crate::error::AppError;

pub struct BackupService {
    db: Arc<DatabaseManager>,
    backup_dir: PathBuf,
}

impl BackupService {
    pub fn new(db: Arc<DatabaseManager>, backup_dir: PathBuf) -> Self {
        Self { db, backup_dir }
    }

    pub fn create_backup(&self, name: Option<String>, backup_type: BackupType) -> Result<BackupResult, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| AppError::Io(e.to_string()))?;

        let backup_name = name.unwrap_or_else(|| format!("backup_{}", Utc::now().format("%Y%m%d_%H%M%S")));
        let backup_filename = format!("{}.sql", backup_name);
        let backup_path = self.backup_dir.join(&backup_filename);

        self.export_database_to_sql(&mut conn, &backup_path)?;

        let metadata = fs::metadata(&backup_path)
            .map_err(|e| AppError::Io(e.to_string()))?;
        let file_size = metadata.len() as i64;

        let checksum = self.calculate_checksum(&backup_path)?;

        let mut backup_record = BackupMetadata {
            id: 0,
            backup_name: backup_name.clone(),
            backup_path: backup_path.to_string_lossy().to_string(),
            backup_type: format!("{:?}", backup_type),
            file_size_bytes: Some(file_size),
            checksum: Some(checksum.clone()),
            status: "completed".to_string(),
            created_at: Utc::now().naive_utc(),
            completed_at: Some(Utc::now().naive_utc()),
        };

        diesel::insert_into(crate::database::schema::backup_metadata::table)
            .values(&backup_record)
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(BackupResult {
            backup_name,
            backup_path: backup_path.to_string_lossy().to_string(),
            file_size_bytes: file_size,
            checksum,
            created_at: Utc::now().naive_utc(),
        })
    }

    fn export_database_to_sql(&self, conn: &mut SqliteConnection, output_path: &Path) -> Result<(), AppError> {
        let mut file = File::create(output_path)
            .map_err(|e| AppError::Io(e.to_string()))?;

        writeln!(file, "-- Database Backup: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"))
            .map_err(|e| AppError::Io(e.to_string()))?;
        writeln!(file, "-- SQLite Database Backup")
            .map_err(|e| AppError::Io(e.to_string()))?;
        writeln!(file).map_err(|e| AppError::Io(e.to_string()))?;

        writeln!(file, "BEGIN TRANSACTION;").map_err(|e| AppError::Io(e.to_string()))?;

        let tables = vec!["player_stats", "economy", "transactions", "api_keys", "query_metrics", "archive_metadata", "backup_metadata", "sync_status"];
        
        for table in tables {
            writeln!(file, "\n-- Table: {}", table).map_err(|e| AppError::Io(e.to_string()))?;
            
            let query = format!("SELECT * FROM {}", table);
            let rows: Vec<HashMap<String, serde_json::Value>> = diesel::sql_query(&query)
                .load(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

            if !rows.is_empty() {
                let columns: Vec<String> = rows[0].keys().cloned().collect();
                writeln!(file, "INSERT INTO {} ({}) VALUES", table, columns.join(", "))
                    .map_err(|e| AppError::Io(e.to_string()))?;

                for (i, row) in rows.iter().enumerate() {
                    let values: Vec<String> = columns.iter().map(|col| {
                        match row.get(col) {
                            Some(serde_json::Value::Null) => "NULL".to_string(),
                            Some(serde_json::Value::String(s)) => format!("'{}'", s.replace('\'', "''")),
                            Some(v) => format!("'{}'", v.to_string().replace('\'', "''")),
                            None => "NULL".to_string(),
                        }
                    }).collect();
                    
                    let suffix = if i == rows.len() - 1 { ";" } else { "," };
                    writeln!(file, "  ({}){}", values.join(", "), suffix)
                        .map_err(|e| AppError::Io(e.to_string()))?;
                }
            }
        }

        writeln!(file, "\nCOMMIT;").map_err(|e| AppError::Io(e.to_string()))?;

        file.flush().map_err(|e| AppError::Io(e.to_string()))?;

        Ok(())
    }

    pub fn restore_backup(&self, backup_path: &str) -> Result<RestoreResult, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let backup_file = Path::new(backup_path);
        if !backup_file.exists() {
            return Err(AppError::NotFound(format!("Backup file not found: {}", backup_path)));
        }

        let content = fs::read_to_string(backup_path)
            .map_err(|e| AppError::Io(e.to_string()))?;

        conn.transaction::<_, AppError, _>(|conn| {
            diesel::sql_query("DELETE FROM transactions")
                .execute(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;
            diesel::sql_query("DELETE FROM economy")
                .execute(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;
            diesel::sql_query("DELETE FROM player_stats")
                .execute(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

            for statement in content.split(';') {
                let trimmed = statement.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") && !trimmed.starts_with("BEGIN") && !trimmed.starts_with("COMMIT") && !trimmed.starts_with('\n') {
                    diesel::sql_query(trimmed)
                        .execute(conn)
                        .ok();
                }
            }

            Ok(())
        })?;

        Ok(RestoreResult {
            backup_name: Path::new(backup_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            restored_tables: vec!["player_stats".to_string(), "economy".to_string(), "transactions".to_string()],
            restored_at: Utc::now().naive_utc(),
        })
    }

    pub fn list_backups(&self) -> Result<Vec<BackupMetadata>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let backups = crate::database::schema::backup_metadata::table
            .order(crate::database::schema::backup_metadata::created_at.desc())
            .load::<BackupMetadata>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(backups)
    }

    pub fn delete_backup(&self, backup_id: i32) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let backup: Option<BackupMetadata> = crate::database::schema::backup_metadata::table
            .filter(crate::database::schema::backup_metadata::id.eq(backup_id))
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(backup) = backup {
            let path = Path::new(&backup.backup_path);
            if path.exists() {
                fs::remove_file(path)
                    .map_err(|e| AppError::Io(e.to_string()))?;
            }
        }

        diesel::delete(
            crate::database::schema::backup_metadata::table
                .filter(crate::database::schema::backup_metadata::id.eq(backup_id))
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn verify_backup(&self, backup_path: &str) -> Result<bool, AppError> {
        let backup_file = Path::new(backup_path);
        if !backup_file.exists() {
            return Ok(false);
        }

        let mut conn = self.db.get_sqlite_conn()?;

        let backup: Option<BackupMetadata> = crate::database::schema::backup_metadata::table
            .filter(crate::database::schema::backup_metadata::backup_path.eq(backup_path))
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(backup) = backup {
            if let Some(expected_checksum) = backup.checksum {
                let actual_checksum = self.calculate_checksum(backup_file)?;
                return Ok(expected_checksum == actual_checksum);
            }
        }

        Ok(true)
    }

    fn calculate_checksum(&self, path: &Path) -> Result<String, AppError> {
        let content = fs::read(path)
            .map_err(|e| AppError::Io(e.to_string()))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn cleanup_old_backups(&self, retention_days: i64) -> Result<i32, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);

        let old_backups: Vec<BackupMetadata> = crate::database::schema::backup_metadata::table
            .filter(crate::database::schema::backup_metadata::created_at.lt(cutoff.naive_utc()))
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut deleted_count = 0;
        for backup in old_backups {
            let path = Path::new(&backup.backup_path);
            if path.exists() {
                let _ = fs::remove_file(path);
            }

            diesel::delete(
                crate::database::schema::backup_metadata::table
                    .filter(crate::database::schema::backup_metadata::id.eq(backup.id))
            )
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

            deleted_count += 1;
        }

        Ok(deleted_count)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub backup_name: String,
    pub backup_path: String,
    pub file_size_bytes: i64,
    pub checksum: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub backup_name: String,
    pub restored_tables: Vec<String>,
    pub restored_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental,
    Compressed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequest {
    pub name: Option<String>,
    pub backup_type: BackupType,
}

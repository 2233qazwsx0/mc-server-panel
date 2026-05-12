use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use crate::database::models::ArchiveMetadata;
use crate::database::connection::DatabaseManager;
use crate::error::AppError;

pub struct ArchiveService {
    db: Arc<DatabaseManager>,
}

impl ArchiveService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub fn archive_transactions(&self, older_than_days: i64, output_dir: &str) -> Result<ArchiveResult, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let cutoff = Utc::now() - Duration::days(older_than_days);

        let transactions: Vec<crate::database::models::Transaction> = 
            crate::database::schema::transactions::table
            .filter(crate::database::schema::transactions::created_at.lt(cutoff.naive_utc()))
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let record_count = transactions.len() as i32;

        if record_count == 0 {
            return Ok(ArchiveResult {
                table_name: "transactions".to_string(),
                archived_count: 0,
                file_path: String::new(),
                file_size_bytes: 0,
                archived_at: Utc::now().naive_utc(),
            });
        }

        fs::create_dir_all(output_dir)
            .map_err(|e| AppError::Io(e.to_string()))?;

        let filename = format!("transactions_archive_{}.csv", Utc::now().format("%Y%m%d_%H%M%S"));
        let file_path = Path::new(output_dir).join(&filename);
        
        let mut file = File::create(&file_path)
            .map_err(|e| AppError::Io(e.to_string()))?;
        
        writeln!(file, "id,player_uuid,transaction_type,amount,balance_before,balance_after,description,created_at")
            .map_err(|e| AppError::Io(e.to_string()))?;

        for tx in &transactions {
            writeln!(file, "{},{},{},{},{},{},{},{}",
                tx.id,
                tx.player_uuid,
                tx.transaction_type,
                tx.amount,
                tx.balance_before,
                tx.balance_after,
                tx.description.as_deref().unwrap_or("").replace(',', ";"),
                tx.created_at
            ).map_err(|e| AppError::Io(e.to_string()))?;
        }

        file.flush().map_err(|e| AppError::Io(e.to_string()))?;

        let metadata = fs::metadata(&file_path)
            .map_err(|e| AppError::Io(e.to_string()))?;
        let file_size = metadata.len() as i64;

        diesel::delete(
            crate::database::schema::transactions::table
                .filter(crate::database::schema::transactions::created_at.lt(cutoff.naive_utc()))
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        let archive_name = format!("transactions_{}", Utc::now().format("%Y%m%d"));
        
        diesel::insert_into(crate::database::schema::archive_metadata::table)
            .values(&crate::database::models::ArchiveMetadata {
                id: 0,
                archive_name: archive_name.clone(),
                archive_type: "transactions".to_string(),
                source_table: "transactions".to_string(),
                record_count,
                file_size_bytes: Some(file_size),
                compressed: false,
                archived_at: Utc::now().naive_utc(),
                retention_days: 90,
                auto_delete: true,
            })
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(ArchiveResult {
            table_name: "transactions".to_string(),
            archived_count: record_count,
            file_path: file_path.to_string_lossy().to_string(),
            file_size_bytes: file_size,
            archived_at: Utc::now().naive_utc(),
        })
    }

    pub fn archive_player_stats(&self, inactive_days: i64, output_dir: &str) -> Result<ArchiveResult, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let cutoff = Utc::now() - Duration::days(inactive_days);

        let players: Vec<crate::database::models::PlayerStats> = 
            crate::database::schema::player_stats::table
            .filter(crate::database::schema::player_stats::last_login.lt(Some(cutoff.naive_utc())))
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let record_count = players.len() as i32;

        if record_count == 0 {
            return Ok(ArchiveResult {
                table_name: "player_stats".to_string(),
                archived_count: 0,
                file_path: String::new(),
                file_size_bytes: 0,
                archived_at: Utc::now().naive_utc(),
            });
        }

        fs::create_dir_all(output_dir)
            .map_err(|e| AppError::Io(e.to_string()))?;

        let filename = format!("player_stats_archive_{}.csv", Utc::now().format("%Y%m%d_%H%M%S"));
        let file_path = Path::new(output_dir).join(&filename);
        
        let mut file = File::create(&file_path)
            .map_err(|e| AppError::Io(e.to_string()))?;
        
        writeln!(file, "id,player_uuid,player_name,play_time_seconds,blocks_placed,blocks_broken,deaths,kills,last_login,first_join,created_at,updated_at")
            .map_err(|e| AppError::Io(e.to_string()))?;

        for player in &players {
            writeln!(file, "{},{},{},{},{},{},{},{},{},{},{},{}",
                player.id,
                player.player_uuid,
                player.player_name.replace(',', ";"),
                player.play_time_seconds,
                player.blocks_placed,
                player.blocks_broken,
                player.deaths,
                player.kills,
                player.last_login.map(|dt| dt.to_string()).unwrap_or_default(),
                player.first_join,
                player.created_at,
                player.updated_at
            ).map_err(|e| AppError::Io(e.to_string()))?;
        }

        file.flush().map_err(|e| AppError::Io(e.to_string()))?;

        let metadata = fs::metadata(&file_path)
            .map_err(|e| AppError::Io(e.to_string()))?;
        let file_size = metadata.len() as i64;

        diesel::delete(
            crate::database::schema::player_stats::table
                .filter(crate::database::schema::player_stats::last_login.lt(Some(cutoff.naive_utc())))
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        let archive_name = format!("player_stats_{}", Utc::now().format("%Y%m%d"));
        
        diesel::insert_into(crate::database::schema::archive_metadata::table)
            .values(&crate::database::models::ArchiveMetadata {
                id: 0,
                archive_name: archive_name.clone(),
                archive_type: "player_stats".to_string(),
                source_table: "player_stats".to_string(),
                record_count,
                file_size_bytes: Some(file_size),
                compressed: false,
                archived_at: Utc::now().naive_utc(),
                retention_days: 365,
                auto_delete: true,
            })
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(ArchiveResult {
            table_name: "player_stats".to_string(),
            archived_count: record_count,
            file_path: file_path.to_string_lossy().to_string(),
            file_size_bytes: file_size,
            archived_at: Utc::now().naive_utc(),
        })
    }

    pub fn restore_from_archive(&self, archive_path: &str, target_table: &str) -> Result<i32, AppError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(archive_path)
            .map_err(|e| AppError::Io(e.to_string()))?;

        let mut conn = self.db.get_sqlite_conn()?;
        let mut restored_count = 0;

        match target_table {
            "transactions" => {
                for result in reader.records() {
                    if let Ok(record) = result {
                        let values: Vec<&str> = record.iter().collect();
                        if values.len() >= 8 {
                            let tx = crate::database::models::NewTransaction {
                                player_uuid: values[1].to_string(),
                                transaction_type: values[2].to_string(),
                                amount: values[3].parse().unwrap_or(0.0),
                                balance_before: values[4].parse().unwrap_or(0.0),
                                balance_after: values[5].parse().unwrap_or(0.0),
                                description: if values[6].is_empty() { None } else { Some(values[6].to_string()) },
                            };
                            
                            if diesel::insert_into(crate::database::schema::transactions::table)
                                .values(&tx)
                                .execute(&mut conn)
                                .is_ok()
                            {
                                restored_count += 1;
                            }
                        }
                    }
                }
            }
            _ => return Err(AppError::Validation(format!("Unknown archive table: {}", target_table))),
        }

        Ok(restored_count)
    }

    pub fn list_archives(&self) -> Result<Vec<ArchiveMetadata>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        let archives = crate::database::schema::archive_metadata::table
            .order(crate::database::schema::archive_metadata::archived_at.desc())
            .load::<ArchiveMetadata>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(archives)
    }

    pub fn delete_archive(&self, archive_id: i32) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;

        diesel::delete(
            crate::database::schema::archive_metadata::table
                .filter(crate::database::schema::archive_metadata::id.eq(archive_id))
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn cleanup_expired_archives(&self) -> Result<i32, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        let mut deleted_count = 0;

        let archives: Vec<ArchiveMetadata> = crate::database::schema::archive_metadata::table
            .filter(crate::database::schema::archive_metadata::auto_delete.eq(true))
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        for archive in archives {
            let cutoff = archive.archived_at + Duration::days(archive.retention_days as i64);
            if cutoff < Utc::now().naive_utc() {
                if let Some(file_path) = Path::new(&archive.archive_name).to_str() {
                    if Path::new(file_path).exists() {
                        let _ = fs::remove_file(file_path);
                    }
                }

                diesel::delete(
                    crate::database::schema::archive_metadata::table
                        .filter(crate::database::schema::archive_metadata::id.eq(archive.id))
                )
                .execute(&mut conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveResult {
    pub table_name: String,
    pub archived_count: i32,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub archived_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRequest {
    pub table_name: String,
    pub older_than_days: i64,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePolicy {
    pub archive_transactions: bool,
    pub transaction_retention_days: i64,
    pub archive_inactive_players: bool,
    pub inactive_days_threshold: i64,
    pub auto_cleanup: bool,
}

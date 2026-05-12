use chrono::Utc;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use crate::database::connection::DatabaseManager;
use crate::error::AppError;

pub struct ExportImportService {
    db: Arc<DatabaseManager>,
}

impl ExportImportService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub fn export_table_to_csv(&self, table_name: &str, output_path: &str) -> Result<ExportResult, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let records = match table_name {
            "player_stats" => {
                use crate::database::schema::player_stats::dsl::*;
                player_stats.load::<crate::database::models::PlayerStats>(&mut conn)
                    .map(|r| {
                        let mut data: Vec<Vec<String>> = vec![];
                        for row in r {
                            data.push(vec![
                                row.id.to_string(),
                                row.player_uuid.clone(),
                                row.player_name.clone(),
                                row.play_time_seconds.to_string(),
                                row.blocks_placed.to_string(),
                                row.blocks_broken.to_string(),
                                row.deaths.to_string(),
                                row.kills.to_string(),
                                row.last_login.map(|dt| dt.to_string()).unwrap_or_default(),
                                row.first_join.to_string(),
                                row.created_at.to_string(),
                                row.updated_at.to_string(),
                            ]);
                        }
                        data
                    })
            }
            "economy" => {
                use crate::database::schema::economy::dsl::*;
                economy.load::<crate::database::models::Economy>(&mut conn)
                    .map(|r| {
                        let mut data: Vec<Vec<String>> = vec![];
                        for row in r {
                            data.push(vec![
                                row.id.to_string(),
                                row.player_uuid.clone(),
                                row.balance.to_string(),
                                row.total_earned.to_string(),
                                row.total_spent.to_string(),
                                row.transaction_count.to_string(),
                                row.created_at.to_string(),
                                row.updated_at.to_string(),
                            ]);
                        }
                        data
                    })
            }
            "transactions" => {
                use crate::database::schema::transactions::dsl::*;
                transactions.load::<crate::database::models::Transaction>(&mut conn)
                    .map(|r| {
                        let mut data: Vec<Vec<String>> = vec![];
                        for row in r {
                            data.push(vec![
                                row.id.to_string(),
                                row.player_uuid.clone(),
                                row.transaction_type.clone(),
                                row.amount.to_string(),
                                row.balance_before.to_string(),
                                row.balance_after.to_string(),
                                row.description.clone().unwrap_or_default(),
                                row.created_at.to_string(),
                            ]);
                        }
                        data
                    })
            }
            _ => return Err(AppError::Validation(format!("Unknown table: {}", table_name))),
        }.map_err(|e| AppError::Database(e.to_string()))?;

        let file = File::create(output_path)
            .map_err(|e| AppError::Io(e.to_string()))?;
        let mut writer = BufWriter::new(file);

        let headers = match table_name {
            "player_stats" => vec!["id", "player_uuid", "player_name", "play_time_seconds", "blocks_placed", "blocks_broken", "deaths", "kills", "last_login", "first_join", "created_at", "updated_at"],
            "economy" => vec!["id", "player_uuid", "balance", "total_earned", "total_spent", "transaction_count", "created_at", "updated_at"],
            "transactions" => vec!["id", "player_uuid", "transaction_type", "amount", "balance_before", "balance_after", "description", "created_at"],
            _ => return Err(AppError::Validation(format!("Unknown table: {}", table_name))),
        };

        writer.write_all(headers.join(",").as_bytes())
            .map_err(|e| AppError::Io(e.to_string()))?;
        writer.write_all(b"\n")
            .map_err(|e| AppError::Io(e.to_string()))?;

        for row in &records {
            let line = row.iter()
                .map(|s| format!("\"{}\"", s.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(",");
            writer.write_all(line.as_bytes())
                .map_err(|e| AppError::Io(e.to_string()))?;
            writer.write_all(b"\n")
                .map_err(|e| AppError::Io(e.to_string()))?;
        }

        writer.flush().map_err(|e| AppError::Io(e.to_string()))?;

        let metadata = fs::metadata(output_path)
            .map_err(|e| AppError::Io(e.to_string()))?;

        Ok(ExportResult {
            table_name: table_name.to_string(),
            record_count: records.len() as i32,
            file_path: output_path.to_string(),
            file_size_bytes: metadata.len() as i64,
            exported_at: Utc::now().naive_utc(),
        })
    }

    pub fn import_csv_to_table(&self, table_name: &str, input_path: &str) -> Result<ImportResult, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(input_path)
            .map_err(|e| AppError::Io(e.to_string()))?;

        let headers = reader.headers()
            .map_err(|e| AppError::Io(e.to_string()))?
            .clone();

        let mut imported_count = 0;
        let mut errors: Vec<String> = vec![];

        match table_name {
            "player_stats" => {
                for result in reader.records() {
                    match result {
                        Ok(record) => {
                            let mut values: Vec<String> = vec![];
                            for field in record.iter() {
                                values.push(field.to_string());
                            }
                            if values.len() >= 12 {
                                let new_player = crate::database::models::NewPlayerStats {
                                    player_uuid: values[1].clone(),
                                    player_name: values[2].clone(),
                                    play_time_seconds: values[3].parse().ok(),
                                    blocks_placed: values[4].parse().ok(),
                                    blocks_broken: values[5].parse().ok(),
                                    deaths: values[6].parse().ok(),
                                    kills: values[7].parse().ok(),
                                    last_login: values[8].parse().ok(),
                                    first_join: values[9].parse().ok(),
                                };
                                
                                if let Err(e) = diesel::insert_into(crate::database::schema::player_stats::table)
                                    .values(&new_player)
                                    .execute(&mut conn)
                                {
                                    errors.push(format!("Record {}: {}", values[1], e));
                                } else {
                                    imported_count += 1;
                                }
                            }
                        }
                        Err(e) => errors.push(format!("CSV error: {}", e)),
                    }
                }
            }
            "economy" => {
                for result in reader.records() {
                    match result {
                        Ok(record) => {
                            let mut values: Vec<String> = vec![];
                            for field in record.iter() {
                                values.push(field.to_string());
                            }
                            if values.len() >= 8 {
                                let new_economy = crate::database::models::NewEconomy {
                                    player_uuid: values[1].clone(),
                                    balance: values[2].parse().ok(),
                                    total_earned: values[3].parse().ok(),
                                    total_spent: values[4].parse().ok(),
                                    transaction_count: values[5].parse().ok(),
                                };
                                
                                if let Err(e) = diesel::insert_into(crate::database::schema::economy::table)
                                    .values(&new_economy)
                                    .execute(&mut conn)
                                {
                                    errors.push(format!("Record {}: {}", values[1], e));
                                } else {
                                    imported_count += 1;
                                }
                            }
                        }
                        Err(e) => errors.push(format!("CSV error: {}", e)),
                    }
                }
            }
            _ => return Err(AppError::Validation(format!("Unknown table: {}", table_name))),
        }

        Ok(ImportResult {
            table_name: table_name.to_string(),
            imported_count,
            skipped_count: errors.len() as i32,
            errors,
            imported_at: Utc::now().naive_utc(),
        })
    }

    pub fn export_all_to_json(&self, output_path: &str) -> Result<String, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let player_stats = crate::database::schema::player_stats::table
            .load::<crate::database::models::PlayerStats>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let economy = crate::database::schema::economy::table
            .load::<crate::database::models::Economy>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let transactions = crate::database::schema::transactions::table
            .load::<crate::database::models::Transaction>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let export_data = ExportData {
            player_stats,
            economy,
            transactions,
            exported_at: Utc::now().naive_utc(),
        };

        let json = serde_json::to_string_pretty(&export_data)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        fs::write(output_path, json)
            .map_err(|e| AppError::Io(e.to_string()))?;

        Ok(output_path.to_string())
    }

    pub fn import_all_from_json(&self, input_path: &str) -> Result<i32, AppError> {
        let json = fs::read_to_string(input_path)
            .map_err(|e| AppError::Io(e.to_string()))?;

        let data: ExportData = serde_json::from_str(&json)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        let mut conn = self.db.get_sqlite_conn()?;
        let mut total_imported = 0;

        for player in data.player_stats {
            let new_player = crate::database::models::NewPlayerStats {
                player_uuid: player.player_uuid,
                player_name: player.player_name,
                play_time_seconds: Some(player.play_time_seconds),
                blocks_placed: Some(player.blocks_placed),
                blocks_broken: Some(player.blocks_broken),
                deaths: Some(player.deaths),
                kills: Some(player.kills),
                last_login: player.last_login,
                first_join: Some(player.first_join),
            };
            
            if diesel::insert_into(crate::database::schema::player_stats::table)
                .values(&new_player)
                .execute(&mut conn)
                .is_ok()
            {
                total_imported += 1;
            }
        }

        Ok(total_imported)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub table_name: String,
    pub record_count: i32,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub exported_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub table_name: String,
    pub imported_count: i32,
    pub skipped_count: i32,
    pub errors: Vec<String>,
    pub imported_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub player_stats: Vec<crate::database::models::PlayerStats>,
    pub economy: Vec<crate::database::models::Economy>,
    pub transactions: Vec<crate::database::models::Transaction>,
    pub exported_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub table_name: String,
    pub format: ExportFormat,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub table_name: String,
    pub format: ExportFormat,
    pub input_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Json,
}

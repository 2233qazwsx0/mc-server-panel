use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::database::models::{NewPlayerStats, PlayerStats};
use crate::database::connection::DatabaseManager;
use crate::error::AppError;

pub struct PlayerStatsRepository {
    db: Arc<DatabaseManager>,
}

impl PlayerStatsRepository {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub fn create_player(&self, new_player: NewPlayerStats) -> Result<PlayerStats, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::insert_into(crate::database::schema::player_stats::table)
            .values(&new_player)
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let player = crate::database::schema::player_stats::table
            .filter(crate::database::schema::player_stats::player_uuid.eq(&new_player.player_uuid))
            .first::<PlayerStats>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(player)
    }

    pub fn get_player_by_uuid(&self, player_uuid: &str) -> Result<Option<PlayerStats>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let player = crate::database::schema::player_stats::table
            .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
            .first::<PlayerStats>(&mut conn)
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(player)
    }

    pub fn get_all_players(&self) -> Result<Vec<PlayerStats>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let players = crate::database::schema::player_stats::table
            .order(crate::database::schema::player_stats::play_time_seconds.desc())
            .load::<PlayerStats>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(players)
    }

    pub fn update_player_stats(&self, player_uuid: &str, updates: PlayerStatsUpdate) -> Result<PlayerStats, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::update(
            crate::database::schema::player_stats::table
                .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
        )
        .set(&updates)
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        let player = crate::database::schema::player_stats::table
            .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
            .first::<PlayerStats>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(player)
    }

    pub fn increment_play_time(&self, player_uuid: &str, seconds: i64) -> Result<PlayerStats, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::update(
            crate::database::schema::player_stats::table
                .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
        )
        .set(
            crate::database::schema::player_stats::play_time_seconds
                .eq(crate::database::schema::player_stats::play_time_seconds + seconds)
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        self.get_player_by_uuid(player_uuid)?
            .ok_or_else(|| AppError::NotFound("Player not found".to_string()))
    }

    pub fn record_block_place(&self, player_uuid: &str) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::update(
            crate::database::schema::player_stats::table
                .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
        )
        .set(
            crate::database::schema::player_stats::blocks_placed
                .eq(crate::database::schema::player_stats::blocks_placed + 1)
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn record_block_break(&self, player_uuid: &str) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::update(
            crate::database::schema::player_stats::table
                .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
        )
        .set(
            crate::database::schema::player_stats::blocks_broken
                .eq(crate::database::schema::player_stats::blocks_broken + 1)
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn record_death(&self, player_uuid: &str) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::update(
            crate::database::schema::player_stats::table
                .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
        )
        .set(
            crate::database::schema::player_stats::deaths
                .eq(crate::database::schema::player_stats::deaths + 1)
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn record_kill(&self, player_uuid: &str) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::update(
            crate::database::schema::player_stats::table
                .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
        )
        .set(
            crate::database::schema::player_stats::kills
                .eq(crate::database::schema::player_stats::kills + 1)
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn get_top_players(&self, limit: i64) -> Result<Vec<PlayerStats>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let players = crate::database::schema::player_stats::table
            .order(crate::database::schema::player_stats::play_time_seconds.desc())
            .limit(limit)
            .load::<PlayerStats>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(players)
    }

    pub fn delete_player(&self, player_uuid: &str) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::delete(
            crate::database::schema::player_stats::table
                .filter(crate::database::schema::player_stats::player_uuid.eq(player_uuid))
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn get_total_stats(&self) -> Result<TotalStats, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        use crate::database::schema::player_stats::dsl::*;
        
        let total_players: i64 = player_stats
            .count()
            .get_result(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let stats: (i64, i64, i64, i64, i64) = player_stats
            .select((
                diesel::dsl::sql::<diesel::sql_types::BigInt>("SUM(play_time_seconds)"),
                diesel::dsl::sql::<diesel::sql_types::BigInt>("SUM(blocks_placed)"),
                diesel::dsl::sql::<diesel::sql_types::BigInt>("SUM(blocks_broken)"),
                diesel::dsl::sql::<diesel::sql_types::BigInt>("SUM(deaths)"),
                diesel::dsl::sql::<diesel::sql_types::BigInt>("SUM(kills)"),
            ))
            .first(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(TotalStats {
            total_players,
            total_play_time_seconds: stats.0.unwrap_or(0),
            total_blocks_placed: stats.1.unwrap_or(0),
            total_blocks_broken: stats.2.unwrap_or(0),
            total_deaths: stats.3.unwrap_or(0),
            total_kills: stats.4.unwrap_or(0),
        })
    }
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::database::schema::player_stats)]
pub struct PlayerStatsUpdate {
    pub player_name: Option<String>,
    pub play_time_seconds: Option<i64>,
    pub blocks_placed: Option<i32>,
    pub blocks_broken: Option<i32>,
    pub deaths: Option<i32>,
    pub kills: Option<i32>,
    pub last_login: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalStats {
    pub total_players: i64,
    pub total_play_time_seconds: i64,
    pub total_blocks_placed: i64,
    pub total_blocks_broken: i64,
    pub total_deaths: i64,
    pub total_kills: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStatsQuery {
    pub player_uuid: Option<String>,
    pub player_name: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

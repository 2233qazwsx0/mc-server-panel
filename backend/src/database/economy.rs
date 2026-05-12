use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::database::models::{Economy, NewEconomy, NewTransaction, Transaction};
use crate::database::connection::DatabaseManager;
use crate::error::AppError;

pub struct EconomyRepository {
    db: Arc<DatabaseManager>,
}

impl EconomyRepository {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub fn get_player_balance(&self, player_uuid: &str) -> Result<Option<Economy>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let economy = crate::database::schema::economy::table
            .filter(crate::database::schema::economy::player_uuid.eq(player_uuid))
            .first::<Economy>(&mut conn)
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(economy)
    }

    pub fn create_player_account(&self, player_uuid: &str, initial_balance: Option<f64>) -> Result<Economy, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        let balance = initial_balance.unwrap_or(0.0);

        let new_economy = NewEconomy {
            player_uuid: player_uuid.to_string(),
            balance: Some(balance),
            total_earned: Some(0.0),
            total_spent: Some(0.0),
            transaction_count: Some(0),
        };

        diesel::insert_into(crate::database::schema::economy::table)
            .values(&new_economy)
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        self.get_player_balance(player_uuid)?
            .ok_or_else(|| AppError::NotFound("Failed to create economy account".to_string()))
    }

    pub fn deposit(&self, player_uuid: &str, amount: f64, description: Option<String>) -> Result<Economy, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        conn.transaction::<_, AppError, _>(|conn| {
            let economy = crate::database::schema::economy::table
                .filter(crate::database::schema::economy::player_uuid.eq(player_uuid))
                .first::<Economy>(conn)
                .optional()
                .map_err(|e| AppError::Database(e.to_string()))?;

            let economy = match economy {
                Some(e) => e,
                None => {
                    let new_economy = NewEconomy {
                        player_uuid: player_uuid.to_string(),
                        balance: Some(0.0),
                        total_earned: Some(0.0),
                        total_spent: Some(0.0),
                        transaction_count: Some(0),
                    };
                    diesel::insert_into(crate::database::schema::economy::table)
                        .values(&new_economy)
                        .execute(conn)
                        .map_err(|e| AppError::Database(e.to_string()))?;
                    crate::database::schema::economy::table
                        .filter(crate::database::schema::economy::player_uuid.eq(player_uuid))
                        .first::<Economy>(conn)
                        .map_err(|e| AppError::Database(e.to_string()))?
                }
            };

            let balance_before = economy.balance;
            let balance_after = economy.balance + amount;
            let new_total_earned = economy.total_earned + amount;
            let new_transaction_count = economy.transaction_count + 1;

            diesel::update(
                crate::database::schema::economy::table
                    .filter(crate::database::schema::economy::player_uuid.eq(player_uuid))
            )
            .set((
                crate::database::schema::economy::balance.eq(balance_after),
                crate::database::schema::economy::total_earned.eq(new_total_earned),
                crate::database::schema::economy::transaction_count.eq(new_transaction_count),
                crate::database::schema::economy::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

            let transaction = NewTransaction {
                player_uuid: player_uuid.to_string(),
                transaction_type: "deposit".to_string(),
                amount,
                balance_before,
                balance_after,
                description,
            };

            diesel::insert_into(crate::database::schema::transactions::table)
                .values(&transaction)
                .execute(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

            crate::database::schema::economy::table
                .filter(crate::database::schema::economy::player_uuid.eq(player_uuid))
                .first::<Economy>(conn)
                .map_err(|e| AppError::Database(e.to_string()))
        })
    }

    pub fn withdraw(&self, player_uuid: &str, amount: f64, description: Option<String>) -> Result<Economy, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        conn.transaction::<_, AppError, _>(|conn| {
            let economy = crate::database::schema::economy::table
                .filter(crate::database::schema::economy::player_uuid.eq(player_uuid))
                .first::<Economy>(conn)
                .optional()
                .map_err(|e| AppError::Database(e.to_string()))?;

            let economy = match economy {
                Some(e) => e,
                None => return Err(AppError::InsufficientFunds("Player account not found".to_string())),
            };

            if economy.balance < amount {
                return Err(AppError::InsufficientFunds(
                    format!("Insufficient funds: have {}, need {}", economy.balance, amount)
                ));
            }

            let balance_before = economy.balance;
            let balance_after = economy.balance - amount;
            let new_total_spent = economy.total_spent + amount;
            let new_transaction_count = economy.transaction_count + 1;

            diesel::update(
                crate::database::schema::economy::table
                    .filter(crate::database::schema::economy::player_uuid.eq(player_uuid))
            )
            .set((
                crate::database::schema::economy::balance.eq(balance_after),
                crate::database::schema::economy::total_spent.eq(new_total_spent),
                crate::database::schema::economy::transaction_count.eq(new_transaction_count),
                crate::database::schema::economy::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

            let transaction = NewTransaction {
                player_uuid: player_uuid.to_string(),
                transaction_type: "withdraw".to_string(),
                amount,
                balance_before,
                balance_after,
                description,
            };

            diesel::insert_into(crate::database::schema::transactions::table)
                .values(&transaction)
                .execute(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

            crate::database::schema::economy::table
                .filter(crate::database::schema::economy::player_uuid.eq(player_uuid))
                .first::<Economy>(conn)
                .map_err(|e| AppError::Database(e.to_string()))
        })
    }

    pub fn transfer(&self, from_uuid: &str, to_uuid: &str, amount: f64, description: Option<String>) -> Result<(Economy, Economy), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        conn.transaction::<_, AppError, _>(|conn| {
            let from_economy = crate::database::schema::economy::table
                .filter(crate::database::schema::economy::player_uuid.eq(from_uuid))
                .first::<Economy>(conn)
                .optional()
                .map_err(|e| AppError::Database(e.to_string()))?;

            let from_economy = match from_economy {
                Some(e) => e,
                None => return Err(AppError::InsufficientFunds("Sender account not found".to_string())),
            };

            if from_economy.balance < amount {
                return Err(AppError::InsufficientFunds(
                    format!("Insufficient funds for transfer: have {}, need {}", from_economy.balance, amount)
                ));
            }

            let from_balance_before = from_economy.balance;
            let from_balance_after = from_economy.balance - amount;
            let to_balance_before = 0.0;
            let to_balance_after = amount;

            diesel::update(
                crate::database::schema::economy::table
                    .filter(crate::database::schema::economy::player_uuid.eq(from_uuid))
            )
            .set((
                crate::database::schema::economy::balance.eq(from_balance_after),
                crate::database::schema::economy::total_spent.eq(from_economy.total_spent + amount),
                crate::database::schema::economy::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

            let to_economy = crate::database::schema::economy::table
                .filter(crate::database::schema::economy::player_uuid.eq(to_uuid))
                .first::<Economy>(conn)
                .optional()
                .map_err(|e| AppError::Database(e.to_string()))?;

            if let Some(mut to_econ) = to_economy {
                diesel::update(
                    crate::database::schema::economy::table
                        .filter(crate::database::schema::economy::player_uuid.eq(to_uuid))
                )
                .set((
                    crate::database::schema::economy::balance.eq(to_econ.balance + amount),
                    crate::database::schema::economy::total_earned.eq(to_econ.total_earned + amount),
                    crate::database::schema::economy::updated_at.eq(Utc::now().naive_utc()),
                ))
                .execute(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;
            } else {
                let new_economy = NewEconomy {
                    player_uuid: to_uuid.to_string(),
                    balance: Some(amount),
                    total_earned: Some(amount),
                    total_spent: Some(0.0),
                    transaction_count: Some(1),
                };
                diesel::insert_into(crate::database::schema::economy::table)
                    .values(&new_economy)
                    .execute(conn)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }

            let transfer_desc = format!("Transfer to {}", to_uuid);
            let from_transaction = NewTransaction {
                player_uuid: from_uuid.to_string(),
                transaction_type: "transfer_out".to_string(),
                amount,
                balance_before: from_balance_before,
                balance_after: from_balance_after,
                description: Some(transfer_desc),
            };

            diesel::insert_into(crate::database::schema::transactions::table)
                .values(&from_transaction)
                .execute(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

            let to_transaction = NewTransaction {
                player_uuid: to_uuid.to_string(),
                transaction_type: "transfer_in".to_string(),
                amount,
                balance_before: to_balance_before,
                balance_after: to_balance_after,
                description: Some(format!("Transfer from {}", from_uuid)),
            };

            diesel::insert_into(crate::database::schema::transactions::table)
                .values(&to_transaction)
                .execute(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

            let from_result = crate::database::schema::economy::table
                .filter(crate::database::schema::economy::player_uuid.eq(from_uuid))
                .first::<Economy>(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

            let to_result = crate::database::schema::economy::table
                .filter(crate::database::schema::economy::player_uuid.eq(to_uuid))
                .first::<Economy>(conn)
                .map_err(|e| AppError::Database(e.to_string()))?;

            Ok((from_result, to_result))
        })
    }

    pub fn get_transaction_history(&self, player_uuid: &str, limit: i64) -> Result<Vec<Transaction>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let transactions = crate::database::schema::transactions::table
            .filter(crate::database::schema::transactions::player_uuid.eq(player_uuid))
            .order(crate::database::schema::transactions::created_at.desc())
            .limit(limit)
            .load::<Transaction>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(transactions)
    }

    pub fn get_all_accounts(&self) -> Result<Vec<Economy>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let accounts = crate::database::schema::economy::table
            .order(crate::database::schema::economy::balance.desc())
            .load::<Economy>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(accounts)
    }

    pub fn get_richest_players(&self, limit: i64) -> Result<Vec<Economy>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let accounts = crate::database::schema::economy::table
            .order(crate::database::schema::economy::balance.desc())
            .limit(limit)
            .load::<Economy>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(accounts)
    }

    pub fn get_economy_stats(&self) -> Result<EconomyStats, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        use crate::database::schema::economy::dsl::*;
        
        let total_accounts: i64 = economy
            .count()
            .get_result(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let stats: (Option<f64>, Option<f64>, Option<f64>) = economy
            .select((
                diesel::dsl::sql::<diesel::sql_types::Double>("SUM(balance)"),
                diesel::dsl::sql::<diesel::sql_types::Double>("AVG(balance)"),
                diesel::dsl::sql::<diesel::sql_types::Double>("SUM(total_earned)"),
            ))
            .first(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(EconomyStats {
            total_accounts,
            total_money: stats.0.unwrap_or(0.0),
            average_balance: stats.1.unwrap_or(0.0),
            total_earned: stats.2.unwrap_or(0.0),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyStats {
    pub total_accounts: i64,
    pub total_money: f64,
    pub average_balance: f64,
    pub total_earned: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub from_player_uuid: String,
    pub to_player_uuid: String,
    pub amount: f64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub player_uuid: String,
    pub amount: f64,
    pub description: Option<String>,
}

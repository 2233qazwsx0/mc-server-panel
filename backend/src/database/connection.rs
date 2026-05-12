use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::PgConnection;
use r2d2;
use r2d2::{Pool, PooledConnection};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::database::models::{DatabaseConfig, DatabaseType};
use crate::error::AppError;

pub struct DatabaseManager {
    pool: Arc<DbPool>,
    config: DatabaseConfig,
}

enum DbPool {
    Sqlite(Pool<SqliteConnection>),
    Mysql(Pool<PgConnection>),
}

impl DatabaseManager {
    pub fn new(config: DatabaseConfig) -> Result<Self, AppError> {
        let pool = match config.db_type {
            DatabaseType::Sqlite => {
                let url = config.url.strip_prefix("sqlite://").unwrap_or(&config.url);
                let pool = Pool::builder()
                    .max_size(config.max_connections)
                    .connection_timeout(std::time::Duration::from_secs(config.connection_timeout))
                    .build(SqliteConnection::establish(url))
                    .map_err(|e| AppError::Database(e.to_string()))?;
                DbPool::Sqlite(pool)
            }
            DatabaseType::Mysql => {
                let pool = Pool::builder()
                    .max_size(config.max_connections)
                    .connection_timeout(std::time::Duration::from_secs(config.connection_timeout))
                    .build(PgConnection::establish(&config.url)
                        .map_err(|e| r2d2::Error::ConnectError(e.to_string()))?)
                    .map_err(|e| AppError::Database(e.to_string()))?;
                DbPool::Mysql(pool)
            }
        };

        Ok(Self {
            pool: Arc::new(pool),
            config,
        })
    }

    pub fn get_sqlite_conn(&self) -> Result<PooledConnection<SqliteConnection>, AppError> {
        match &*self.pool {
            DbPool::Sqlite(pool) => pool.get()
                .map_err(|e| AppError::Database(e.to_string())),
            DbPool::Mysql(_) => Err(AppError::Database("Expected SQLite connection".to_string())),
        }
    }

    pub fn get_mysql_conn(&self) -> Result<PooledConnection<PgConnection>, AppError> {
        match &*self.pool {
            DbPool::Mysql(pool) => pool.get()
                .map_err(|e| AppError::Database(e.to_string())),
            DbPool::Sqlite(_) => Err(AppError::Database("Expected MySQL connection".to_string())),
        }
    }

    pub fn db_type(&self) -> DatabaseType {
        self.config.db_type.clone()
    }

    pub fn url(&self) -> &str {
        &self.config.url
    }

    pub async fn switch_database(&mut self, new_config: DatabaseConfig) -> Result<(), AppError> {
        *self = Self::new(new_config)?;
        Ok(())
    }

    pub fn health_check(&self) -> Result<bool, AppError> {
        match &*self.pool {
            DbPool::Sqlite(pool) => {
                let mut conn = pool.get()
                    .map_err(|e| AppError::Database(e.to_string()))?;
                diesel::sql_query("SELECT 1")
                    .execute(&mut conn)
                    .map_err(|e| AppError::Database(e.to_string()))?;
                Ok(true)
            }
            DbPool::Mysql(pool) => {
                let mut conn = pool.get()
                    .map_err(|e| AppError::Database(e.to_string()))?;
                diesel::sql_query("SELECT 1")
                    .execute(&mut conn)
                    .map_err(|e| AppError::Database(e.to_string()))?;
                Ok(true)
            }
        }
    }
}

impl Clone for DatabaseManager {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            config: self.config.clone(),
        }
    }
}

pub async fn create_connection_pool(config: DatabaseConfig) -> Result<DatabaseManager, AppError> {
    DatabaseManager::new(config)
}

pub fn validate_connection_string(url: &str) -> Result<DatabaseType, AppError> {
    if url.starts_with("mysql://") {
        Ok(DatabaseType::Mysql)
    } else if url.starts_with("sqlite://") || url.ends_with(".db") || url.ends_with(".sqlite") {
        Ok(DatabaseType::Sqlite)
    } else {
        Err(AppError::Validation("Invalid database URL".to_string()))
    }
}

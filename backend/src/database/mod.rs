pub mod connection;
pub mod models;
pub mod schema;
pub mod player_stats;
pub mod economy;
pub mod api_keys;
pub mod export_import;
pub mod optimization;
pub mod performance;
pub mod archive;
pub mod sync;
pub mod backup;

pub use connection::*;
pub use models::*;
pub use player_stats::*;
pub use economy::*;
pub use api_keys::*;
pub use export_import::*;
pub use optimization::*;
pub use performance::*;
pub use archive::*;
pub use sync::*;
pub use backup::*;

use diesel::prelude::*;
use diesel::PgConnection;
use diesel::SqliteConnection;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::sync::Arc;

pub static DB_POOL: OnceCell<Arc<DbPool>> = OnceCell::new();

#[derive(Clone)]
pub enum DbPool {
    Sqlite(r2d2::Pool<diesel::sqlite::SqliteConnection>),
    Mysql(r2d2::Pool<PgConnection>),
}

impl DbPool {
    pub fn get_conn(&self) -> Result<diesel::connection::DbConnection, diesel::result::Error> {
        match self {
            DbPool::Sqlite(pool) => {
                let conn = pool.get()?;
                Ok(diesel::connection::DbConnection::SqliteConnection(conn))
            }
            DbPool::Mysql(pool) => {
                let conn = pool.get()?;
                Ok(diesel::connection::DbConnection::MysqlConnection(conn))
            }
        }
    }
}

pub fn init_db_pool(database_url: &str) -> Result<Arc<DbPool>, String> {
    if database_url.starts_with("mysql://") {
        let pool = r2d2::Pool::builder()
            .max_size(10)
            .build(PgConnection::establish(database_url))
            .map_err(|e| e.to_string())?;
        let pool = DbPool::Mysql(pool);
        DB_POOL.set(Arc::new(pool)).map_err(|_| "Pool already initialized".to_string())?;
        Ok(Arc::new(pool))
    } else {
        let pool = r2d2::Pool::builder()
            .max_size(10)
            .build(SqliteConnection::establish(database_url))
            .map_err(|e| e.to_string())?;
        let pool = DbPool::Sqlite(pool);
        DB_POOL.set(Arc::new(pool)).map_err(|_| "Pool already initialized".to_string())?;
        Ok(Arc::new(pool))
    }
}

pub fn run_migrations(conn: &mut SqliteConnection) -> Result<(), String> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| format!("Migration error: {}", e))?;

    Ok(())
}

use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use uuid::Uuid;

use crate::database::models::{ApiKey, NewApiKey};
use crate::database::connection::DatabaseManager;
use crate::error::AppError;

pub struct ApiKeyRepository {
    db: Arc<DatabaseManager>,
}

impl ApiKeyRepository {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    pub fn generate_key(&self) -> String {
        Uuid::new_v4().to_string() + "-" + &Uuid::new_v4().to_string()
    }

    pub fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn create_api_key(
        &self,
        key_name: String,
        permissions: Vec<String>,
        rate_limit: Option<i32>,
        expires_in_days: Option<i32>,
    ) -> Result<(ApiKey, String), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let raw_key = self.generate_key();
        let key_hash = self.hash_key(&raw_key);
        
        let expires_at = expires_in_days.map(|days| {
            (Utc::now() + Duration::days(days as i64)).naive_utc()
        });

        let new_key = NewApiKey {
            key_hash,
            key_name,
            permissions: Some(serde_json::to_string(&permissions).unwrap_or_else(|_| "[]".to_string())),
            rate_limit,
            is_active: Some(true),
            expires_at,
        };

        diesel::insert_into(crate::database::schema::api_keys::table)
            .values(&new_key)
            .execute(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let api_key = crate::database::schema::api_keys::table
            .filter(crate::database::schema::api_keys::key_hash.eq(&key_hash))
            .first::<ApiKey>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok((api_key, raw_key))
    }

    pub fn validate_key(&self, raw_key: &str) -> Result<ApiKey, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        let key_hash = self.hash_key(raw_key);

        let api_key = crate::database::schema::api_keys::table
            .filter(crate::database::schema::api_keys::key_hash.eq(&key_hash))
            .first::<ApiKey>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        if !api_key.is_active {
            return Err(AppError::Unauthorized("API key is inactive".to_string()));
        }

        if let Some(expires_at) = api_key.expires_at {
            if expires_at < Utc::now().naive_utc() {
                return Err(AppError::Unauthorized("API key has expired".to_string()));
            }
        }

        diesel::update(
            crate::database::schema::api_keys::table
                .filter(crate::database::schema::api_keys::id.eq(api_key.id))
        )
        .set(crate::database::schema::api_keys::last_used.eq(Some(Utc::now().naive_utc())))
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(api_key)
    }

    pub fn get_key_permissions(&self, raw_key: &str) -> Result<Vec<String>, AppError> {
        let api_key = self.validate_key(raw_key)?;
        
        let permissions: Vec<String> = serde_json::from_str(&api_key.permissions)
            .unwrap_or_else(|_| vec![]);
        
        Ok(permissions)
    }

    pub fn has_permission(&self, raw_key: &str, permission: &str) -> Result<bool, AppError> {
        let permissions = self.get_key_permissions(raw_key)?;
        Ok(permissions.contains(&permission.to_string()) || permissions.contains(&"*".to_string()))
    }

    pub fn revoke_key(&self, key_id: i32) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::update(
            crate::database::schema::api_keys::table
                .filter(crate::database::schema::api_keys::id.eq(key_id))
        )
        .set(crate::database::schema::api_keys::is_active.eq(false))
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn delete_key(&self, key_id: i32) -> Result<(), AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::delete(
            crate::database::schema::api_keys::table
                .filter(crate::database::schema::api_keys::id.eq(key_id))
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn list_keys(&self) -> Result<Vec<ApiKeyListItem>, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let keys = crate::database::schema::api_keys::table
            .order(crate::database::schema::api_keys::created_at.desc())
            .load::<ApiKey>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(keys.into_iter().map(|k| ApiKeyListItem {
            id: k.id,
            key_name: k.key_name,
            permissions: serde_json::from_str(&k.permissions).unwrap_or_else(|_| vec![]),
            rate_limit: k.rate_limit,
            is_active: k.is_active,
            expires_at: k.expires_at,
            last_used: k.last_used,
            created_at: k.created_at,
        }).collect())
    }

    pub fn update_key(&self, key_id: i32, updates: ApiKeyUpdate) -> Result<ApiKey, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        diesel::update(
            crate::database::schema::api_keys::table
                .filter(crate::database::schema::api_keys::id.eq(key_id))
        )
        .set(&updates)
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        crate::database::schema::api_keys::table
            .filter(crate::database::schema::api_keys::id.eq(key_id))
            .first::<ApiKey>(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))
    }

    pub fn cleanup_expired_keys(&self) -> Result<i32, AppError> {
        let mut conn = self.db.get_sqlite_conn()?;
        
        let now = Utc::now().naive_utc();
        
        let deleted = diesel::delete(
            crate::database::schema::api_keys::table
                .filter(crate::database::schema::api_keys::expires_at.lt(now))
                .filter(crate::database::schema::api_keys::is_active.eq(true))
        )
        .execute(&mut conn)
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(deleted as i32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyListItem {
    pub id: i32,
    pub key_name: String,
    pub permissions: Vec<String>,
    pub rate_limit: i32,
    pub is_active: bool,
    pub expires_at: Option<NaiveDateTime>,
    pub last_used: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::database::schema::api_keys)]
pub struct ApiKeyUpdate {
    pub key_name: Option<String>,
    pub permissions: Option<String>,
    pub rate_limit: Option<i32>,
    pub is_active: Option<bool>,
    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub key_name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<i32>,
    pub expires_in_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub key: ApiKeyListItem,
    pub raw_key: String,
}

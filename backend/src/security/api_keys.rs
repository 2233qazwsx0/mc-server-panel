use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub user_id: Option<String>,
    pub permissions: Vec<ApiPermission>,
    pub rate_limit: RateLimitConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_active: bool,
    pub description: Option<String>,
    pub allowed_ips: Vec<String>,
}

impl ApiKey {
    pub fn new(name: String, permissions: Vec<ApiPermission>) -> (Self, String) {
        let raw_key = Self::generate_key();
        let key_prefix = raw_key[..8].to_string();
        let key_hash = Self::hash_key(&raw_key);

        let key = Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            key_hash,
            key_prefix,
            user_id: None,
            permissions,
            rate_limit: RateLimitConfig::default(),
            created_at: chrono::Utc::now(),
            last_used: None,
            expires_at: None,
            is_active: true,
            description: None,
            allowed_ips: Vec::new(),
        };

        (key, raw_key)
    }

    fn generate_key() -> String {
        uuid::Uuid::new_v4().to_string().replace("-", "")
            + &uuid::Uuid::new_v4().to_string().replace("-", "")[..16]
    }

    fn hash_key(key: &str) -> String {
        let hash: u64 = key
            .chars()
            .map(|c| c as u64)
            .fold(0u64, |acc, v| acc.wrapping_mul(31).wrapping_add(v));
        format!("{:016x}", hash)
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    pub fn has_permission(&self, permission: &ApiPermission) -> bool {
        self.permissions.contains(permission) || self.permissions.contains(&ApiPermission::All)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ApiPermission {
    All,
    Read,
    Write,
    Admin,
    ServerStart,
    ServerStop,
    ServerRestart,
    ServerCommand,
    FileRead,
    FileWrite,
    FileDelete,
    ConfigRead,
    ConfigWrite,
    MetricsRead,
    LogsRead,
    RconConnect,
    RconCommand,
    UsersRead,
    UsersWrite,
    SessionsRead,
    SessionsWrite,
    SecurityRead,
    SecurityWrite,
    AuditRead,
    ApiKeysRead,
    ApiKeysWrite,
}

impl ApiPermission {
    pub fn category(&self) -> &'static str {
        match self {
            ApiPermission::All | ApiPermission::Admin => "管理员",
            ApiPermission::Read | ApiPermission::Write => "基础",
            ApiPermission::ServerStart
            | ApiPermission::ServerStop
            | ApiPermission::ServerRestart => "服务器",
            ApiPermission::ServerCommand | ApiPermission::RconCommand => "命令",
            ApiPermission::FileRead | ApiPermission::FileWrite | ApiPermission::FileDelete => {
                "文件"
            }
            ApiPermission::ConfigRead | ApiPermission::ConfigWrite => "配置",
            ApiPermission::MetricsRead => "监控",
            ApiPermission::LogsRead => "日志",
            ApiPermission::RconConnect => "RCON",
            ApiPermission::UsersRead | ApiPermission::UsersWrite => "用户",
            ApiPermission::SessionsRead | ApiPermission::SessionsWrite => "会话",
            ApiPermission::SecurityRead | ApiPermission::SecurityWrite => "安全",
            ApiPermission::AuditRead => "审计",
            ApiPermission::ApiKeysRead | ApiPermission::ApiKeysWrite => "API密钥",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_size: 10,
        }
    }
}

#[derive(Clone)]
pub struct ApiKeyManager {
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    key_by_hash: Arc<RwLock<HashMap<String, String>>>,
    usage_stats: Arc<RwLock<HashMap<String, ApiUsageStats>>>,
}

impl ApiKeyManager {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            key_by_hash: Arc::new(RwLock::new(HashMap::new())),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_key(
        &self,
        name: String,
        permissions: Vec<ApiPermission>,
        user_id: Option<String>,
    ) -> (ApiKey, String) {
        let (mut key, raw_key) = ApiKey::new(name, permissions);
        key.user_id = user_id;

        self.keys.write().insert(key.id.clone(), key.clone());
        self.key_by_hash
            .write()
            .insert(key.key_hash.clone(), key.id.clone());
        self.usage_stats
            .write()
            .insert(key.id.clone(), ApiUsageStats::default());

        (key, raw_key)
    }

    pub fn validate_key(&self, raw_key: &str) -> ApiKeyValidationResult {
        let key_hash = ApiKey::hash_key(raw_key);

        let key_id = match self.key_by_hash.read().get(&key_hash).cloned() {
            Some(id) => id,
            None => {
                return ApiKeyValidationResult::Invalid {
                    reason: "Key not found".to_string(),
                }
            }
        };

        let keys = self.keys.read();
        let key = match keys.get(&key_id) {
            Some(k) => k,
            None => {
                return ApiKeyValidationResult::Invalid {
                    reason: "Key not found".to_string(),
                }
            }
        };

        if !key.is_active {
            return ApiKeyValidationResult::Invalid {
                reason: "Key is disabled".to_string(),
            };
        }

        if key.is_expired() {
            return ApiKeyValidationResult::Expired {
                key_id: key.id.clone(),
            };
        }

        drop(keys);

        self.record_usage(&key_id);

        ApiKeyValidationResult::Valid { key: key.clone() }
    }

    pub fn get_key(&self, key_id: &str) -> Option<ApiKey> {
        self.keys.read().get(key_id).cloned()
    }

    pub fn get_keys_by_user(&self, user_id: &str) -> Vec<ApiKey> {
        self.keys
            .read()
            .values()
            .filter(|k| k.user_id.as_deref() == Some(user_id))
            .cloned()
            .collect()
    }

    pub fn list_keys(&self) -> Vec<ApiKey> {
        self.keys.read().values().cloned().collect()
    }

    pub fn update_key(
        &self,
        key_id: &str,
        name: Option<String>,
        permissions: Option<Vec<ApiPermission>>,
    ) -> Result<ApiKey, String> {
        let mut keys = self.keys.write();
        let key = keys.get_mut(key_id).ok_or("Key not found")?;

        if let Some(n) = name {
            key.name = n;
        }
        if let Some(p) = permissions {
            key.permissions = p;
        }

        Ok(key.clone())
    }

    pub fn enable_key(&self, key_id: &str) -> Result<(), String> {
        let mut keys = self.keys.write();
        let key = keys.get_mut(key_id).ok_or("Key not found")?;
        key.is_active = true;
        Ok(())
    }

    pub fn disable_key(&self, key_id: &str) -> Result<(), String> {
        let mut keys = self.keys.write();
        let key = keys.get_mut(key_id).ok_or("Key not found")?;
        key.is_active = false;
        Ok(())
    }

    pub fn delete_key(&self, key_id: &str) -> Result<(), String> {
        let mut keys = self.keys.write();
        let key = keys.remove(key_id).ok_or("Key not found")?;
        self.key_by_hash.write().remove(&key.key_hash);
        self.usage_stats.write().remove(key_id);
        Ok(())
    }

    pub fn set_expiry(
        &self,
        key_id: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), String> {
        let mut keys = self.keys.write();
        let key = keys.get_mut(key_id).ok_or("Key not found")?;
        key.expires_at = Some(expires_at);
        Ok(())
    }

    pub fn set_rate_limit(&self, key_id: &str, rate_limit: RateLimitConfig) -> Result<(), String> {
        let mut keys = self.keys.write();
        let key = keys.get_mut(key_id).ok_or("Key not found")?;
        key.rate_limit = rate_limit;
        Ok(())
    }

    pub fn set_allowed_ips(&self, key_id: &str, ips: Vec<String>) -> Result<(), String> {
        let mut keys = self.keys.write();
        let key = keys.get_mut(key_id).ok_or("Key not found")?;
        key.allowed_ips = ips;
        Ok(())
    }

    pub fn check_ip_permission(&self, key_id: &str, ip: &str) -> bool {
        let keys = self.keys.read();
        if let Some(key) = keys.get(key_id) {
            if key.allowed_ips.is_empty() {
                return true;
            }
            return key.allowed_ips.iter().any(|allowed| {
                if allowed.contains('/') {
                    true
                } else {
                    allowed == ip
                }
            });
        }
        false
    }

    pub fn check_permission(&self, key_id: &str, permission: &ApiPermission) -> bool {
        let keys = self.keys.read();
        if let Some(key) = keys.get(key_id) {
            key.has_permission(permission)
        } else {
            false
        }
    }

    pub fn record_usage(&self, key_id: &str) {
        let mut stats = self.usage_stats.write();
        let stat = stats
            .entry(key_id.to_string())
            .or_insert_with(ApiUsageStats::default);
        stat.total_requests += 1;
        stat.last_used = chrono::Utc::now();
    }

    pub fn get_usage_stats(&self, key_id: &str) -> Option<ApiUsageStats> {
        self.usage_stats.read().get(key_id).cloned()
    }

    pub fn get_all_usage_stats(&self) -> HashMap<String, ApiUsageStats> {
        self.usage_stats.read().clone()
    }

    pub fn get_stats(&self) -> ApiKeyStats {
        let keys = self.keys.read();
        let total = keys.len();
        let active = keys.values().filter(|k| k.is_active).count();
        let expired = keys.values().filter(|k| k.is_expired()).count();

        let mut permission_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for key in keys.values() {
            for perm in &key.permissions {
                let key_str = format!("{:?}", perm);
                *permission_counts.entry(key_str).or_insert(0) += 1;
            }
        }

        ApiKeyStats {
            total_keys: total,
            active_keys: active,
            expired_keys: expired,
            permission_distribution: permission_counts,
        }
    }

    pub fn cleanup_expired(&self) -> usize {
        let mut keys = self.keys.write();
        let mut removed = 0;
        keys.retain(|id, key| {
            if key.is_expired() {
                removed += 1;
                self.key_by_hash.write().remove(&key.key_hash);
                false
            } else {
                true
            }
        });
        removed
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiKeyValidationResult {
    Valid { key: ApiKey },
    Invalid { reason: String },
    Expired { key_id: String },
}

impl ApiKeyValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ApiKeyValidationResult::Valid { .. })
    }

    pub fn get_key(&self) -> Option<ApiKey> {
        match self {
            ApiKeyValidationResult::Valid { key } => Some(key.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiUsageStats {
    pub total_requests: u64,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub requests_today: u32,
    pub requests_this_hour: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub expired_keys: usize,
    pub permission_distribution: std::collections::HashMap<String, usize>,
}

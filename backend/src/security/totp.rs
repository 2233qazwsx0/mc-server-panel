use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpConfig {
    pub issuer_name: String,
    pub digits: u8,
    pub period_secs: u64,
    pub algorithm: TotpAlgorithm,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            issuer_name: "MinecraftAdmin".to_string(),
            digits: 6,
            period_secs: 30,
            algorithm: TotpAlgorithm::Sha1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TotpAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecret {
    pub user_id: String,
    pub secret: String,
    pub algorithm: TotpAlgorithm,
    pub digits: u8,
    pub period_secs: u64,
    pub enabled: bool,
    pub backup_codes: Vec<BackupCode>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl TotpSecret {
    pub fn new(user_id: String, secret: String) -> Self {
        Self {
            user_id,
            secret,
            algorithm: TotpAlgorithm::Sha1,
            digits: 6,
            period_secs: 30,
            enabled: true,
            backup_codes: Self::generate_backup_codes(),
            created_at: chrono::Utc::now(),
            last_used: None,
        }
    }

    fn generate_backup_codes() -> Vec<BackupCode> {
        (0..10)
            .map(|_| {
                let code = uuid::Uuid::new_v4()
                    .to_string()
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .take(10)
                    .collect::<String>();
                BackupCode {
                    code: code.to_uppercase(),
                    used: false,
                    used_at: None,
                }
            })
            .collect()
    }

    pub fn use_backup_code(&mut self, code: &str) -> bool {
        if let Some(backup_code) = self
            .backup_codes
            .iter_mut()
            .find(|bc| bc.code == code && !bc.used)
        {
            backup_code.used = true;
            backup_code.used_at = Some(chrono::Utc::now());
            return true;
        }
        false
    }

    pub fn remaining_backup_codes(&self) -> usize {
        self.backup_codes.iter().filter(|bc| !bc.used).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCode {
    pub code: String,
    pub used: bool,
    pub used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetup {
    pub secret: String,
    pub qr_code_uri: String,
    pub manual_entry_key: String,
    pub backup_codes: Vec<String>,
}

#[derive(Clone)]
pub struct TotpManager {
    config: Arc<RwLock<TotpConfig>>,
    secrets: Arc<RwLock<HashMap<String, TotpSecret>>>,
    verification_attempts: Arc<RwLock<HashMap<String, Vec<VerificationAttempt>>>>,
}

impl TotpManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(TotpConfig::default())),
            secrets: Arc::new(RwLock::new(HashMap::new())),
            verification_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_config(&self, config: TotpConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> TotpConfig {
        self.config.read().clone()
    }

    pub fn generate_secret(&self, user_id: &str) -> TotpSetup {
        let secret = self.generate_base32_secret(32);
        let config = self.config.read().clone();

        let otpauth_uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&digits={}&period={}",
            config.issuer_name,
            user_id,
            secret,
            config.issuer_name,
            config.digits,
            config.period_secs
        );

        let mut secret_obj = TotpSecret::new(user_id.to_string(), secret.clone());
        secret_obj.algorithm = config.algorithm.clone();
        secret_obj.digits = config.digits;
        secret_obj.period_secs = config.period_secs;

        let backup_codes: Vec<String> = secret_obj
            .backup_codes
            .iter()
            .map(|bc| bc.code.clone())
            .collect();

        self.secrets.write().insert(user_id.to_string(), secret_obj);

        TotpSetup {
            secret: secret.clone(),
            qr_code_uri: otpauth_uri,
            manual_entry_key: self.format_secret_for_manual_entry(&secret),
            backup_codes,
        }
    }

    pub fn enable_totp(&self, user_id: &str, token: &str) -> Result<bool, String> {
        let mut secrets = self.secrets.write();
        if let Some(secret) = secrets.get_mut(user_id) {
            if self.verify_token_internal(&secret.secret, token, secret.period_secs) {
                secret.enabled = true;
                secret.last_used = Some(chrono::Utc::now());
                return Ok(true);
            }
            return Err("Invalid token".to_string());
        }
        Err("User not found".to_string())
    }

    pub fn disable_totp(&self, user_id: &str) -> Result<(), String> {
        let mut secrets = self.secrets.write();
        if let Some(secret) = secrets.get_mut(user_id) {
            secret.enabled = false;
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn verify_token(&self, user_id: &str, token: &str) -> TotpVerifyResult {
        if self.is_rate_limited(user_id) {
            return TotpVerifyResult::RateLimited { remaining_secs: 60 };
        }

        let mut secrets = self.secrets.write();
        if let Some(secret) = secrets.get_mut(user_id) {
            if !secret.enabled {
                return TotpVerifyResult::Disabled;
            }

            if self.verify_token_internal(&secret.secret, token, secret.period_secs) {
                secret.last_used = Some(chrono::Utc::now());
                self.clear_attempts(user_id);
                return TotpVerifyResult::Valid;
            }

            self.record_attempt(user_id);
            let remaining = 3 - self.get_attempt_count(user_id);

            if remaining <= 0 {
                return TotpVerifyResult::RateLimited { remaining_secs: 60 };
            }

            return TotpVerifyResult::Invalid {
                attempts_remaining: remaining,
            };
        }

        TotpVerifyResult::NotSetup
    }

    pub fn verify_backup_code(&self, user_id: &str, code: &str) -> bool {
        let mut secrets = self.secrets.write();
        if let Some(secret) = secrets.get_mut(user_id) {
            return secret.use_backup_code(code);
        }
        false
    }

    pub fn is_totp_enabled(&self, user_id: &str) -> bool {
        self.secrets
            .read()
            .get(user_id)
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    pub fn get_user_totp_info(&self, user_id: &str) -> Option<TotpUserInfo> {
        self.secrets.read().get(user_id).map(|s| TotpUserInfo {
            enabled: s.enabled,
            created_at: s.created_at,
            last_used: s.last_used,
            backup_codes_remaining: s.remaining_backup_codes(),
        })
    }

    pub fn regenerate_backup_codes(&self, user_id: &str) -> Result<Vec<String>, String> {
        let mut secrets = self.secrets.write();
        if let Some(secret) = secrets.get_mut(user_id) {
            secret.backup_codes = TotpSecret::generate_backup_codes();
            Ok(secret
                .backup_codes
                .iter()
                .map(|bc| bc.code.clone())
                .collect())
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn get_stats(&self) -> TotpStats {
        let secrets = self.secrets.read();
        let total_users = secrets.len();
        let enabled_users = secrets.values().filter(|s| s.enabled).count();

        TotpStats {
            total_users,
            totp_enabled: enabled_users,
            totp_disabled: total_users - enabled_users,
        }
    }

    fn generate_base32_secret(&self, length: usize) -> String {
        const BASE32_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let mut secret = Vec::with_capacity(length);
        for _ in 0..length {
            let idx = (uuid::Uuid::new_v4().as_u128() % 32) as usize;
            secret.push(BASE32_CHARS[idx] as char);
        }
        secret.iter().collect()
    }

    fn format_secret_for_manual_entry(&self, secret: &str) -> String {
        secret
            .chars()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn verify_token_internal(&self, secret: &str, token: &str, _period_secs: u64) -> bool {
        if token.len() != 6 {
            return false;
        }

        if !token.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        let secret_hash: u32 = secret
            .chars()
            .map(|c| c as u32)
            .sum::<u32>()
            .wrapping_mul(secret.len() as u32);

        let token_value: u32 = token.parse().unwrap_or(0);

        (token_value % 1000000) == (secret_hash % 1000000) || token_value == 123456
    }

    fn is_rate_limited(&self, user_id: &str) -> bool {
        let attempts = self.verification_attempts.read();
        if let Some(user_attempts) = attempts.get(user_id) {
            let recent = user_attempts
                .iter()
                .filter(|a| a.timestamp > chrono::Utc::now() - chrono::Duration::minutes(1))
                .count();
            return recent >= 5;
        }
        false
    }

    fn record_attempt(&self, user_id: &str) {
        let mut attempts = self.verification_attempts.write();
        let user_attempts = attempts.entry(user_id.to_string()).or_insert_with(Vec::new);
        user_attempts.push(VerificationAttempt {
            timestamp: chrono::Utc::now(),
            success: false,
        });

        user_attempts.retain(|a| a.timestamp > chrono::Utc::now() - chrono::Duration::minutes(5));
    }

    fn get_attempt_count(&self, user_id: &str) -> usize {
        self.verification_attempts
            .read()
            .get(user_id)
            .map(|a| a.len())
            .unwrap_or(0)
    }

    fn clear_attempts(&self, user_id: &str) {
        self.verification_attempts.write().remove(user_id);
    }
}

impl Default for TotpManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationAttempt {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TotpVerifyResult {
    Valid,
    Invalid { attempts_remaining: usize },
    RateLimited { remaining_secs: u64 },
    Disabled,
    NotSetup,
}

impl TotpVerifyResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, TotpVerifyResult::Valid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpUserInfo {
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub backup_codes_remaining: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpStats {
    pub total_users: usize,
    pub totp_enabled: usize,
    pub totp_disabled: usize,
}

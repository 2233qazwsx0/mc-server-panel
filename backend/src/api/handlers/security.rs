pub use crate::security::handlers::*;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json, Router,
};
use serde::Deserialize;

use crate::state::AppState;
use crate::security::*;
use crate::config::Config;

#[derive(Clone)]
pub struct SecurityAppState {
    pub security: SecurityState,
}

pub fn create_security_routes() -> Router {
    Router::new()
        .route("/security/ip/whitelist", get(get_whitelist).post(add_to_whitelist).delete(clear_whitelist))
        .route("/security/ip/blacklist", get(get_blacklist).post(add_to_blacklist).delete(clear_blacklist))
        .route("/security/ip/check/:ip", get(check_ip))
        .route("/security/ip/stats", get(get_ip_stats))
        .route("/security/ddos/stats", get(get_ddos_stats))
        .route("/security/ddos/config", get(get_ddos_config).post(set_ddos_config))
        .route("/security/ddos/unblock/:ip", delete(unblock_ip))
        .route("/security/bruteforce/stats", get(get_bruteforce_stats))
        .route("/security/bruteforce/locked", get(get_locked_ips))
        .route("/security/bruteforce/unblock/:ip", delete(unblock_bruteforce_ip))
        .route("/security/bruteforce/config", get(get_bruteforce_config).post(set_bruteforce_config))
        .route("/security/ssl/cert", get(get_ssl_cert))
        .route("/security/ssl/check", get(check_ssl_expiry))
        .route("/security/ssl/renew", post(initiate_ssl_renewal))
        .route("/security/ssl/stats", get(get_ssl_stats))
        .route("/security/totp/setup", post(setup_totp))
        .route("/security/totp/enable/:user_id", post(enable_totp))
        .route("/security/totp/disable/:user_id", post(disable_totp))
        .route("/security/totp/verify/:user_id", post(verify_totp))
        .route("/security/totp/stats", get(get_totp_stats))
        .route("/security/totp/info/:user_id", get(get_totp_info))
        .route("/security/audit/logs", get(get_audit_logs))
        .route("/security/audit/stats", get(get_audit_stats))
        .route("/security/session/list", get(list_sessions))
        .route("/security/session/:session_id", get(get_session).delete(invalidate_session))
        .route("/security/session/invalidate/:user_id", delete(invalidate_user_sessions))
        .route("/security/session/stats", get(get_session_stats))
        .route("/security/apikey/list", get(list_api_keys))
        .route("/security/apikey/create", post(create_api_key))
        .route("/security/apikey/:key_id", get(get_api_key).delete(delete_api_key))
        .route("/security/apikey/:key_id/enable", post(enable_api_key))
        .route("/security/apikey/:key_id/disable", post(disable_api_key))
        .route("/security/apikey/stats", get(get_api_key_stats))
        .route("/security/encrypt", post(encrypt_data))
        .route("/security/decrypt", post(decrypt_data))
        .route("/security/hash", post(hash_password))
        .route("/security/scan", post(run_security_scan))
        .route("/security/scan/baselines", get(get_security_baselines))
        .route("/security/scan/reports", get(get_scan_reports))
        .route("/security/scan/latest", get(get_latest_scan_report))
        .route("/security/scan/stats", get(get_scan_stats))
        .with_state(AppState::new(Config::default()))
}

async fn get_whitelist(State(state): State<AppState>) -> Json<Vec<String>> {
    if let Some(security) = &state.security {
        Json(security.ip_filter.get_whitelist())
    } else {
        Json(Vec::new())
    }
}

async fn add_to_whitelist(
    State(state): State<AppState>,
    Json(payload): Json<IpEntryRequest>,
) -> Result<Json<()>, StatusCode> {
    if let Some(security) = &state.security {
        security.ip_filter
            .add_to_whitelist(&payload.ip, payload.reason)
            .map(|_| Json(()))
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn clear_whitelist(State(state): State<AppState>) -> Json<()> {
    if let Some(security) = &state.security {
        security.ip_filter.clear_whitelist();
    }
    Json(())
}

async fn get_blacklist(State(state): State<AppState>) -> Json<Vec<String>> {
    if let Some(security) = &state.security {
        Json(security.ip_filter.get_blacklist())
    } else {
        Json(Vec::new())
    }
}

async fn add_to_blacklist(
    State(state): State<AppState>,
    Json(payload): Json<IpEntryRequest>,
) -> Result<Json<()>, StatusCode> {
    if let Some(security) = &state.security {
        security.ip_filter
            .add_to_blacklist(&payload.ip, payload.reason)
            .map(|_| Json(()))
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn clear_blacklist(State(state): State<AppState>) -> Json<()> {
    if let Some(security) = &state.security {
        security.ip_filter.clear_blacklist();
    }
    Json(())
}

async fn check_ip(
    State(state): State<AppState>,
    Path(ip): Path<String>,
) -> Json<IpCheckResponse> {
    if let Some(security) = &state.security {
        let result = security.ip_filter.check_ip(&ip);
        Json(IpCheckResponse {
            ip,
            allowed: result.is_allowed(),
            reason: match result {
                IpCheckResult::Allowed { reason } => reason,
                IpCheckResult::Denied { reason } => reason,
                IpCheckResult::Invalid { reason } => reason,
            },
        })
    } else {
        Json(IpCheckResponse {
            ip,
            allowed: true,
            reason: "security module not loaded".to_string(),
        })
    }
}

async fn get_ip_stats(State(state): State<AppState>) -> Json<IpFilterStats> {
    if let Some(security) = &state.security {
        Json(security.ip_filter.get_stats())
    } else {
        Json(IpFilterStats {
            whitelist_count: 0,
            blacklist_count: 0,
            total_entries: 0,
        })
    }
}

async fn get_ddos_stats(State(state): State<AppState>) -> Json<DdosStats> {
    if let Some(security) = &state.security {
        Json(security.ddos_guard.get_stats())
    } else {
        Json(DdosStats {
            total_requests: 0,
            active_ips: 0,
            currently_blocked: 0,
            high_traffic_ips: Vec::new(),
        })
    }
}

async fn get_ddos_config(State(state): State<AppState>) -> Json<RateLimitConfig> {
    if let Some(security) = &state.security {
        Json(security.ddos_guard.get_config())
    } else {
        Json(RateLimitConfig::default())
    }
}

async fn set_ddos_config(
    State(state): State<AppState>,
    Json(config): Json<RateLimitConfig>,
) -> Json<()> {
    if let Some(security) = &state.security {
        security.ddos_guard.set_config(config);
    }
    Json(())
}

async fn unblock_ip(
    State(state): State<AppState>,
    Path(ip): Path<String>,
) -> Result<Json<()>, StatusCode> {
    if let Some(security) = &state.security {
        security.ddos_guard.unblock_ip(&ip)
            .map(|_| Json(()))
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn get_bruteforce_stats(State(state): State<AppState>) -> Json<BruteForceStats> {
    if let Some(security) = &state.security {
        Json(security.brute_force_guard.get_stats())
    } else {
        Json(BruteForceStats {
            tracked_ips: 0,
            locked_ips: 0,
            total_attempts: 0,
            total_failures: 0,
            alerts_count: 0,
        })
    }
}

async fn get_locked_ips(State(state): State<AppState>) -> Json<Vec<LockedIpInfo>> {
    if let Some(security) = &state.security {
        Json(security.brute_force_guard.get_locked_ips())
    } else {
        Json(Vec::new())
    }
}

async fn unblock_bruteforce_ip(
    State(state): State<AppState>,
    Path(ip): Path<String>,
) -> Result<Json<()>, StatusCode> {
    if let Some(security) = &state.security {
        security.brute_force_guard.unlock_ip(&ip)
            .map(|_| Json(()))
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn get_bruteforce_config(State(state): State<AppState>) -> Json<BruteForceConfig> {
    if let Some(security) = &state.security {
        Json(security.brute_force_guard.get_config())
    } else {
        Json(BruteForceConfig::default())
    }
}

async fn set_bruteforce_config(
    State(state): State<AppState>,
    Json(config): Json<BruteForceConfig>,
) -> Json<()> {
    if let Some(security) = &state.security {
        security.brute_force_guard.set_config(config);
    }
    Json(())
}

async fn get_ssl_cert(State(state): State<AppState>) -> Result<Json<Option<CertificateInfo>>, StatusCode> {
    if let Some(security) = &state.security {
        match security.ssl_manager.load_certificate() {
            Ok(cert) => Ok(Json(Some(cert))),
            Err(_) => Ok(Json(None)),
        }
    } else {
        Ok(Json(None))
    }
}

async fn check_ssl_expiry(State(state): State<AppState>) -> Json<CertificateExpiryStatus> {
    if let Some(security) = &state.security {
        Json(security.ssl_manager.check_certificate_expiry())
    } else {
        Json(CertificateExpiryStatus::NotLoaded)
    }
}

async fn initiate_ssl_renewal(State(state): State<AppState>) -> Result<Json<RenewalTask>, StatusCode> {
    if let Some(security) = &state.security {
        match security.ssl_manager.initiate_renewal() {
            Ok(task_id) => {
                if let Some(task) = security.ssl_manager.get_renewal_status(&task_id) {
                    Ok(Json(task))
                } else {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(_) => Err(StatusCode::BAD_REQUEST),
        }
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn get_ssl_stats(State(state): State<AppState>) -> Json<SslCertStats> {
    if let Some(security) = &state.security {
        Json(security.ssl_manager.get_stats())
    } else {
        Json(SslCertStats {
            certificate_loaded: false,
            days_until_expiry: None,
            total_renewals: 0,
            successful_renewals: 0,
            failed_renewals: 0,
            pending_renewals: 0,
        })
    }
}

async fn setup_totp(
    State(state): State<AppState>,
    Json(payload): Json<TotpSetupRequest>,
) -> Json<TotpSetup> {
    if let Some(security) = &state.security {
        Json(security.totp_manager.generate_secret(&payload.user_id))
    } else {
        Json(TotpSetup {
            secret: String::new(),
            qr_code_uri: String::new(),
            manual_entry_key: String::new(),
            backup_codes: Vec::new(),
        })
    }
}

async fn enable_totp(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(payload): Json<TotpVerifyRequest>,
) -> Result<Json<TotpVerifyResponse>, StatusCode> {
    if let Some(security) = &state.security {
        match security.totp_manager.enable_totp(&user_id, &payload.token) {
            Ok(_) => Ok(Json(TotpVerifyResponse { success: true, message: "TOTP enabled".to_string() })),
            Err(e) => Ok(Json(TotpVerifyResponse { success: false, message: e })),
        }
    } else {
        Ok(Json(TotpVerifyResponse { success: false, message: "security module not loaded".to_string() }))
    }
}

async fn disable_totp(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<TotpVerifyResponse>, StatusCode> {
    if let Some(security) = &state.security {
        match security.totp_manager.disable_totp(&user_id) {
            Ok(_) => Ok(Json(TotpVerifyResponse { success: true, message: "TOTP disabled".to_string() })),
            Err(e) => Ok(Json(TotpVerifyResponse { success: false, message: e })),
        }
    } else {
        Ok(Json(TotpVerifyResponse { success: false, message: "security module not loaded".to_string() }))
    }
}

async fn verify_totp(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(payload): Json<TotpVerifyRequest>,
) -> Json<TotpVerifyResponse> {
    if let Some(security) = &state.security {
        let result = security.totp_manager.verify_token(&user_id, &payload.token);
        Json(TotpVerifyResponse {
            success: result.is_valid(),
            message: match result {
                TotpVerifyResult::Valid => "Valid token".to_string(),
                TotpVerifyResult::Invalid { attempts_remaining } => format!("Invalid token, {} attempts remaining", attempts_remaining),
                TotpVerifyResult::RateLimited { remaining_secs } => format!("Rate limited, try again in {} seconds", remaining_secs),
                TotpVerifyResult::Disabled => "TOTP is disabled".to_string(),
                TotpVerifyResult::NotSetup => "TOTP not setup".to_string(),
            },
        })
    } else {
        Json(TotpVerifyResponse { success: false, message: "security module not loaded".to_string() })
    }
}

async fn get_totp_stats(State(state): State<AppState>) -> Json<TotpStats> {
    if let Some(security) = &state.security {
        Json(security.totp_manager.get_stats())
    } else {
        Json(TotpStats {
            total_users: 0,
            totp_enabled: 0,
            totp_disabled: 0,
        })
    }
}

async fn get_totp_info(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Json<Option<TotpUserInfo>> {
    if let Some(security) = &state.security {
        Json(security.totp_manager.get_user_totp_info(&user_id))
    } else {
        Json(None)
    }
}

async fn get_audit_logs(
    State(state): State<AppState>,
    Query(filter): Query<AuditLogFilter>,
) -> Json<Vec<AuditLogEntry>> {
    if let Some(security) = &state.security {
        Json(security.audit_logger.get_entries(filter))
    } else {
        Json(Vec::new())
    }
}

async fn get_audit_stats(State(state): State<AppState>) -> Json<AuditStats> {
    if let Some(security) = &state.security {
        Json(security.audit_logger.get_stats())
    } else {
        Json(AuditStats {
            total_entries: 0,
            success_count: 0,
            failure_count: 0,
            action_counts: std::collections::HashMap::new(),
            user_activity_counts: std::collections::HashMap::new(),
            last_entry: None,
        })
    }
}

async fn list_sessions(State(state): State<AppState>) -> Json<Vec<Session>> {
    if let Some(security) = &state.security {
        Json(security.session_manager.get_active_sessions())
    } else {
        Json(Vec::new())
    }
}

async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<Option<Session>>, StatusCode> {
    if let Some(security) = &state.security {
        Ok(Json(security.session_manager.get_session(&session_id)))
    } else {
        Ok(Json(None))
    }
}

async fn invalidate_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<()>, StatusCode> {
    if let Some(security) = &state.security {
        security.session_manager.invalidate_session(&session_id)
            .map(|_| Json(()))
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn invalidate_user_sessions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Json<usize> {
    if let Some(security) = &state.security {
        Json(security.session_manager.invalidate_all_user_sessions(&user_id))
    } else {
        Json(0)
    }
}

async fn get_session_stats(State(state): State<AppState>) -> Json<SessionStats> {
    if let Some(security) = &state.security {
        Json(security.session_manager.get_stats())
    } else {
        Json(SessionStats {
            total_sessions: 0,
            active_sessions: 0,
            expired_sessions: 0,
            unique_users: 0,
            sessions_per_user: std::collections::HashMap::new(),
            sessions_per_ip: std::collections::HashMap::new(),
        })
    }
}

async fn list_api_keys(State(state): State<AppState>) -> Json<Vec<ApiKey>> {
    if let Some(security) = &state.security {
        Json(security.api_key_manager.list_keys())
    } else {
        Json(Vec::new())
    }
}

async fn create_api_key(
    State(state): State<AppState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Json<CreateApiKeyResponse> {
    if let Some(security) = &state.security {
        let (key, raw_key) = security.api_key_manager.create_key(
            payload.name,
            payload.permissions,
            payload.user_id,
        );
        Json(CreateApiKeyResponse { key, raw_key })
    } else {
        Json(CreateApiKeyResponse {
            key: ApiKey {
                id: String::new(),
                name: String::new(),
                key_prefix: String::new(),
                user_id: None,
                permissions: Vec::new(),
                created_at: chrono::Utc::now(),
                last_used: None,
                expires_at: None,
                is_active: false,
                description: None,
            },
            raw_key: String::new(),
        })
    }
}

async fn get_api_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> Result<Json<Option<ApiKey>>, StatusCode> {
    if let Some(security) = &state.security {
        Ok(Json(security.api_key_manager.get_key(&key_id)))
    } else {
        Ok(Json(None))
    }
}

async fn delete_api_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> Result<Json<()>, StatusCode> {
    if let Some(security) = &state.security {
        security.api_key_manager.delete_key(&key_id)
            .map(|_| Json(()))
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn enable_api_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> Result<Json<()>, StatusCode> {
    if let Some(security) = &state.security {
        security.api_key_manager.enable_key(&key_id)
            .map(|_| Json(()))
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn disable_api_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
) -> Result<Json<()>, StatusCode> {
    if let Some(security) = &state.security {
        security.api_key_manager.disable_key(&key_id)
            .map(|_| Json(()))
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn get_api_key_stats(State(state): State<AppState>) -> Json<ApiKeyStats> {
    if let Some(security) = &state.security {
        Json(security.api_key_manager.get_stats())
    } else {
        Json(ApiKeyStats {
            total_keys: 0,
            active_keys: 0,
            expired_keys: 0,
            permission_distribution: std::collections::HashMap::new(),
        })
    }
}

async fn encrypt_data(
    State(state): State<AppState>,
    Json(payload): Json<EncryptRequest>,
) -> Result<Json<EncryptedData>, StatusCode> {
    if let Some(security) = &state.security {
        security.encryption.encrypt(&payload.data)
            .map(Json)
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn decrypt_data(
    State(state): State<AppState>,
    Json(payload): Json<DecryptRequest>,
) -> Result<Json<DecryptResponse>, StatusCode> {
    if let Some(security) = &state.security {
        match security.encryption.decrypt(&payload.data) {
            Ok(plaintext) => Ok(Json(DecryptResponse { success: true, data: plaintext })),
            Err(_) => Ok(Json(DecryptResponse { success: false, data: String::new() })),
        }
    } else {
        Ok(Json(DecryptResponse { success: false, data: String::new() }))
    }
}

async fn hash_password(
    State(state): State<AppState>,
    Json(payload): Json<HashRequest>,
) -> Result<Json<HashResult>, StatusCode> {
    if let Some(security) = &state.security {
        security.encryption.hash_password(&payload.password)
            .map(Json)
            .map_err(|_| StatusCode::BAD_REQUEST)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn run_security_scan(State(state): State<AppState>) -> Json<ScanReport> {
    if let Some(security) = &state.security {
        Json(security.security_scanner.run_scan())
    } else {
        Json(ScanReport {
            id: String::new(),
            timestamp: chrono::Utc::now(),
            overall_score: 0,
            total_checks: 0,
            passed: 0,
            failed: 0,
            warnings: 0,
            errors: 0,
            category_scores: std::collections::HashMap::new(),
            critical_findings: Vec::new(),
            recommendations: Vec::new(),
        })
    }
}

async fn get_security_baselines(State(state): State<AppState>) -> Json<Vec<SecurityBaseline>> {
    if let Some(security) = &state.security {
        Json(security.security_scanner.get_baselines())
    } else {
        Json(Vec::new())
    }
}

async fn get_scan_reports(State(state): State<AppState>) -> Json<Vec<ScanReport>> {
    if let Some(security) = &state.security {
        Json(security.security_scanner.get_reports())
    } else {
        Json(Vec::new())
    }
}

async fn get_latest_scan_report(State(state): State<AppState>) -> Json<Option<ScanReport>> {
    if let Some(security) = &state.security {
        Json(security.security_scanner.get_latest_report())
    } else {
        Json(None)
    }
}

async fn get_scan_stats(State(state): State<AppState>) -> Json<SecurityScanStats> {
    if let Some(security) = &state.security {
        Json(security.security_scanner.get_stats())
    } else {
        Json(SecurityScanStats {
            total_baselines: 0,
            enabled_baselines: 0,
            total_scans: 0,
            latest_scan_score: None,
            last_scan_time: None,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct IpEntryRequest {
    pub ip: String,
    pub reason: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct IpCheckResponse {
    pub ip: String,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct TotpSetupRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    pub token: String,
}

#[derive(Debug, serde::Serialize)]
pub struct TotpVerifyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<ApiPermission>,
    pub user_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateApiKeyResponse {
    pub key: ApiKey,
    pub raw_key: String,
}

#[derive(Debug, Deserialize)]
pub struct EncryptRequest {
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct DecryptRequest {
    pub data: EncryptedData,
}

#[derive(Debug, serde::Serialize)]
pub struct DecryptResponse {
    pub success: bool,
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct HashRequest {
    pub password: String,
}

use crate::security::ip_filter::{IpFilterStats, IpCheckResult};
use crate::security::ddos_protection::{DdosStats, RateLimitConfig};
use crate::security::rcon_brute_force::{BruteForceConfig, BruteForceStats, LockedIpInfo};
use crate::security::ssl_cert::{CertificateInfo, SslCertStats, RenewalTask, CertificateExpiryStatus};
use crate::security::totp::{TotpSetup, TotpStats, TotpUserInfo};
use crate::security::audit_log::{AuditLogEntry, AuditLogFilter, AuditStats};
use crate::security::session::{Session, SessionStats};
use crate::security::api_keys::{ApiKey, ApiKeyStats, ApiPermission};
use crate::security::encryption::{EncryptedData, HashResult};
use crate::security::security_scan::{SecurityBaseline, ScanReport, SecurityScanStats};

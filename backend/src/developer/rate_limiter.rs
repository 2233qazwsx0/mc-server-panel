use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::Utc;

use crate::state::AppState;
use crate::developer::DeveloperState;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    pub requests: HashMap<String, Vec<i64>>,
    pub config: RateLimitConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: usize,
    pub requests_per_hour: usize,
    pub burst_size: usize,
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_size: 10,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub client_id: String,
    pub requests_this_minute: usize,
    pub requests_this_hour: usize,
    pub remaining_this_minute: usize,
    pub remaining_this_hour: usize,
    pub reset_at_minute: i64,
    pub reset_at_hour: i64,
    pub is_limited: bool,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            requests: HashMap::new(),
            config: RateLimitConfig::default(),
        }
    }
    
    pub fn new_with_config(config: RateLimitConfig) -> Self {
        Self {
            requests: HashMap::new(),
            config,
        }
    }
    
    pub fn check_rate_limit(&mut self, client_id: &str) -> RateLimitInfo {
        let now = Utc::now().timestamp();
        let minute_start = now - (now % 60);
        let hour_start = now - (now % 3600);
        
        let requests = self.requests.entry(client_id.to_string())
            .or_insert_with(Vec::new);
        
        requests.retain(|&t| t >= minute_start - 3600);
        
        let requests_this_minute = requests.iter()
            .filter(|&&t| t >= minute_start)
            .count();
        
        let requests_this_hour = requests.len();
        
        let remaining_this_minute = self.config.requests_per_minute.saturating_sub(requests_this_minute);
        let remaining_this_hour = self.config.requests_per_hour.saturating_sub(requests_this_hour);
        
        let is_limited = requests_this_minute >= self.config.requests_per_minute 
            || requests_this_hour >= self.config.requests_per_hour;
        
        if !is_limited {
            requests.push(now);
        }
        
        RateLimitInfo {
            client_id: client_id.to_string(),
            requests_this_minute,
            requests_this_hour,
            remaining_this_minute,
            remaining_this_hour,
            reset_at_minute: (minute_start + 60) - now,
            reset_at_hour: (hour_start + 3600) - now,
            is_limited,
        }
    }
    
    pub fn reset_client(&mut self, client_id: &str) {
        self.requests.remove(client_id);
    }
    
    pub fn reset_all(&mut self) {
        self.requests.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RateLimitStatus {
    pub enabled: bool,
    pub requests_per_minute: usize,
    pub requests_per_hour: usize,
    pub burst_size: usize,
    pub total_tracked_clients: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RateLimitUpdateRequest {
    pub requests_per_minute: Option<usize>,
    pub requests_per_hour: Option<usize>,
    pub burst_size: Option<usize>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RateLimitCheckResponse {
    pub allowed: bool,
    pub client_id: String,
    pub remaining: usize,
    pub reset_at: i64,
    pub retry_after: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RateLimitCheckRequest {
    pub client_id: String,
}

#[utoipa::path(
    get,
    path = "/api/developer/rate-limit/status",
    responses(
        (status = 200, description = "Get rate limit status", body = RateLimitStatus)
    ),
    tag = "Developer"
)]
pub async fn get_rate_limit_status(
    State(state): State<AppState>,
) -> Result<Json<RateLimitStatus>, crate::error::AppError> {
    let limiter = state.developer_state.rate_limiter.read();
    
    Ok(Json(RateLimitStatus {
        enabled: limiter.config.enabled,
        requests_per_minute: limiter.config.requests_per_minute,
        requests_per_hour: limiter.config.requests_per_hour,
        burst_size: limiter.config.burst_size,
        total_tracked_clients: limiter.requests.len(),
    }))
}

#[utoipa::path(
    put,
    path = "/api/developer/rate-limit/config",
    request_body = RateLimitUpdateRequest,
    responses(
        (status = 200, description = "Update rate limit configuration", body = RateLimitStatus)
    ),
    tag = "Developer"
)]
pub async fn update_rate_limit_config(
    State(state): State<AppState>,
    Json(req): Json<RateLimitUpdateRequest>,
) -> Result<Json<RateLimitStatus>, crate::error::AppError> {
    let mut limiter = state.developer_state.rate_limiter.write();
    
    if let Some(rpm) = req.requests_per_minute {
        limiter.config.requests_per_minute = rpm;
    }
    if let Some(rph) = req.requests_per_hour {
        limiter.config.requests_per_hour = rph;
    }
    if let Some(burst) = req.burst_size {
        limiter.config.burst_size = burst;
    }
    if let Some(enabled) = req.enabled {
        limiter.config.enabled = enabled;
    }
    
    Ok(Json(RateLimitStatus {
        enabled: limiter.config.enabled,
        requests_per_minute: limiter.config.requests_per_minute,
        requests_per_hour: limiter.config.requests_per_hour,
        burst_size: limiter.config.burst_size,
        total_tracked_clients: limiter.requests.len(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/rate-limit/check",
    request_body = RateLimitCheckRequest,
    responses(
        (status = 200, description = "Check rate limit for client", body = RateLimitCheckResponse),
        (status = 429, description = "Rate limit exceeded")
    ),
    tag = "Developer"
)]
pub async fn check_rate_limit(
    State(state): State<AppState>,
    Json(req): Json<RateLimitCheckRequest>,
) -> Result<Json<RateLimitCheckResponse>, crate::error::AppError> {
    let mut limiter = state.developer_state.rate_limiter.write();
    
    let info = limiter.check_rate_limit(&req.client_id);
    
    if info.is_limited {
        return Err(crate::error::AppError::Internal(format!(
            "Rate limit exceeded. Retry after {} seconds",
            info.reset_at_minute
        )));
    }
    
    Ok(Json(RateLimitCheckResponse {
        allowed: true,
        client_id: info.client_id,
        remaining: info.remaining_this_minute,
        reset_at: info.reset_at_minute,
        retry_after: None,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/developer/rate-limit/clients/{client_id}",
    responses(
        (status = 200, description = "Reset rate limit for client")
    ),
    params(
        ("client_id" = String, Path, description = "Client ID to reset")
    ),
    tag = "Developer"
)]
pub async fn reset_client_limit(
    State(state): State<AppState>,
    axum::extract::Path(client_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    state.developer_state.rate_limiter.write()
        .reset_client(&client_id);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "client_id": client_id
    })))
}

#[utoipa::path(
    delete,
    path = "/api/developer/rate-limit/clients",
    responses(
        (status = 200, description = "Reset all rate limits")
    ),
    tag = "Developer"
)]
pub async fn reset_all_limits(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    state.developer_state.rate_limiter.write()
        .reset_all();
    
    Ok(Json(serde_json::json!({
        "success": true
    })))
}

pub fn rate_limit_middleware(
    state: Arc<DeveloperState>,
    client_id: &str,
) -> Option<RateLimitInfo> {
    let mut limiter = state.rate_limiter.write();
    let info = limiter.check_rate_limit(client_id);
    
    if info.is_limited && limiter.config.enabled {
        Some(info)
    } else {
        None
    }
}

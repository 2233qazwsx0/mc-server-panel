use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::Utc;
use std::sync::Arc;

use crate::state::AppState;
use crate::developer::{DeveloperState, RequestLogEntry};

#[derive(Debug, Serialize, ToSchema)]
pub struct RequestLogListResponse {
    pub logs: Vec<RequestLogEntry>,
    pub total: usize,
    pub filtered: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RequestLogFilter {
    pub method: Option<String>,
    pub path: Option<String>,
    pub status: Option<u16>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RequestLogStats {
    pub total_requests: usize,
    pub requests_per_method: std::collections::HashMap<String, usize>,
    pub average_duration_ms: f64,
    pub slow_requests: Vec<RequestLogEntry>,
    pub error_requests: Vec<RequestLogEntry>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClearLogsResponse {
    pub success: bool,
    pub deleted_count: usize,
}

pub fn log_request(
    state: &Arc<DeveloperState>,
    method: &str,
    path: &str,
    status: u16,
    duration_ms: u64,
    request_body: Option<String>,
    response_body: Option<String>,
    client_ip: &str,
) {
    let entry = RequestLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        method: method.to_string(),
        path: path.to_string(),
        status,
        duration_ms,
        timestamp: Utc::now(),
        request_body,
        response_body,
        client_ip: client_ip.to_string(),
    };
    
    let mut logs = state.request_logs.write();
    logs.push(entry);
    
    if logs.len() > 10000 {
        logs.drain(0..5000);
    }
}

#[utoipa::path(
    get,
    path = "/api/developer/request-logs",
    responses(
        (status = 200, description = "Get request logs", body = RequestLogListResponse)
    ),
    params(
        ("filter" = Option<RequestLogFilter>, Query, description = "Filter parameters")
    ),
    tag = "Developer"
)]
pub async fn get_request_logs(
    State(state): State<AppState>,
    axum::extract::Query(filter): axum::extract::Query<RequestLogFilter>,
) -> Result<Json<RequestLogListResponse>, crate::error::AppError> {
    let logs = state.developer_state.request_logs.read();
    let total = logs.len();
    
    let mut filtered: Vec<_> = logs.iter().collect();
    
    if let Some(ref method) = filter.method {
        filtered.retain(|l| l.method.to_lowercase() == method.to_lowercase());
    }
    
    if let Some(ref path) = filter.path {
        filtered.retain(|l| l.path.contains(path));
    }
    
    if let Some(status) = filter.status {
        filtered.retain(|l| l.status == status);
    }
    
    if let Some(ref from) = filter.from {
        if let Ok(from_time) = chrono::DateTime::parse_from_rfc3339(from) {
            filtered.retain(|l| l.timestamp >= from_time.with_timezone(&Utc));
        }
    }
    
    if let Some(ref to) = filter.to {
        if let Ok(to_time) = chrono::DateTime::parse_from_rfc3339(to) {
            filtered.retain(|l| l.timestamp <= to_time.with_timezone(&Utc));
        }
    }
    
    let offset = filter.offset.unwrap_or(0);
    let limit = filter.limit.unwrap_or(100);
    
    let paginated: Vec<_> = filtered.iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();
    
    Ok(Json(RequestLogListResponse {
        logs: paginated,
        total,
        filtered: filtered.len(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/developer/request-logs/stats",
    responses(
        (status = 200, description = "Get request log statistics", body = RequestLogStats)
    ),
    tag = "Developer"
)]
pub async fn get_request_log_stats(
    State(state): State<AppState>,
) -> Result<Json<RequestLogStats>, crate::error::AppError> {
    let logs = state.developer_state.request_logs.read();
    
    let mut requests_per_method: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_duration: u64 = 0;
    let mut slow_requests = Vec::new();
    let mut error_requests = Vec::new();
    
    for log in logs.iter() {
        *requests_per_method.entry(log.method.clone()).or_insert(0) += 1;
        total_duration += log.duration_ms;
        
        if log.duration_ms > 1000 {
            slow_requests.push(log.clone());
        }
        
        if log.status >= 400 {
            error_requests.push(log.clone());
        }
    }
    
    slow_requests.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
    slow_requests.truncate(10);
    
    error_requests.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    error_requests.truncate(10);
    
    let average_duration_ms = if !logs.is_empty() {
        total_duration as f64 / logs.len() as f64
    } else {
        0.0
    };
    
    Ok(Json(RequestLogStats {
        total_requests: logs.len(),
        requests_per_method,
        average_duration_ms,
        slow_requests,
        error_requests,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/developer/request-logs",
    responses(
        (status = 200, description = "Clear request logs", body = ClearLogsResponse)
    ),
    tag = "Developer"
)]
pub async fn clear_request_logs(
    State(state): State<AppState>,
) -> Result<Json<ClearLogsResponse>, crate::error::AppError> {
    let count = state.developer_state.request_logs.read().len();
    state.developer_state.request_logs.write().clear();
    
    Ok(Json(ClearLogsResponse {
        success: true,
        deleted_count: count,
    }))
}

#[utoipa::path(
    get,
    path = "/api/developer/request-logs/{id}",
    responses(
        (status = 200, description = "Get single request log", body = RequestLogEntry),
        (status = 404, description = "Log entry not found")
    ),
    params(
        ("id" = String, Path, description = "Log ID")
    ),
    tag = "Developer"
)]
pub async fn get_request_log(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<RequestLogEntry>, crate::error::AppError> {
    let logs = state.developer_state.request_logs.read();
    
    let log = logs.iter()
        .find(|l| l.id == id)
        .cloned()
        .ok_or_else(|| crate::error::AppError::Internal("Log entry not found".to_string()))?;
    
    Ok(Json(log))
}

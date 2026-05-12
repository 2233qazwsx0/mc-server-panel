use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;

use crate::state::AppState;
use crate::developer::{DeveloperState, ProfilerSnapshot};

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfilerSnapshotList {
    pub snapshots: Vec<ProfilerSnapshot>,
    pub total: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfilerStatus {
    pub is_running: bool,
    pub snapshots_count: usize,
    pub total_samples: usize,
}

#[derive(Debug, Clone)]
pub struct ProfilerGuard {
    pub id: String,
    pub name: String,
    pub start_time: Instant,
    pub tags: HashMap<String, String>,
}

lazy_static::lazy_static! {
    static ref ACTIVE_PROFILERS: parking_lot::RwLock<HashMap<String, ProfilerGuard>> = parking_lot::RwLock::new(HashMap::new());
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartProfilerRequest {
    pub name: String,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StartProfilerResponse {
    pub profiler_id: String,
    pub started_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StopProfilerResponse {
    pub snapshot_id: String,
    pub name: String,
    pub duration_ns: u64,
    pub duration_ms: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfilerSnapshotDetail {
    pub id: String,
    pub name: String,
    pub timestamp: String,
    pub duration_ns: u64,
    pub duration_ms: f64,
    pub tags: HashMap<String, String>,
}

#[utoipa::path(
    get,
    path = "/api/developer/profiler/status",
    responses(
        (status = 200, description = "Get profiler status", body = ProfilerStatus)
    ),
    tag = "Developer"
)]
pub async fn get_profiler_status(
    State(state): State<AppState>,
) -> Result<Json<ProfilerStatus>, crate::error::AppError> {
    let active = ACTIVE_PROFILERS.read();
    let snapshots = state.developer_state.profiler_snapshots.read();
    
    Ok(Json(ProfilerStatus {
        is_running: !active.is_empty(),
        snapshots_count: snapshots.len(),
        total_samples: active.len(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/profiler/start",
    request_body = StartProfilerRequest,
    responses(
        (status = 201, description = "Start profiler", body = StartProfilerResponse)
    ),
    tag = "Developer"
)]
pub async fn start_profiler(
    State(state): State<AppState>,
    Json(req): Json<StartProfilerRequest>,
) -> Result<Json<StartProfilerResponse>, crate::error::AppError> {
    let profiler_id = Uuid::new_v4().to_string();
    let started_at = Utc::now();
    
    let guard = ProfilerGuard {
        id: profiler_id.clone(),
        name: req.name.clone(),
        start_time: Instant::now(),
        tags: req.tags.unwrap_or_default(),
    };
    
    ACTIVE_PROFILERS.write().insert(profiler_id.clone(), guard);
    
    Ok(Json(StartProfilerResponse {
        profiler_id,
        started_at: started_at.to_rfc3339(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/profiler/stop",
    responses(
        (status = 200, description = "Stop profiler and create snapshot", body = StopProfilerResponse)
    ),
    params(
        ("id" = Option<String>, Query, description = "Profiler ID (stops all if not provided)")
    ),
    tag = "Developer"
)]
pub async fn stop_profiler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<StopProfilerResponse>, crate::error::AppError> {
    let mut active = ACTIVE_PROFILERS.write();
    
    if let Some(id) = params.get("id") {
        if let Some(guard) = active.remove(id) {
            let duration_ns = guard.start_time.elapsed().as_nanos() as u64;
            let snapshot = ProfilerSnapshot {
                id: Uuid::new_v4().to_string(),
                name: guard.name.clone(),
                timestamp: Utc::now(),
                duration_ns,
                tags: guard.tags.clone(),
            };
            
            state.developer_state.profiler_snapshots.write()
                .push(snapshot.clone());
            
            return Ok(Json(StopProfilerResponse {
                snapshot_id: snapshot.id,
                name: guard.name,
                duration_ns,
                duration_ms: duration_ns as f64 / 1_000_000.0,
            }));
        }
        return Err(crate::error::AppError::Internal("Profiler not found".to_string()));
    }
    
    if let Some((id, guard)) = active.iter().next().cloned() {
        active.remove(&id);
        let duration_ns = guard.start_time.elapsed().as_nanos() as u64;
        let snapshot = ProfilerSnapshot {
            id: Uuid::new_v4().to_string(),
            name: guard.name.clone(),
            timestamp: Utc::now(),
            duration_ns,
            tags: guard.tags.clone(),
        };
        
        state.developer_state.profiler_snapshots.write()
            .push(snapshot.clone());
        
        return Ok(Json(StopProfilerResponse {
            snapshot_id: snapshot.id,
            name: guard.name,
            duration_ns,
            duration_ms: duration_ns as f64 / 1_000_000.0,
        }));
    }
    
    Err(crate::error::AppError::Internal("No active profiler".to_string()))
}

#[utoipa::path(
    get,
    path = "/api/developer/profiler/snapshots",
    responses(
        (status = 200, description = "Get profiler snapshots", body = ProfilerSnapshotList)
    ),
    params(
        ("name" = Option<String>, Query, description = "Filter by name"),
        ("limit" = Option<usize>, Query, description = "Limit results"),
        ("offset" = Option<usize>, Query, description = "Offset for pagination")
    ),
    tag = "Developer"
)]
pub async fn get_profiler_snapshots(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<ProfilerSnapshotList>, crate::error::AppError> {
    let snapshots = state.developer_state.profiler_snapshots.read().clone();
    
    let mut filtered: Vec<_> = snapshots;
    
    if let Some(name) = params.get("name") {
        filtered.retain(|s| s.name.contains(name));
    }
    
    let offset = params.get("offset")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let limit = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    
    filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    let total = filtered.len();
    filtered = filtered.into_iter().skip(offset).take(limit).collect();
    
    Ok(Json(ProfilerSnapshotList {
        snapshots: filtered,
        total,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/developer/profiler/snapshots",
    responses(
        (status = 200, description = "Clear profiler snapshots")
    ),
    tag = "Developer"
)]
pub async fn clear_profiler_snapshots(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    state.developer_state.profiler_snapshots.write().clear();
    
    Ok(Json(serde_json::json!({ "success": true })))
}

pub fn profile_function<F, T>(
    name: &str,
    tags: HashMap<String, String>,
    f: F
) -> T
where
    F: FnOnce() -> T
{
    let start = Instant::now();
    let result = f();
    let duration_ns = start.elapsed().as_nanos() as u64;
    
    tracing::debug!(
        name = name,
        duration_ns = duration_ns,
        duration_ms = duration_ns as f64 / 1_000_000.0,
        ?tags,
        "Profiler function completed"
    );
    
    result
}

pub async fn profile_async<F, T>(
    name: &str,
    tags: HashMap<String, String>,
    f: F
) -> T
where
    F: std::future::Future<Output = T>
{
    let start = Instant::now();
    let result = f.await;
    let duration_ns = start.elapsed().as_nanos() as u64;
    
    tracing::debug!(
        name = name,
        duration_ns = duration_ns,
        duration_ms = duration_ns as f64 / 1_000_000.0,
        ?tags,
        "Profiler async function completed"
    );
    
    result
}

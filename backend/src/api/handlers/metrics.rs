use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::monitor::SystemMetrics;
use crate::state::AppState;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub seconds: Option<u64>,
}

#[derive(Debug, serde::Serialize)]
pub struct MetricsResponse {
    pub system: SystemMetrics,
    pub process_memory_mb: Option<f64>,
    pub process_cpu: Option<f32>,
}

pub async fn get_metrics(
    State(state): State<AppState>,
) -> Result<Json<MetricsResponse>> {
    let pid = state.process_manager.get_pid().await;
    let snapshot = state.monitor.collect(pid).await;

    let (process_memory_mb, process_cpu) = if let Some(pid) = pid {
        if let Some(proc) = state.monitor.get_process_metrics(pid).await {
            (Some(proc.memory_used as f64 / 1024.0 / 1024.0), Some(proc.cpu_usage))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(Json(MetricsResponse {
        system: snapshot.system,
        process_memory_mb,
        process_cpu,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct HistoryResponse {
    pub metrics: Vec<SystemMetrics>,
    pub duration_secs: u64,
}

pub async fn get_metrics_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let seconds = params.seconds.unwrap_or(60);
    let history = state.monitor.get_history(seconds).await;

    Ok(Json(HistoryResponse {
        metrics: history,
        duration_secs: seconds,
    }))
}

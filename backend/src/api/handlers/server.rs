use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::result::Result as StdResult;

use crate::error::AppError;
use crate::state::AppState;

pub type Result<T> = StdResult<T, AppError>;

#[derive(Debug, Serialize)]
pub struct ServerStatusResponse {
    pub running: bool,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<i64>,
}

pub async fn get_server_status(
    State(state): State<AppState>,
) -> Result<Json<ServerStatusResponse>> {
    let running = state.process_manager.is_running().await;
    let pid = state.process_manager.get_pid().await;

    Ok(Json(ServerStatusResponse {
        running,
        pid,
        uptime_seconds: None,
    }))
}

pub async fn start_server(
    State(state): State<AppState>,
) -> Result<Json<ServerStatusResponse>> {
    if state.process_manager.is_running().await {
        return Err(AppError::ServerAlreadyRunning);
    }

    let permit = state.process_manager.acquire_permit().await?;
    let handle = state.process_manager.start(&state.config.server, permit).await?;

    Ok(Json(ServerStatusResponse {
        running: true,
        pid: Some(handle.pid),
        uptime_seconds: Some(0),
    }))
}

#[axum::debug_handler]
pub async fn stop_server(
    State(state): State<AppState>,
) -> Result<Json<ServerStatusResponse>> {
    if !state.process_manager.is_running().await {
        return Err(AppError::ServerNotRunning);
    }

    state.process_manager.stop().await?;

    Ok(Json(ServerStatusResponse {
        running: false,
        pid: None,
        uptime_seconds: None,
    }))
}

pub async fn restart_server(
    State(state): State<AppState>,
) -> Result<Json<ServerStatusResponse>> {
    let handle = state.process_manager.restart(&state.config.server).await?;

    Ok(Json(ServerStatusResponse {
        running: true,
        pid: Some(handle.pid),
        uptime_seconds: Some(0),
    }))
}

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
}

pub async fn send_command(
    State(state): State<AppState>,
    Json(req): Json<CommandRequest>,
) -> Result<Json<CommandResponse>> {
    if !state.process_manager.is_running().await {
        return Err(AppError::ServerNotRunning);
    }

    state.process_manager.send_command(&req.command).await?;

    Ok(Json(CommandResponse {
        success: true,
        message: format!("Command executed: {}", req.command),
    }))
}

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub logs: Vec<crate::core::LogEntry>,
    pub total: usize,
}

pub async fn get_logs(
    State(state): State<AppState>,
    Query(params): Query<LogsQuery>,
) -> Result<Json<LogsResponse>> {
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100);

    let logs = state.process_manager.get_logs(offset).await;
    let total = logs.len();

    let logs = logs.into_iter().take(limit).collect();

    Ok(Json(LogsResponse { logs, total }))
}

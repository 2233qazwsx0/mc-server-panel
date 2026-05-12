use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::Utc;

use crate::state::AppState;
use crate::developer::{DeveloperState, WsDebugSession, WsMessageEntry};

#[derive(Debug, Serialize, ToSchema)]
pub struct WsDebugSessionInfo {
    pub id: String,
    pub created_at: String,
    pub message_count: usize,
    pub is_active: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WsDebugListResponse {
    pub sessions: Vec<WsDebugSessionInfo>,
    pub total: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WsDebugSendRequest {
    pub session_id: String,
    pub message: String,
    pub direction: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WsDebugSendResponse {
    pub success: bool,
    pub message_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WsDebugSessionDetail {
    pub id: String,
    pub created_at: String,
    pub messages: Vec<WsMessageDetail>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WsMessageDetail {
    pub id: String,
    pub direction: String,
    pub content: String,
    pub timestamp: String,
    pub parsed: Option<serde_json::Value>,
}

#[utoipa::path(
    get,
    path = "/api/developer/ws-debug/sessions",
    responses(
        (status = 200, description = "List WebSocket debug sessions", body = WsDebugListResponse)
    ),
    tag = "Developer"
)]
pub async fn list_ws_debug_sessions(
    State(state): State<AppState>,
) -> Result<Json<WsDebugListResponse>, crate::error::AppError> {
    let sessions = state.developer_state.ws_debug_sessions.read();
    let session_infos: Vec<WsDebugSessionInfo> = sessions.values()
        .map(|s| WsDebugSessionInfo {
            id: s.id.clone(),
            created_at: s.created_at.to_rfc3339(),
            message_count: s.messages.len(),
            is_active: s.is_active,
        })
        .collect();
    
    Ok(Json(WsDebugListResponse {
        total: session_infos.len(),
        sessions: session_infos,
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/ws-debug/sessions",
    responses(
        (status = 201, description = "Create new WebSocket debug session", body = WsDebugSessionInfo)
    ),
    tag = "Developer"
)]
pub async fn create_ws_debug_session(
    State(state): State<AppState>,
) -> Result<Json<WsDebugSessionInfo>, crate::error::AppError> {
    let session = WsDebugSession::new();
    let info = WsDebugSessionInfo {
        id: session.id.clone(),
        created_at: session.created_at.to_rfc3339(),
        message_count: 0,
        is_active: true,
    };
    
    state.developer_state.ws_debug_sessions.write()
        .insert(session.id.clone(), session);
    
    Ok(Json(info))
}

#[utoipa::path(
    get,
    path = "/api/developer/ws-debug/sessions/{id}",
    responses(
        (status = 200, description = "Get WebSocket debug session details", body = WsDebugSessionDetail),
        (status = 404, description = "Session not found")
    ),
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    tag = "Developer"
)]
pub async fn get_ws_debug_session(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<WsDebugSessionDetail>, crate::error::AppError> {
    let sessions = state.developer_state.ws_debug_sessions.read();
    
    let session = sessions.get(&id)
        .ok_or_else(|| crate::error::AppError::Internal("Session not found".to_string()))?;
    
    let messages: Vec<WsMessageDetail> = session.messages.iter()
        .map(|m| {
            let parsed = serde_json::from_str(&m.content).ok();
            WsMessageDetail {
                id: m.id.clone(),
                direction: m.direction.clone(),
                content: m.content.clone(),
                timestamp: m.timestamp.to_rfc3339(),
                parsed,
            }
        })
        .collect();
    
    Ok(Json(WsDebugSessionDetail {
        id: session.id.clone(),
        created_at: session.created_at.to_rfc3339(),
        messages,
        is_active: session.is_active,
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/ws-debug/send",
    request_body = WsDebugSendRequest,
    responses(
        (status = 200, description = "Send message to debug session", body = WsDebugSendResponse),
        (status = 404, description = "Session not found")
    ),
    tag = "Developer"
)]
pub async fn ws_debug_send(
    State(state): State<AppState>,
    Json(req): Json<WsDebugSendRequest>,
) -> Result<Json<WsDebugSendResponse>, crate::error::AppError> {
    let mut sessions = state.developer_state.ws_debug_sessions.write();
    
    let session = sessions.get_mut(&req.session_id)
        .ok_or_else(|| crate::error::AppError::Internal("Session not found".to_string()))?;
    
    let message_id = Uuid::new_v4().to_string();
    let entry = WsMessageEntry {
        id: message_id.clone(),
        direction: req.direction,
        content: req.message,
        timestamp: Utc::now(),
    };
    
    session.messages.push(entry);
    
    Ok(Json(WsDebugSendResponse {
        success: true,
        message_id,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/developer/ws-debug/sessions/{id}",
    responses(
        (status = 200, description = "Delete WebSocket debug session")
    ),
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    tag = "Developer"
)]
pub async fn delete_ws_debug_session(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    state.developer_state.ws_debug_sessions.write()
        .remove(&id);
    
    Ok(Json(serde_json::json!({ "success": true })))
}

pub fn record_ws_message(state: &DeveloperState, session_id: &str, direction: &str, content: &str) {
    if let Some(session) = state.ws_debug_sessions.write().get_mut(session_id) {
        let entry = WsMessageEntry {
            id: Uuid::new_v4().to_string(),
            direction: direction.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        };
        session.messages.push(entry);
    }
}

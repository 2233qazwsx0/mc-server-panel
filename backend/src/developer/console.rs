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

#[derive(Debug, Serialize, ToSchema)]
pub struct ConsoleEntry {
    pub id: String,
    pub command: String,
    pub output: String,
    pub exit_code: Option<i32>,
    pub executed_at: String,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConsoleSession {
    pub id: String,
    pub created_at: String,
    pub entries: Vec<ConsoleEntry>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConsoleHistory {
    pub entries: Vec<ConsoleEntry>,
    pub total: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteCommandRequest {
    pub command: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExecuteCommandResponse {
    pub id: String,
    pub command: String,
    pub output: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConsoleSessionList {
    pub sessions: Vec<ConsoleSessionSummary>,
    pub total: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConsoleSessionSummary {
    pub id: String,
    pub created_at: String,
    pub entry_count: usize,
    pub is_active: bool,
}

lazy_static::lazy_static! {
    static ref CONSOLE_SESSIONS: parking_lot::RwLock<Vec<ConsoleSession>> = parking_lot::RwLock::new(Vec::new());
    static ref CURRENT_SESSION: parking_lot::RwLock<Option<String>> = parking_lot::RwLock::new(None);
}

#[derive(Debug, Clone)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub permission: Option<String>,
    pub aliases: Vec<String>,
}

lazy_static::lazy_static! {
    static ref REGISTERED_COMMANDS: parking_lot::RwLock<Vec<PluginCommand>> = parking_lot::RwLock::new(Vec::new());
}

#[utoipa::path(
    post,
    path = "/api/developer/console/execute",
    request_body = ExecuteCommandRequest,
    responses(
        (status = 200, description = "Execute console command", body = ExecuteCommandResponse)
    ),
    tag = "Developer"
)]
pub async fn execute_command(
    State(state): State<AppState>,
    Json(req): Json<ExecuteCommandRequest>,
) -> Result<Json<ExecuteCommandResponse>, crate::error::AppError> {
    let start = std::time::Instant::now();
    let timeout_ms = req.timeout_ms.unwrap_or(30000);
    
    let output = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        state.rcon_client.send_command(&req.command)
    )
    .await
    .map_err(|_| crate::error::AppError::Timeout("Command execution timed out".to_string()))?
    .map_err(|e| crate::error::AppError::RconError(e.to_string()))?;
    
    let duration_ms = start.elapsed().as_millis() as u64;
    
    let entry = ConsoleEntry {
        id: Uuid::new_v4().to_string(),
        command: req.command.clone(),
        output: output.clone(),
        exit_code: None,
        executed_at: Utc::now().to_rfc3339(),
        duration_ms,
    };
    
    let mut sessions = CONSOLE_SESSIONS.write();
    if let Some(current_id) = CURRENT_SESSION.read().clone() {
        if let Some(session) = sessions.iter_mut().find(|s| s.id == current_id) {
            session.entries.push(entry);
        }
    }
    
    Ok(Json(ExecuteCommandResponse {
        id: Uuid::new_v4().to_string(),
        command: req.command,
        output,
        exit_code: None,
        duration_ms,
        success: true,
    }))
}

#[utoipa::path(
    get,
    path = "/api/developer/console/history",
    responses(
        (status = 200, description = "Get console history", body = ConsoleHistory)
    ),
    params(
        ("limit" = Option<usize>, Query, description = "Limit results"),
        ("offset" = Option<usize>, Query, description = "Offset for pagination"),
        ("command" = Option<String>, Query, description = "Filter by command")
    ),
    tag = "Developer"
)]
pub async fn get_console_history(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ConsoleHistory>, crate::error::AppError> {
    let sessions = CONSOLE_SESSIONS.read();
    
    let mut all_entries: Vec<_> = sessions.iter()
        .flat_map(|s| s.entries.iter())
        .cloned()
        .collect();
    
    if let Some(cmd) = params.get("command") {
        all_entries.retain(|e| e.command.contains(cmd));
    }
    
    let offset = params.get("offset")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let limit = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    
    let total = all_entries.len();
    all_entries.sort_by(|a, b| b.executed_at.cmp(&a.executed_at));
    all_entries = all_entries.into_iter().skip(offset).take(limit).collect();
    
    Ok(Json(ConsoleHistory {
        entries: all_entries,
        total,
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/console/sessions",
    responses(
        (status = 201, description = "Create new console session", body = ConsoleSessionSummary)
    ),
    tag = "Developer"
)]
pub async fn create_session() -> Result<Json<ConsoleSessionSummary>, crate::error::AppError> {
    let session = ConsoleSession {
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        entries: Vec::new(),
        is_active: true,
    };
    
    let summary = ConsoleSessionSummary {
        id: session.id.clone(),
        created_at: session.created_at.clone(),
        entry_count: 0,
        is_active: true,
    };
    
    CONSOLE_SESSIONS.write().push(session);
    *CURRENT_SESSION.write() = Some(summary.id.clone());
    
    Ok(Json(summary))
}

#[utoipa::path(
    get,
    path = "/api/developer/console/sessions",
    responses(
        (status = 200, description = "List console sessions", body = ConsoleSessionList)
    ),
    params(
        ("limit" = Option<usize>, Query, description = "Limit results")
    ),
    tag = "Developer"
)]
pub async fn list_sessions(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ConsoleSessionList>, crate::error::AppError> {
    let sessions = CONSOLE_SESSIONS.read();
    
    let limit = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    
    let summaries: Vec<ConsoleSessionSummary> = sessions.iter()
        .rev()
        .take(limit)
        .map(|s| ConsoleSessionSummary {
            id: s.id.clone(),
            created_at: s.created_at.clone(),
            entry_count: s.entries.len(),
            is_active: s.is_active,
        })
        .collect();
    
    Ok(Json(ConsoleSessionList {
        total: sessions.len(),
        sessions: summaries,
    }))
}

#[utoipa::path(
    get,
    path = "/api/developer/console/sessions/{id}",
    responses(
        (status = 200, description = "Get console session details", body = ConsoleSession)
    ),
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    tag = "Developer"
)]
pub async fn get_session(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<ConsoleSession>, crate::error::AppError> {
    let sessions = CONSOLE_SESSIONS.read();
    
    let session = sessions.iter()
        .find(|s| s.id == id)
        .ok_or_else(|| crate::error::AppError::Internal("Session not found".to_string()))?
        .clone();
    
    Ok(Json(session))
}

#[utoipa::path(
    put,
    path = "/api/developer/console/sessions/{id}/activate",
    responses(
        (status = 200, description = "Activate console session")
    ),
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    tag = "Developer"
)]
pub async fn activate_session(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    let sessions = CONSOLE_SESSIONS.read();
    
    if !sessions.iter().any(|s| s.id == id) {
        return Err(crate::error::AppError::Internal("Session not found".to_string()));
    }
    
    *CURRENT_SESSION.write() = Some(id);
    
    Ok(Json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    delete,
    path = "/api/developer/console/sessions/{id}",
    responses(
        (status = 200, description = "Delete console session")
    ),
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    tag = "Developer"
)]
pub async fn delete_session(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    let mut sessions = CONSOLE_SESSIONS.write();
    sessions.retain(|s| s.id != id);
    
    if let Some(current_id) = CURRENT_SESSION.read().clone() {
        if current_id == id {
            *CURRENT_SESSION.write() = None;
        }
    }
    
    Ok(Json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    get,
    path = "/api/developer/console/commands",
    responses(
        (status = 200, description = "List registered plugin commands")
    ),
    tag = "Developer"
)]
pub async fn list_registered_commands() -> Result<impl IntoResponse, crate::error::AppError> {
    let commands = REGISTERED_COMMANDS.read();
    
    Ok(Json(commands.clone()))
}

#[utoipa::path(
    post,
    path = "/api/developer/console/commands",
    request_body = PluginCommand,
    responses(
        (status = 201, description = "Register a plugin command")
    ),
    tag = "Developer"
)]
pub async fn register_command(
    Json(cmd): Json<PluginCommand>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    let mut commands = REGISTERED_COMMANDS.write();
    
    if commands.iter().any(|c| c.name == cmd.name) {
        return Err(crate::error::AppError::Internal("Command already registered".to_string()));
    }
    
    commands.push(cmd);
    
    Ok(Json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    delete,
    path = "/api/developer/console/commands/{name}",
    responses(
        (status = 200, description = "Unregister a plugin command")
    ),
    params(
        ("name" = String, Path, description = "Command name")
    ),
    tag = "Developer"
)]
pub async fn unregister_command(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    let mut commands = REGISTERED_COMMANDS.write();
    commands.retain(|c| c.name != name);
    
    Ok(Json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    get,
    path = "/api/developer/console/cheatsheet",
    responses(
        (status = 200, description = "Get MC server console cheatsheet")
    ),
    tag = "Developer"
)]
pub async fn get_cheatsheet() -> Result<impl IntoResponse, crate::error::AppError> {
    let cheatsheet = vec![
        serde_json::json!({
            "command": "help [page|command]",
            "description": "Display help for all commands or a specific command"
        }),
        serde_json::json!({
            "command": "list",
            "description": "Display list of connected players"
        }),
        serde_json::json!({
            "command": "tp <player> <x> <y> <z>",
            "description": "Teleport player to coordinates"
        }),
        serde_json::json!({
            "command": "give <player> <item> [amount]",
            "description": "Give item to player"
        }),
        serde_json::json!({
            "command": "kick <player> [reason]",
            "description": "Kick player from server"
        }),
        serde_json::json!({
            "command": "ban <player> [reason]",
            "description": "Ban player from server"
        }),
        serde_json::json!({
            "command": "pardon <player>",
            "description": "Unban player"
        }),
        serde_json::json!({
            "command": "op <player>",
            "description": "Make player an operator"
        }),
        serde_json::json!({
            "command": "deop <player>",
            "description": "Remove operator status from player"
        }),
        serde_json::json!({
            "command": "kill <player>",
            "description": "Kill player"
        }),
        serde_json::json!({
            "command": "weather <clear|rain|thunder> [duration]",
            "description": "Set weather"
        }),
        serde_json::json!({
            "command": "time set <day|night|number>",
            "description": "Set game time"
        }),
        serde_json::json!({
            "command": "gamemode <survival|creative|adventure|spectator> [player]",
            "description": "Set player's game mode"
        }),
        serde_json::json!({
            "command": "difficulty <peaceful|easy|normal|hard>",
            "description": "Set game difficulty"
        }),
        serde_json::json!({
            "command": "stop",
            "description": "Stop the server"
        }),
        serde_json::json!({
            "command": "reload",
            "description": "Reload server configuration"
        }),
    ];
    
    Ok(Json(cheatsheet))
}

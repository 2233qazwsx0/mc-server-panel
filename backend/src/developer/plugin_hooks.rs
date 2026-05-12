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
use crate::developer::{DeveloperState, PluginHook};

#[derive(Debug, Serialize, ToSchema)]
pub struct PluginHookInfo {
    pub id: String,
    pub name: String,
    pub hook_type: String,
    pub enabled: bool,
    pub callback_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PluginHookListResponse {
    pub hooks: Vec<PluginHookInfo>,
    pub total: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePluginHookRequest {
    pub name: String,
    pub hook_type: String,
    pub callback_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePluginHookRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReloadResponse {
    pub success: bool,
    pub reloaded_hooks: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HookExecutionResult {
    pub hook_id: String,
    pub success: bool,
    pub response_code: Option<u16>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum HookEventType {
    ServerStart,
    ServerStop,
    PlayerJoin,
    PlayerLeave,
    CommandExecute,
    PluginLoad,
    PluginUnload,
    ConsoleOutput,
}

impl HookEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookEventType::ServerStart => "server.start",
            HookEventType::ServerStop => "server.stop",
            HookEventType::PlayerJoin => "player.join",
            HookEventType::PlayerLeave => "player.leave",
            HookEventType::CommandExecute => "command.execute",
            HookEventType::PluginLoad => "plugin.load",
            HookEventType::PluginUnload => "plugin.unload",
            HookEventType::ConsoleOutput => "console.output",
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/developer/plugins/hooks",
    responses(
        (status = 200, description = "List all plugin hooks", body = PluginHookListResponse)
    ),
    tag = "Developer"
)]
pub async fn list_plugin_hooks(
    State(state): State<AppState>,
) -> Result<Json<PluginHookListResponse>, crate::error::AppError> {
    let hooks = state.developer_state.plugin_hooks.read();
    let hook_infos: Vec<PluginHookInfo> = hooks.values()
        .map(|h| PluginHookInfo {
            id: h.id.clone(),
            name: h.name.clone(),
            hook_type: h.hook_type.clone(),
            enabled: h.enabled,
            callback_url: h.callback_url.clone(),
            created_at: h.created_at.to_rfc3339(),
        })
        .collect();
    
    Ok(Json(PluginHookListResponse {
        total: hook_infos.len(),
        hooks: hook_infos,
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/plugins/hooks",
    request_body = CreatePluginHookRequest,
    responses(
        (status = 201, description = "Create new plugin hook", body = PluginHookInfo)
    ),
    tag = "Developer"
)]
pub async fn create_plugin_hook(
    State(state): State<AppState>,
    Json(req): Json<CreatePluginHookRequest>,
) -> Result<Json<PluginHookInfo>, crate::error::AppError> {
    let hook = PluginHook {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        hook_type: req.hook_type,
        enabled: true,
        callback_url: req.callback_url,
        created_at: Utc::now(),
    };
    
    let info = PluginHookInfo {
        id: hook.id.clone(),
        name: hook.name.clone(),
        hook_type: hook.hook_type.clone(),
        enabled: hook.enabled,
        callback_url: hook.callback_url.clone(),
        created_at: hook.created_at.to_rfc3339(),
    };
    
    state.developer_state.plugin_hooks.write()
        .insert(hook.id.clone(), hook);
    
    Ok(Json(info))
}

#[utoipa::path(
    put,
    path = "/api/developer/plugins/hooks/{id}",
    request_body = UpdatePluginHookRequest,
    responses(
        (status = 200, description = "Update plugin hook", body = PluginHookInfo)
    ),
    params(
        ("id" = String, Path, description = "Hook ID")
    ),
    tag = "Developer"
)]
pub async fn update_plugin_hook(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdatePluginHookRequest>,
) -> Result<Json<PluginHookInfo>, crate::error::AppError> {
    let mut hooks = state.developer_state.plugin_hooks.write();
    
    let hook = hooks.get_mut(&id)
        .ok_or_else(|| crate::error::AppError::Internal("Hook not found".to_string()))?;
    
    if let Some(name) = req.name {
        hook.name = name;
    }
    if let Some(enabled) = req.enabled {
        hook.enabled = enabled;
    }
    if let Some(callback_url) = req.callback_url {
        hook.callback_url = Some(callback_url);
    }
    
    Ok(Json(PluginHookInfo {
        id: hook.id.clone(),
        name: hook.name.clone(),
        hook_type: hook.hook_type.clone(),
        enabled: hook.enabled,
        callback_url: hook.callback_url.clone(),
        created_at: hook.created_at.to_rfc3339(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/developer/plugins/hooks/{id}",
    responses(
        (status = 200, description = "Delete plugin hook")
    ),
    params(
        ("id" = String, Path, description = "Hook ID")
    ),
    tag = "Developer"
)]
pub async fn delete_plugin_hook(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    state.developer_state.plugin_hooks.write()
        .remove(&id);
    
    Ok(Json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    post,
    path = "/api/developer/plugins/reload",
    responses(
        (status = 200, description = "Reload plugin hooks", body = ReloadResponse)
    ),
    tag = "Developer"
)]
pub async fn reload_plugin_hooks(
    State(state): State<AppState>,
) -> Result<Json<ReloadResponse>, crate::error::AppError> {
    let hooks = state.developer_state.plugin_hooks.read().clone();
    let mut reloaded = Vec::new();
    let mut errors = Vec::new();
    
    for (_, hook) in hooks.iter() {
        if hook.enabled {
            if let Some(ref url) = hook.callback_url {
                match reqwest::get(url).await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            reloaded.push(hook.name.clone());
                        } else {
                            errors.push(format!("{}: HTTP {}", hook.name, resp.status()));
                        }
                    }
                    Err(e) => {
                        errors.push(format!("{}: {}", hook.name, e));
                    }
                }
            } else {
                reloaded.push(hook.name.clone());
            }
        }
    }
    
    Ok(Json(ReloadResponse {
        success: errors.is_empty(),
        reloaded_hooks: reloaded,
        errors,
    }))
}

pub async fn trigger_hook(state: &DeveloperState, event_type: HookEventType, payload: serde_json::Value) -> Vec<HookExecutionResult> {
    let mut results = Vec::new();
    let hooks = state.plugin_hooks.read();
    
    let event_name = event_type.as_str();
    
    for (_, hook) in hooks.iter() {
        if hook.enabled && hook.hook_type == event_name {
            let start = std::time::Instant::now();
            
            if let Some(ref url) = hook.callback_url {
                match reqwest::Client::new()
                    .post(url)
                    .json(&payload)
                    .send()
                    .await
                {
                    Ok(resp) => {
                        results.push(HookExecutionResult {
                            hook_id: hook.id.clone(),
                            success: resp.status().is_success(),
                            response_code: Some(resp.status().as_u16()),
                            error: None,
                            duration_ms: start.elapsed().as_millis() as u64,
                        });
                    }
                    Err(e) => {
                        results.push(HookExecutionResult {
                            hook_id: hook.id.clone(),
                            success: false,
                            response_code: None,
                            error: Some(e.to_string()),
                            duration_ms: start.elapsed().as_millis() as u64,
                        });
                    }
                }
            }
        }
    }
    
    results
}

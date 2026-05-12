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

use crate::state::AppState;
use crate::developer::{DeveloperState, EventSubscriber};

#[derive(Debug, Serialize, ToSchema)]
pub struct EventSubscriptionInfo {
    pub id: String,
    pub name: String,
    pub event_type: String,
    pub callback_url: String,
    pub enabled: bool,
    pub has_secret: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventSubscriptionList {
    pub subscriptions: Vec<EventSubscriptionInfo>,
    pub total: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSubscriptionRequest {
    pub name: String,
    pub event_type: String,
    pub callback_url: String,
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSubscriptionRequest {
    pub name: Option<String>,
    pub callback_url: Option<String>,
    pub secret: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EventPayload {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventDeliveryResult {
    pub subscription_id: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventHistory {
    pub events: Vec<EventHistoryEntry>,
    pub total: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventHistoryEntry {
    pub id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub delivered_to: Vec<String>,
    pub timestamp: String,
}

lazy_static::lazy_static! {
    static ref EVENT_HISTORY: parking_lot::RwLock<Vec<EventHistoryEntry>> = parking_lot::RwLock::new(Vec::new());
}

#[derive(Debug, Clone)]
pub struct EventBus {
    pub handlers: HashMap<String, Vec<Box<dyn Fn(EventPayload) + Send + Sync>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    pub fn subscribe<F>(&mut self, event_type: String, handler: F)
    where
        F: Fn(EventPayload) + Send + Sync + 'static
    {
        self.handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }
    
    pub fn publish(&self, event_type: &str, payload: EventPayload) -> Vec<EventDeliveryResult> {
        let mut results = Vec::new();
        
        if let Some(handlers) = self.handlers.get(event_type) {
            for handler in handlers {
                let start = std::time::Instant::now();
                handler(payload.clone());
                results.push(EventDeliveryResult {
                    subscription_id: "local".to_string(),
                    success: true,
                    status_code: None,
                    error: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }
        
        results
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[utoipa::path(
    get,
    path = "/api/developer/events/subscriptions",
    responses(
        (status = 200, description = "List event subscriptions", body = EventSubscriptionList)
    ),
    params(
        ("event_type" = Option<String>, Query, description = "Filter by event type")
    ),
    tag = "Developer"
)]
pub async fn list_subscriptions(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<EventSubscriptionList>, crate::error::AppError> {
    let subscribers = state.developer_state.event_subscribers.read();
    
    let mut subs: Vec<_> = subscribers.values().collect();
    
    if let Some(event_type) = params.get("event_type") {
        subs.retain(|s| s.event_type == *event_type);
    }
    
    let infos: Vec<EventSubscriptionInfo> = subs.iter()
        .map(|s| EventSubscriptionInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            event_type: s.event_type.clone(),
            callback_url: s.callback_url.clone(),
            enabled: s.enabled,
            has_secret: s.secret.is_some(),
            created_at: s.created_at.to_rfc3339(),
        })
        .collect();
    
    Ok(Json(EventSubscriptionList {
        total: infos.len(),
        subscriptions: infos,
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/events/subscribe",
    request_body = CreateSubscriptionRequest,
    responses(
        (status = 201, description = "Create event subscription", body = EventSubscriptionInfo)
    ),
    tag = "Developer"
)]
pub async fn create_subscription(
    State(state): State<AppState>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<Json<EventSubscriptionInfo>, crate::error::AppError> {
    let subscriber = EventSubscriber {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        event_type: req.event_type,
        callback_url: req.callback_url,
        secret: req.secret,
        enabled: true,
        created_at: Utc::now(),
    };
    
    let info = EventSubscriptionInfo {
        id: subscriber.id.clone(),
        name: subscriber.name.clone(),
        event_type: subscriber.event_type.clone(),
        callback_url: subscriber.callback_url.clone(),
        enabled: subscriber.enabled,
        has_secret: subscriber.secret.is_some(),
        created_at: subscriber.created_at.to_rfc3339(),
    };
    
    state.developer_state.event_subscribers.write()
        .insert(subscriber.id.clone(), subscriber);
    
    Ok(Json(info))
}

#[utoipa::path(
    put,
    path = "/api/developer/events/subscribe/{id}",
    request_body = UpdateSubscriptionRequest,
    responses(
        (status = 200, description = "Update event subscription", body = EventSubscriptionInfo)
    ),
    params(
        ("id" = String, Path, description = "Subscription ID")
    ),
    tag = "Developer"
)]
pub async fn update_subscription(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdateSubscriptionRequest>,
) -> Result<Json<EventSubscriptionInfo>, crate::error::AppError> {
    let mut subscribers = state.developer_state.event_subscribers.write();
    
    let sub = subscribers.get_mut(&id)
        .ok_or_else(|| crate::error::AppError::Internal("Subscription not found".to_string()))?;
    
    if let Some(name) = req.name {
        sub.name = name;
    }
    if let Some(url) = req.callback_url {
        sub.callback_url = url;
    }
    if let Some(secret) = req.secret {
        sub.secret = Some(secret);
    }
    if let Some(enabled) = req.enabled {
        sub.enabled = enabled;
    }
    
    Ok(Json(EventSubscriptionInfo {
        id: sub.id.clone(),
        name: sub.name.clone(),
        event_type: sub.event_type.clone(),
        callback_url: sub.callback_url.clone(),
        enabled: sub.enabled,
        has_secret: sub.secret.is_some(),
        created_at: sub.created_at.to_rfc3339(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/developer/events/subscribe/{id}",
    responses(
        (status = 200, description = "Delete event subscription")
    ),
    params(
        ("id" = String, Path, description = "Subscription ID")
    ),
    tag = "Developer"
)]
pub async fn delete_subscription(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    state.developer_state.event_subscribers.write()
        .remove(&id);
    
    Ok(Json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    post,
    path = "/api/developer/events/unsubscribe",
    request_body = HashMap<String, String>,
    responses(
        (status = 200, description = "Unsubscribe from event")
    ),
    params(
        ("id" = String, Query, description = "Subscription ID")
    ),
    tag = "Developer"
)]
pub async fn unsubscribe(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    if let Some(id) = params.get("id") {
        state.developer_state.event_subscribers.write()
            .remove(id);
    }
    
    Ok(Json(serde_json::json!({ "success": true })))
}

#[utoipa::path(
    post,
    path = "/api/developer/events/publish",
    request_body = EventPayload,
    responses(
        (status = 200, description = "Publish event to subscribers", body = Vec<EventDeliveryResult>)
    ),
    tag = "Developer"
)]
pub async fn publish_event(
    State(state): State<AppState>,
    Json(payload): Json<EventPayload>,
) -> Result<Json<Vec<EventDeliveryResult>>, crate::error::AppError> {
    let subscribers = state.developer_state.event_subscribers.read();
    
    let mut results = Vec::new();
    let mut delivered_to = Vec::new();
    
    for sub in subscribers.values() {
        if sub.event_type == payload.event_type && sub.enabled {
            let start = std::time::Instant::now();
            
            let mut payload_with_sig = payload.clone();
            if let Some(ref secret) = sub.secret {
                let sig = crate::developer::webhook::compute_signature_sha1(
                    secret,
                    &serde_json::to_string(&payload.data).unwrap_or_default(),
                    None
                );
                payload_with_sig.signature = Some(sig);
            }
            
            match reqwest::Client::new()
                .post(&sub.callback_url)
                .json(&payload_with_sig)
                .send()
                .await
            {
                Ok(resp) => {
                    results.push(EventDeliveryResult {
                        subscription_id: sub.id.clone(),
                        success: resp.status().is_success(),
                        status_code: Some(resp.status().as_u16()),
                        error: None,
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                    delivered_to.push(sub.id.clone());
                }
                Err(e) => {
                    results.push(EventDeliveryResult {
                        subscription_id: sub.id.clone(),
                        success: false,
                        status_code: None,
                        error: Some(e.to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                }
            }
        }
    }
    
    EVENT_HISTORY.write().push(EventHistoryEntry {
        id: Uuid::new_v4().to_string(),
        event_type: payload.event_type.clone(),
        payload: payload.data.clone(),
        delivered_to,
        timestamp: Utc::now().to_rfc3339(),
    });
    
    Ok(Json(results))
}

#[utoipa::path(
    get,
    path = "/api/developer/events/history",
    responses(
        (status = 200, description = "Get event history", body = EventHistory)
    ),
    params(
        ("event_type" = Option<String>, Query, description = "Filter by event type"),
        ("limit" = Option<usize>, Query, description = "Limit results"),
        ("offset" = Option<usize>, Query, description = "Offset for pagination")
    ),
    tag = "Developer"
)]
pub async fn get_event_history(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<EventHistory>, crate::error::AppError> {
    let history = EVENT_HISTORY.read();
    
    let mut events: Vec<_> = history.iter().collect();
    
    if let Some(event_type) = params.get("event_type") {
        events.retain(|e| e.event_type == *event_type);
    }
    
    let offset = params.get("offset")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let limit = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    
    let total = events.len();
    events = events.into_iter().rev().skip(offset).take(limit).cloned().collect();
    
    Ok(Json(EventHistory {
        events: events.into_iter().cloned().collect(),
        total,
    }))
}

#[utoipa::path(
    get,
    path = "/api/developer/events/types",
    responses(
        (status = 200, description = "List available event types")
    ),
    tag = "Developer"
)]
pub async fn list_event_types() -> Result<impl IntoResponse, crate::error::AppError> {
    let event_types = vec![
        serde_json::json!({
            "id": "server.start",
            "name": "Server Start",
            "description": "Fires when the Minecraft server starts"
        }),
        serde_json::json!({
            "id": "server.stop",
            "name": "Server Stop",
            "description": "Fires when the Minecraft server stops"
        }),
        serde_json::json!({
            "id": "player.join",
            "name": "Player Join",
            "description": "Fires when a player joins the server"
        }),
        serde_json::json!({
            "id": "player.leave",
            "name": "Player Leave",
            "description": "Fires when a player leaves the server"
        }),
        serde_json::json!({
            "id": "command.execute",
            "name": "Command Execute",
            "description": "Fires when a command is executed"
        }),
        serde_json::json!({
            "id": "plugin.load",
            "name": "Plugin Load",
            "description": "Fires when a plugin is loaded"
        }),
        serde_json::json!({
            "id": "plugin.unload",
            "name": "Plugin Unload",
            "description": "Fires when a plugin is unloaded"
        }),
        serde_json::json!({
            "id": "console.output",
            "name": "Console Output",
            "description": "Fires on new console output"
        }),
    ];
    
    Ok(Json(event_types))
}

pub async fn trigger_event(state: &DeveloperState, event_type: &str, data: serde_json::Value) -> Vec<EventDeliveryResult> {
    let payload = EventPayload {
        event_type: event_type.to_string(),
        data,
        timestamp: Utc::now().to_rfc3339(),
        signature: None,
    };
    
    let subscribers = state.event_subscribers.read();
    let mut results = Vec::new();
    
    for sub in subscribers.values() {
        if sub.event_type == event_type && sub.enabled {
            let start = std::time::Instant::now();
            
            let mut payload_with_sig = payload.clone();
            if let Some(ref secret) = sub.secret {
                let sig = crate::developer::webhook::compute_signature_sha1(
                    secret,
                    &serde_json::to_string(&payload.data).unwrap_or_default(),
                    None
                );
                payload_with_sig.signature = Some(sig);
            }
            
            match reqwest::Client::new()
                .post(&sub.callback_url)
                .json(&payload_with_sig)
                .send()
                .await
            {
                Ok(resp) => {
                    results.push(EventDeliveryResult {
                        subscription_id: sub.id.clone(),
                        success: resp.status().is_success(),
                        status_code: Some(resp.status().as_u16()),
                        error: None,
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    results.push(EventDeliveryResult {
                        subscription_id: sub.id.clone(),
                        success: false,
                        status_code: None,
                        error: Some(e.to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                }
            }
        }
    }
    
    results
}

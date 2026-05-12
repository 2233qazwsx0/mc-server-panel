pub mod openapi;
pub mod websocket_debugger;
pub mod plugin_hooks;
pub mod request_logger;
pub mod profiler;
pub mod sdk_generator;
pub mod webhook;
pub mod rate_limiter;
pub mod events;
pub mod console;

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub use openapi::*;
pub use websocket_debugger::*;
pub use plugin_hooks::*;
pub use request_logger::*;
pub use profiler::*;
pub use sdk_generator::*;
pub use webhook::*;
pub use rate_limiter::*;
pub use events::*;
pub use console::*;

#[derive(Clone)]
pub struct DeveloperState {
    pub request_logs: Arc<RwLock<Vec<RequestLogEntry>>>,
    pub rate_limiter: Arc<RwLock<RateLimiter>>,
    pub event_subscribers: Arc<RwLock<HashMap<String, Vec<EventSubscriber>>>>,
    pub ws_debug_sessions: Arc<RwLock<HashMap<String, WsDebugSession>>>,
    pub plugin_hooks: Arc<RwLock<HashMap<String, PluginHook>>>,
    pub profiler_snapshots: Arc<RwLock<Vec<ProfilerSnapshot>>>,
}

impl DeveloperState {
    pub fn new() -> Self {
        Self {
            request_logs: Arc::new(RwLock::new(Vec::new())),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            event_subscribers: Arc::new(RwLock::new(HashMap::new())),
            ws_debug_sessions: Arc::new(RwLock::new(HashMap::new())),
            plugin_hooks: Arc::new(RwLock::new(HashMap::new())),
            profiler_snapshots: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for DeveloperState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestLogEntry {
    pub id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub client_ip: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WsDebugSession {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub messages: Vec<WsMessageEntry>,
    pub is_active: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WsMessageEntry {
    pub id: String,
    pub direction: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginHook {
    pub id: String,
    pub name: String,
    pub hook_type: String,
    pub enabled: bool,
    pub callback_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfilerSnapshot {
    pub id: String,
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ns: u64,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventSubscriber {
    pub id: String,
    pub name: String,
    pub event_type: String,
    pub callback_url: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl WsDebugSession {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            messages: Vec::new(),
            is_active: true,
        }
    }
}

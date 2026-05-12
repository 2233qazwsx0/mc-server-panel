use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;

use crate::cluster::types::*;

#[derive(Clone)]
pub struct ProxyManager {
    state: Arc<RwLock<ProxyState>>,
}

#[derive(Default)]
struct ProxyState {
    config: ProxyConfig,
    servers: Vec<ManagedServer>,
    sessions: Vec<PlayerSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedServer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub motd: String,
    pub player_count: u32,
    pub max_players: u32,
    pub online: bool,
    pub priority: u32,
    pub hidden: bool,
    pub restricted: bool,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSession {
    pub id: String,
    pub player_name: String,
    pub player_uuid: String,
    pub current_server: String,
    pub original_server: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: String,
    pub version: String,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ProxyState::default())),
        }
    }

    pub async fn get_config(&self) -> ProxyConfig {
        self.state.read().await.config.clone()
    }

    pub async fn update_config(&self, config: ProxyConfig) -> Result<(), ProxyError> {
        self.validate_config(&config)?;
        self.state.write().await.config = config;
        Ok(())
    }

    pub async fn register_server(&self, entry: ServerEntry) -> Result<ManagedServer, ProxyError> {
        let state = &mut *self.state.write().await;
        if state.servers.iter().any(|s| s.name == entry.name) {
            return Err(ProxyError::ServerAlreadyRegistered(entry.name));
        }

        let server = ManagedServer {
            id: Uuid::new_v4().to_string(),
            name: entry.name,
            address: entry.address,
            port: 25565,
            motd: entry.motd,
            player_count: 0,
            max_players: 100,
            online: false,
            priority: entry.priority,
            hidden: entry.hidden,
            restricted: entry.restricted,
            registered_at: Utc::now(),
        };

        state.servers.push(server.clone());
        Ok(server)
    }

    pub async fn unregister_server(&self, name: &str) -> Result<(), ProxyError> {
        let state = &mut *self.state.write().await;
        let pos = state.servers.iter().position(|s| s.name == name)
            .ok_or(ProxyError::ServerNotFound(name.to_string()))?;

        state.servers.remove(pos);
        Ok(())
    }

    pub async fn list_servers(&self) -> Vec<ManagedServer> {
        self.state.read().await.servers.clone()
    }

    pub async fn get_server(&self, name: &str) -> Option<ManagedServer> {
        self.state.read().await.servers.iter()
            .find(|s| s.name == name)
            .cloned()
    }

    pub async fn update_server_status(&self, name: &str, online: bool) -> Result<(), ProxyError> {
        let state = &mut *self.state.write().await;
        let server = state.servers.iter_mut()
            .find(|s| s.name == name)
            .ok_or(ProxyError::ServerNotFound(name.to_string()))?;
        server.online = online;
        Ok(())
    }

    pub async fn update_server_player_count(&self, name: &str, count: u32) -> Result<(), ProxyError> {
        let state = &mut *self.state.write().await;
        let server = state.servers.iter_mut()
            .find(|s| s.name == name)
            .ok_or(ProxyError::ServerNotFound(name.to_string()))?;
        server.player_count = count;
        Ok(())
    }

    pub async fn create_player_session(&self, player_name: &str, server: &str) -> PlayerSession {
        let state = &mut *self.state.write().await;
        let session = PlayerSession {
            id: Uuid::new_v4().to_string(),
            player_name: player_name.to_string(),
            player_uuid: Uuid::new_v4().to_string(),
            current_server: server.to_string(),
            original_server: None,
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            ip_address: "0.0.0.0".to_string(),
            version: "1.21".to_string(),
        };
        state.sessions.push(session.clone());
        session
    }

    pub async fn transfer_player(&self, player_name: &str, target_server: &str) -> Result<(), ProxyError> {
        let state = &mut *self.state.write().await;
        if !state.servers.iter().any(|s| s.name == target_server) {
            return Err(ProxyError::ServerNotFound(target_server.to_string()));
        }

        if let Some(session) = state.sessions.iter_mut().find(|s| s.player_name == player_name) {
            session.original_server = Some(session.current_server.clone());
            session.current_server = target_server.to_string();
            session.last_activity = Utc::now();
            Ok(())
        } else {
            Err(ProxyError::PlayerNotFound(player_name.to_string()))
        }
    }

    pub async fn remove_player_session(&self, player_name: &str) -> Result<(), ProxyError> {
        let state = &mut *self.state.write().await;
        let pos = state.sessions.iter().position(|s| s.player_name == player_name)
            .ok_or(ProxyError::PlayerNotFound(player_name.to_string()))?;
        state.sessions.remove(pos);
        Ok(())
    }

    pub async fn get_player_session(&self, player_name: &str) -> Option<PlayerSession> {
        self.state.read().await.sessions.iter()
            .find(|s| s.player_name == player_name)
            .cloned()
    }

    pub async fn list_connected_players(&self) -> Vec<PlayerSession> {
        self.state.read().await.sessions.clone()
    }

    pub async fn get_online_players(&self) -> u32 {
        self.state.read().await.sessions.len() as u32
    }

    pub async fn get_total_slots(&self) -> u32 {
        let state = self.state.read().await;
        state.servers.iter()
            .filter(|s| !s.hidden)
            .map(|s| s.max_players)
            .sum()
    }

    pub async fn generate_velocity_forwarding_token(&self, secret: &str, expiry_secs: u64) -> String {
        let expiry = Utc::now().timestamp() + expiry_secs as i64;
        format!("velocity-token:{}:{}", expiry, secret)
    }

    pub async fn validate_velocity_config(&self, public_key: &str) -> Result<(), ProxyError> {
        if public_key.is_empty() {
            return Err(ProxyError::InvalidConfig("Public key is required for Velocity".to_string()));
        }
        Ok(())
    }

    pub async fn generate_bungee_uuid(&self) -> String {
        Uuid::new_v4().to_string()
    }

    fn validate_config(&self, config: &ProxyConfig) -> Result<(), ProxyError> {
        if config.port < 1024 || config.port > 65535 {
            return Err(ProxyError::InvalidPort(config.port));
        }
        if config.max_players == 0 {
            return Err(ProxyError::InvalidConfig("Max players must be greater than 0".to_string()));
        }
        if matches!(config.proxy_type, ProxyType::Velocity) && config.ip_forward {
            tracing::warn!("IP forward enabled on Velocity - ensure servers support it");
        }
        Ok(())
    }
}

impl Default for ProxyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Server {0} is already registered")]
    ServerAlreadyRegistered(String),

    #[error("Server {0} not found")]
    ServerNotFound(String),

    #[error("Player {0} not found")]
    PlayerNotFound(String),

    #[error("Invalid port: {0}")]
    InvalidPort(u16),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    pub online_players: u32,
    pub max_players: u32,
    pub total_servers: u32,
    pub online_servers: u32,
    pub uptime_secs: u64,
    pub proxy_type: ProxyType,
    pub version: String,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Log {
        content: String,
        level: String,
        timestamp: DateTime<Utc>,
        source: String,
    },
    Metrics {
        cpu_usage: f32,
        memory_percent: f32,
        memory_used: u64,
        memory_total: u64,
        timestamp: DateTime<Utc>,
    },
    Status {
        running: bool,
        pid: Option<u32>,
        players_online: u32,
        tps: Option<f64>,
        timestamp: DateTime<Utc>,
    },
    PlayerList {
        players: Vec<PlayerInfo>,
        timestamp: DateTime<Utc>,
    },
    Pong,
    Error {
        message: String,
        code: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub name: String,
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    Ping,
    Subscribe {
        channel: String,
    },
    Unsubscribe {
        channel: String,
    },
}

impl WsMessage {
    pub fn log(content: &str, level: &str, source: &str) -> Self {
        Self::Log {
            content: content.to_string(),
            level: level.to_string(),
            timestamp: Utc::now(),
            source: source.to_string(),
        }
    }

    pub fn metrics(
        cpu_usage: f32,
        memory_percent: f32,
        memory_used: u64,
        memory_total: u64,
    ) -> Self {
        Self::Metrics {
            cpu_usage,
            memory_percent,
            memory_used,
            memory_total,
            timestamp: Utc::now(),
        }
    }

    pub fn status(running: bool, pid: Option<u32>, players_online: u32, tps: Option<f64>) -> Self {
        Self::Status {
            running,
            pid,
            players_online,
            tps,
            timestamp: Utc::now(),
        }
    }

    pub fn player_list(players: Vec<PlayerInfo>) -> Self {
        Self::PlayerList {
            players,
            timestamp: Utc::now(),
        }
    }

    pub fn pong() -> Self {
        Self::Pong
    }

    pub fn error(message: &str, code: &str) -> Self {
        Self::Error {
            message: message.to_string(),
            code: code.to_string(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| r#"{"type":"error","message":"serialization failed","code":"internal"}"#.to_string())
    }
}

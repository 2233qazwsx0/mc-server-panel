use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::cluster::types::*;

#[derive(Clone)]
pub struct ChatSyncManager {
    state: Arc<ChatSyncState>,
    config: Arc<RwLock<ChatSyncConfig>>,
}

struct ChatSyncState {
    channels: RwLock<HashMap<String, ChatChannel>>,
    messages: RwLock<Vec<ChatMessage>>,
    player_channels: RwLock<HashMap<String, String>>,
    cooldowns: RwLock<HashMap<String, DateTime<Utc>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChannel {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub color: String,
    pub range_blocks: Option<u32>,
    pub cross_server: bool,
    pub staff_only: bool,
    pub muted_players: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub channel: String,
    pub sender_name: String,
    pub sender_uuid: String,
    pub sender_server: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub recipient_server: Option<String>,
    pub message_type: ChatMessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatMessageType {
    Global,
    Local,
    Private,
    Staff,
    Channel,
}

impl ChatSyncManager {
    pub fn new(config: ChatSyncConfig) -> Self {
        Self {
            state: Arc::new(ChatSyncState {
                channels: RwLock::new(HashMap::new()),
                messages: RwLock::new(Vec::new()),
                player_channels: RwLock::new(HashMap::new()),
                cooldowns: RwLock::new(HashMap::new()),
            }),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn get_config(&self) -> ChatSyncConfig {
        self.config.read().clone()
    }

    pub fn update_config(&self, config: ChatSyncConfig) {
        *self.config.write() = config;
    }

    pub fn create_channel(&self, name: String, prefix: String, color: String) -> ChatChannel {
        let channel = ChatChannel {
            id: Uuid::new_v4().to_string(),
            name,
            prefix,
            color,
            range_blocks: None,
            cross_server: true,
            staff_only: false,
            muted_players: Vec::new(),
        };

        let mut channels = self.state.channels.write();
        channels.insert(channel.id.clone(), channel.clone());
        channel
    }

    pub fn delete_channel(&self, channel_id: &str) -> bool {
        self.state.channels.write().remove(channel_id).is_some()
    }

    pub fn get_channels(&self) -> Vec<ChatChannel> {
        self.state.channels.read().values().cloned().collect()
    }

    pub fn set_player_channel(&self, player_id: &str, channel_id: &str) -> Result<(), ChatError> {
        let channels = self.state.channels.read();
        if !channels.contains_key(channel_id) {
            return Err(ChatError::ChannelNotFound(channel_id.to_string()));
        }
        drop(channels);

        let mut player_channels = self.state.player_channels.write();
        player_channels.insert(player_id.to_string(), channel_id.to_string());
        Ok(())
    }

    pub fn get_player_channel(&self, player_id: &str) -> Option<String> {
        self.state.player_channels.read().get(player_id).cloned()
    }

    pub fn mute_player(&self, channel_id: &str, player_id: &str) -> Result<(), ChatError> {
        let mut channels = self.state.channels.write();
        let channel = channels.get_mut(channel_id)
            .ok_or_else(|| ChatError::ChannelNotFound(channel_id.to_string()))?;

        if !channel.muted_players.contains(&player_id.to_string()) {
            channel.muted_players.push(player_id.to_string());
        }
        Ok(())
    }

    pub fn unmute_player(&self, channel_id: &str, player_id: &str) -> Result<(), ChatError> {
        let mut channels = self.state.channels.write();
        let channel = channels.get_mut(channel_id)
            .ok_or_else(|| ChatError::ChannelNotFound(channel_id.to_string()))?;

        channel.muted_players.retain(|p| p != player_id);
        Ok(())
    }

    pub fn send_message(&self, sender_name: &str, sender_uuid: &str, sender_server: &str, content: &str) -> Result<ChatMessage, ChatError> {
        let config = self.config.read();

        if !config.enabled {
            return Err(ChatError::ChatSyncDisabled);
        }

        let channel_id = self.state.player_channels.read()
            .get(sender_uuid)
            .cloned()
            .unwrap_or_else(|| "global".to_string());

        let mut channels = self.state.channels.write();
        let channel = channels.get_mut(&channel_id)
            .ok_or_else(|| ChatError::ChannelNotFound(channel_id.clone()))?;

        if channel.muted_players.contains(&sender_uuid.to_string()) {
            return Err(ChatError::PlayerMuted(channel_id.clone()));
        }

        if let Some(last_time) = self.state.cooldowns.read().get(sender_uuid) {
            let elapsed = Utc::now() - *last_time;
            if elapsed.num_milliseconds() < config.cooldown_ms as i64 {
                return Err(ChatError::CooldownActive);
            }
        }

        let msg_type = if channel.cross_server {
            ChatMessageType::Global
        } else {
            ChatMessageType::Local
        };

        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            channel: channel_id.clone(),
            sender_name: sender_name.to_string(),
            sender_uuid: sender_uuid.to_string(),
            sender_server: sender_server.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            recipient_server: if channel.cross_server { None } else { Some(sender_server.to_string()) },
            message_type: msg_type,
        };

        let mut messages = self.state.messages.write();
        messages.push(message.clone());

        if messages.len() > 1000 {
            messages.drain(0..500);
        }

        *self.state.cooldowns.write().entry(sender_uuid.to_string()).or_insert(Utc::now()) = Utc::now();

        drop(channels);
        drop(messages);

        Ok(message)
    }

    pub fn send_private_message(&self, from_name: &str, from_uuid: &str, to_name: &str, to_uuid: &str, content: &str) -> Result<ChatMessage, ChatError> {
        if !self.config.read().private_messages_enabled {
            return Err(ChatError::PrivateMessagesDisabled);
        }

        let message = ChatMessage {
            id: Uuid::new_v4().to_string(),
            channel: "private".to_string(),
            sender_name: from_name.to_string(),
            sender_uuid: from_uuid.to_string(),
            sender_server: "proxy".to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            recipient_server: Some(to_uuid.to_string()),
            message_type: ChatMessageType::Private,
        };

        self.state.messages.write().push(message.clone());
        Ok(message)
    }

    pub fn get_recent_messages(&self, limit: usize, channel_id: Option<&str>) -> Vec<ChatMessage> {
        let messages = self.state.messages.read();

        match channel_id {
            Some(ch) => messages.iter().rev()
                .filter(|m| m.channel == ch)
                .take(limit)
                .cloned()
                .collect(),
            None => messages.iter().rev().take(limit).cloned().collect(),
        }
    }

    pub fn format_for_proxy(&self, message: &ChatMessage) -> String {
        let config = self.config.read();
        config.proxy_chat_format
            .replace("{server}", &message.sender_server)
            .replace("{display_name}", &message.sender_name)
            .replace("{message}", &message.content)
            .replace("{channel}", &message.channel)
    }

    pub fn get_stats(&self) -> ChatStats {
        let messages = self.state.messages.read();
        let channels = self.state.channels.read();
        let total_messages = messages.len();
        let total_channels = channels.len();

        ChatStats {
            total_messages,
            total_channels,
            recent_messages_count: messages.iter()
                .filter(|m| Utc::now() - m.timestamp < chrono::Duration::minutes(5))
                .count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStats {
    pub total_messages: usize,
    pub total_channels: usize,
    pub recent_messages_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("Chat sync is disabled")]
    ChatSyncDisabled,

    #[error("Channel {0} not found")]
    ChannelNotFound(String),

    #[error("Player is muted in channel {0}")]
    PlayerMuted(String),

    #[error("Cooldown active, please wait")]
    CooldownActive,

    #[error("Private messages are disabled")]
    PrivateMessagesDisabled,
}

use crate::monitor::types::Alert;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum WebhookError {
    #[error("Webhook not found: {0}")]
    NotFound(String),
    #[error("Failed to send webhook: {0}")]
    SendFailed(String),
    #[error("Invalid webhook configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub webhook_type: WebhookType,
    pub enabled: bool,
    pub secret: Option<String>,
    pub headers: HashMap<String, String>,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
    pub timeout_secs: u64,
    pub filters: WebhookFilters,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookType {
    Discord,
    Slack,
    Teams,
    Telegram,
    Email,
    Generic,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFilters {
    pub alert_levels: Option<Vec<String>>,
    pub metric_types: Option<Vec<String>>,
    pub min_value: Option<f64>,
    pub pattern: Option<String>,
}

impl Default for WebhookFilters {
    fn default() -> Self {
        Self {
            alert_levels: None,
            metric_types: None,
            min_value: None,
            pattern: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: String,
    pub webhook_type: WebhookType,
    pub alert: Option<Alert>,
    pub message: Option<String>,
    pub data: Option<HashMap<String, serde_json::Value>>,
    pub timestamp: DateTime<Utc>,
}

impl WebhookPayload {
    pub fn from_alert(webhook_type: WebhookType, alert: Alert) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            webhook_type,
            alert: Some(alert),
            message: None,
            data: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_message(webhook_type: WebhookType, message: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            webhook_type,
            alert: None,
            message: Some(message),
            data: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordWebhook {
    pub content: Option<String>,
    pub embeds: Vec<DiscordEmbed>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordEmbed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub color: u32,
    pub fields: Vec<DiscordField>,
    pub footer: Option<DiscordFooter>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordFooter {
    pub text: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackWebhook {
    pub text: String,
    pub blocks: Vec<SlackBlock>,
    pub attachments: Vec<SlackAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<SlackText>,
    pub elements: Option<Vec<SlackElement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackText {
    #[serde(rename = "type")]
    pub text_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub text: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackAttachment {
    pub color: String,
    pub blocks: Vec<SlackBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsWebhook {
    #[serde(rename = "@type")]
    pub msg_type: String,
    #[serde(rename = "@context")]
    pub context: String,
    pub theme_color: String,
    pub summary: String,
    pub sections: Vec<TeamsSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsSection {
    pub activity_title: String,
    pub activity_subtitle: Option<String>,
    pub facts: Vec<TeamsFact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsFact {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub chat_id: String,
    pub text: String,
    pub parse_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliveryResult {
    pub webhook_id: String,
    pub payload_id: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub delivered_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct WebhookNotifier {
    client: Client,
    webhooks: Arc<RwLock<HashMap<String, WebhookConfig>>>,
    delivery_history: Arc<RwLock<VecDeque<WebhookDeliveryResult>>>,
    #[allow(dead_code)]
    sender: mpsc::Sender<(WebhookConfig, WebhookPayload)>,
    max_history_size: usize,
}

impl WebhookNotifier {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let (sender, _receiver) = mpsc::channel::<(WebhookConfig, WebhookPayload)>(100);

        Self {
            client,
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            delivery_history: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            sender,
            max_history_size: 1000,
        }
    }

    pub async fn start_processor(self: Arc<Self>) {
        let (tx, mut rx) = mpsc::channel::<(WebhookConfig, WebhookPayload)>(100);
        let webhooks = self.webhooks.clone();
        let history = self.delivery_history.clone();
        let client = self.client.clone();

        let notifier = Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            webhooks: Arc::new(RwLock::new(HashMap::new())),
            delivery_history: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            sender: tx,
            max_history_size: 1000,
        };
        let _ = notifier;

        tokio::spawn(async move {
            while let Some((config, payload)) = rx.recv().await {
                let result = Self::send_webhook_internal(&client, &config, &payload).await;

                let mut delivery_history = history.write();
                if delivery_history.len() >= 1000 {
                    delivery_history.pop_front();
                }
                delivery_history.push_back(result);
            }
        });
    }

    pub async fn send_webhook(&self, webhook_id: &str, payload: WebhookPayload) -> Result<WebhookDeliveryResult, WebhookError> {
        let webhooks = self.webhooks.read();
        let config = webhooks.get(webhook_id).ok_or_else(|| {
            WebhookError::NotFound(webhook_id.to_string())
        })?;

        if !config.enabled {
            return Err(WebhookError::InvalidConfig("Webhook is disabled".to_string()));
        }

        let result = Self::send_webhook_internal(&self.client, config, &payload).await;

        let mut history = self.delivery_history.write();
        if history.len() >= self.max_history_size {
            history.pop_front();
        }
        history.push_back(result.clone());

        if !result.success {
            Err(WebhookError::SendFailed(result.error_message.clone().unwrap_or_default()))
        } else {
            Ok(result)
        }
    }

    async fn send_webhook_internal(
        client: &Client,
        config: &WebhookConfig,
        payload: &WebhookPayload,
    ) -> WebhookDeliveryResult {
        let body = match config.webhook_type {
            WebhookType::Discord => {
                serde_json::to_string(&Self::build_discord_payload(payload)).unwrap_or_default()
            }
            WebhookType::Slack => {
                serde_json::to_string(&Self::build_slack_payload(payload)).unwrap_or_default()
            }
            WebhookType::Teams => {
                serde_json::to_string(&Self::build_teams_payload(payload)).unwrap_or_default()
            }
            WebhookType::Generic | WebhookType::Custom => {
                serde_json::to_string(payload).unwrap_or_default()
            }
            _ => serde_json::to_string(payload).unwrap_or_default(),
        };

        let mut request = client
            .post(&config.url)
            .header("Content-Type", "application/json");

        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        if let Some(secret) = &config.secret {
            request = request.header("X-Webhook-Secret", secret);
        }

        match request.body(body).send().await {
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.ok();
                WebhookDeliveryResult {
                    webhook_id: config.id.clone(),
                    payload_id: payload.id.clone(),
                    success: status.is_success(),
                    status_code: Some(status.as_u16()),
                    response_body: body,
                    error_message: if status.is_success() { None } else { Some(format!("HTTP {}", status)) },
                    delivered_at: Utc::now(),
                }
            }
            Err(e) => WebhookDeliveryResult {
                webhook_id: config.id.clone(),
                payload_id: payload.id.clone(),
                success: false,
                status_code: None,
                response_body: None,
                error_message: Some(e.to_string()),
                delivered_at: Utc::now(),
            },
        }
    }

    fn build_discord_payload(payload: &WebhookPayload) -> DiscordWebhook {
        let (description, color) = if let Some(alert) = &payload.alert {
            (
                format!("{}\n**Value:** {:.2}\n**Threshold:** {:.2}", 
                    alert.message, alert.value, alert.threshold),
                match alert.level {
                    crate::monitor::types::AlertLevel::Info => 3447003,
                    crate::monitor::types::AlertLevel::Warning => 16776960,
                    crate::monitor::types::AlertLevel::Critical => 15158332,
                },
            )
        } else {
            (
                payload.message.clone().unwrap_or_default(),
                3447003,
            )
        };

        DiscordWebhook {
            content: Some(format!("**{} Alert**", payload.webhook_type)),
            embeds: vec![DiscordEmbed {
                title: Some(payload.webhook_type.to_string()),
                description: Some(description),
                color,
                fields: if let Some(alert) = &payload.alert {
                    vec![
                        DiscordField {
                            name: "Metric".to_string(),
                            value: alert.metric_type.clone(),
                            inline: true,
                        },
                        DiscordField {
                            name: "Level".to_string(),
                            value: format!("{:?}", alert.level),
                            inline: true,
                        },
                    ]
                } else {
                    Vec::new()
                },
                footer: Some(DiscordFooter {
                    text: format!("MC Server Monitor • {}", Utc::now().format("%Y-%m-%d %H:%M:%S")),
                    icon_url: None,
                }),
                timestamp: Some(Utc::now()),
            }],
            username: Some("MC Server Monitor".to_string()),
            avatar_url: None,
        }
    }

    fn build_slack_payload(payload: &WebhookPayload) -> SlackWebhook {
        let (color, level_text) = if let Some(alert) = &payload.alert {
            (
                match alert.level {
                    crate::monitor::types::AlertLevel::Info => "#3498db",
                    crate::monitor::types::AlertLevel::Warning => "#f39c12",
                    crate::monitor::types::AlertLevel::Critical => "#e74c3c",
                }.to_string(),
                format!("{:?}", alert.level),
            )
        } else {
            ("#3498db".to_string(), "Info".to_string())
        };

        let text = if let Some(alert) = &payload.alert {
            format!("*{}*\n> {}\n*Current:* {:.2} | *Threshold:* {:.2}", 
                level_text, alert.message, alert.value, alert.threshold)
        } else {
            payload.message.clone().unwrap_or_default()
        };

        SlackWebhook {
            text,
            blocks: vec![
                SlackBlock {
                    block_type: "header".to_string(),
                    text: Some(SlackText {
                        text_type: "plain_text".to_string(),
                        text: "MC Server Alert".to_string(),
                    }),
                    elements: None,
                },
                SlackBlock {
                    block_type: "section".to_string(),
                    text: Some(SlackText {
                        text_type: "mrkdwn".to_string(),
                        text,
                    }),
                    elements: None,
                },
            ],
            attachments: vec![SlackAttachment {
                color,
                blocks: Vec::new(),
            }],
        }
    }

    fn build_teams_payload(payload: &WebhookPayload) -> TeamsWebhook {
        let (color, level_text, facts) = if let Some(alert) = &payload.alert {
            (
                match alert.level {
                    crate::monitor::types::AlertLevel::Info => "3498db",
                    crate::monitor::types::AlertLevel::Warning => "f39c12",
                    crate::monitor::types::AlertLevel::Critical => "e74c3c",
                }.to_string(),
                format!("{:?}", alert.level),
                vec![
                    TeamsFact {
                        name: "Metric".to_string(),
                        value: alert.metric_type.clone(),
                    },
                    TeamsFact {
                        name: "Current Value".to_string(),
                        value: format!("{:.2}", alert.value),
                    },
                    TeamsFact {
                        name: "Threshold".to_string(),
                        value: format!("{:.2}", alert.threshold),
                    },
                ],
            )
        } else {
            (
                "3498db".to_string(),
                "Info".to_string(),
                Vec::new(),
            )
        };

        let title = if let Some(alert) = &payload.alert {
            format!("{} - {}", level_text, alert.metric_type)
        } else {
            "MC Server Monitor".to_string()
        };

        let message = if let Some(alert) = &payload.alert {
            alert.message.clone()
        } else {
            payload.message.clone().unwrap_or_default()
        };

        TeamsWebhook {
            msg_type: "Message".to_string(),
            context: "http://schema.org/extensions/message/1.0/em".to_string(),
            theme_color: color,
            summary: title.clone(),
            sections: vec![TeamsSection {
                activity_title: title,
                activity_subtitle: Some(message),
                facts,
            }],
        }
    }

    pub fn add_webhook(&self, config: WebhookConfig) {
        let mut webhooks = self.webhooks.write();
        webhooks.insert(config.id.clone(), config);
    }

    pub fn remove_webhook(&self, id: &str) -> Option<WebhookConfig> {
        let mut webhooks = self.webhooks.write();
        webhooks.remove(id)
    }

    pub fn get_webhook(&self, id: &str) -> Option<WebhookConfig> {
        let webhooks = self.webhooks.read();
        webhooks.get(id).cloned()
    }

    pub fn get_all_webhooks(&self) -> Vec<WebhookConfig> {
        let webhooks = self.webhooks.read();
        webhooks.values().cloned().collect()
    }

    pub fn update_webhook(&self, config: WebhookConfig) -> Result<(), WebhookError> {
        let mut webhooks = self.webhooks.write();
        if !webhooks.contains_key(&config.id) {
            return Err(WebhookError::NotFound(config.id.clone()));
        }
        webhooks.insert(config.id.clone(), config);
        Ok(())
    }

    pub fn enable_webhook(&self, id: &str) -> Result<(), WebhookError> {
        let mut webhooks = self.webhooks.write();
        let webhook = webhooks.get_mut(id).ok_or_else(|| {
            WebhookError::NotFound(id.to_string())
        })?;
        webhook.enabled = true;
        Ok(())
    }

    pub fn disable_webhook(&self, id: &str) -> Result<(), WebhookError> {
        let mut webhooks = self.webhooks.write();
        let webhook = webhooks.get_mut(id).ok_or_else(|| {
            WebhookError::NotFound(id.to_string())
        })?;
        webhook.enabled = false;
        Ok(())
    }

    pub fn get_delivery_history(&self, limit: usize) -> Vec<WebhookDeliveryResult> {
        let history = self.delivery_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn filter_should_notify(&self, webhook_id: &str, alert: &Alert) -> bool {
        let webhooks = self.webhooks.read();
        let config = match webhooks.get(webhook_id) {
            Some(c) => c,
            None => return false,
        };

        if let Some(ref levels) = config.filters.alert_levels {
            if !levels.iter().any(|l| l.eq_ignore_ascii_case(&format!("{:?}", alert.level))) {
                return false;
            }
        }

        if let Some(ref types) = config.filters.metric_types {
            if !types.contains(&alert.metric_type) {
                return false;
            }
        }

        if let Some(min_value) = config.filters.min_value {
            if alert.value < min_value {
                return false;
            }
        }

        true
    }

    pub async fn notify_alert(&self, webhook_id: &str, alert: Alert) -> Result<WebhookDeliveryResult, WebhookError> {
        if !self.filter_should_notify(webhook_id, &alert) {
            return Err(WebhookError::InvalidConfig("Alert does not match webhook filters".to_string()));
        }

        let payload = WebhookPayload::from_alert(WebhookType::Generic, alert);
        self.send_webhook(webhook_id, payload).await
    }

    pub async fn queue_notification(&self, webhook_id: &str, alert: Alert) -> Result<(), WebhookError> {
        if !self.filter_should_notify(webhook_id, &alert) {
            return Err(WebhookError::InvalidConfig("Alert does not match webhook filters".to_string()));
        }

        let webhooks = self.webhooks.read();
        let config = webhooks.get(webhook_id).ok_or_else(|| {
            WebhookError::NotFound(webhook_id.to_string())
        })?.clone();
        drop(webhooks);

        let payload = WebhookPayload::from_alert(config.webhook_type, alert);
        self.sender.send((config, payload)).await
            .map_err(|e| WebhookError::SendFailed(e.to_string()))?;

        Ok(())
    }

    pub fn create_discord_webhook(name: &str, url: &str) -> WebhookConfig {
        WebhookConfig {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            url: url.to_string(),
            webhook_type: WebhookType::Discord,
            enabled: true,
            secret: None,
            headers: HashMap::new(),
            retry_count: 3,
            retry_delay_ms: 1000,
            timeout_secs: 30,
            filters: WebhookFilters::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_slack_webhook(name: &str, url: &str) -> WebhookConfig {
        WebhookConfig {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            url: url.to_string(),
            webhook_type: WebhookType::Slack,
            enabled: true,
            secret: None,
            headers: HashMap::new(),
            retry_count: 3,
            retry_delay_ms: 1000,
            timeout_secs: 30,
            filters: WebhookFilters::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for WebhookNotifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_config_creation() {
        let config = WebhookNotifier::create_discord_webhook("Test", "https://discord.com/webhook");
        assert_eq!(config.name, "Test");
        assert_eq!(config.webhook_type, WebhookType::Discord);
        assert!(config.enabled);
    }

    #[test]
    fn test_add_and_get_webhook() {
        let notifier = WebhookNotifier::new();
        let config = WebhookNotifier::create_discord_webhook("Test", "https://discord.com/webhook");

        notifier.add_webhook(config.clone());
        let retrieved = notifier.get_webhook(&config.id).unwrap();
        assert_eq!(retrieved.name, "Test");
    }

    #[test]
    fn test_enable_disable_webhook() {
        let notifier = WebhookNotifier::new();
        let config = WebhookNotifier::create_discord_webhook("Test", "https://discord.com/webhook");
        notifier.add_webhook(config);

        notifier.disable_webhook("test").unwrap_err();

        let retrieved = notifier.get_all_webhooks();
        assert!(retrieved[0].enabled);
    }

    #[test]
    fn test_build_discord_payload() {
        let alert = Alert::new(
            "test".to_string(),
            crate::monitor::types::AlertLevel::Warning,
            "cpu_usage".to_string(),
            "CPU usage exceeded".to_string(),
            95.0,
            90.0,
        );

        let payload = WebhookPayload::from_alert(WebhookType::Discord, alert);
        let discord = WebhookNotifier::build_discord_payload(&payload);

        assert!(!discord.embeds.is_empty());
        assert_eq!(discord.embeds[0].color, 16776960);
    }

    #[test]
    fn test_build_slack_payload() {
        let alert = Alert::new(
            "test".to_string(),
            crate::monitor::types::AlertLevel::Critical,
            "memory_percent".to_string(),
            "Memory usage critical".to_string(),
            98.0,
            95.0,
        );

        let payload = WebhookPayload::from_alert(WebhookType::Slack, alert);
        let slack = WebhookNotifier::build_slack_payload(&payload);

        assert!(!slack.blocks.is_empty());
        assert_eq!(slack.attachments[0].color, "#e74c3c");
    }

    #[test]
    fn test_filter_should_notify() {
        let notifier = WebhookNotifier::new();
        let mut config = WebhookNotifier::create_discord_webhook("Test", "https://discord.com/webhook");
        config.filters.alert_levels = Some(vec!["Warning".to_string(), "Critical".to_string()]);
        notifier.add_webhook(config.clone());

        let warning_alert = Alert::new(
            "test".to_string(),
            crate::monitor::types::AlertLevel::Warning,
            "cpu_usage".to_string(),
            "CPU warning".to_string(),
            85.0,
            80.0,
        );

        let info_alert = Alert::new(
            "test".to_string(),
            crate::monitor::types::AlertLevel::Info,
            "cpu_usage".to_string(),
            "CPU info".to_string(),
            70.0,
            80.0,
        );

        assert!(notifier.filter_should_notify(&config.id, &warning_alert));
        assert!(!notifier.filter_should_notify(&config.id, &info_alert));
    }
}

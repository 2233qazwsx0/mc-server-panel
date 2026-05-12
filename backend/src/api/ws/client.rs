use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WsClient {
    pub id: Uuid,
    pub channels: Vec<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_ping: std::time::Instant,
}

impl WsClient {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            channels: vec!["logs".to_string(), "metrics".to_string(), "status".to_string()],
            connected_at: chrono::Utc::now(),
            last_ping: std::time::Instant::now(),
        }
    }

    pub fn update_ping(&mut self) {
        self.last_ping = std::time::Instant::now();
    }

    pub fn subscribe(&mut self, channel: &str) {
        if !self.channels.contains(&channel.to_string()) {
            self.channels.push(channel.to_string());
        }
    }

    pub fn unsubscribe(&mut self, channel: &str) {
        self.channels.retain(|c| c != channel);
    }
}

#[derive(Debug, Clone)]
pub struct ClientManager {
    clients: Arc<RwLock<HashMap<Uuid, WsClient>>>,
    max_connections: usize,
}

impl ClientManager {
    pub fn new(max_connections: usize) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
        }
    }

    pub fn add_client(&self, client: WsClient) -> Option<Uuid> {
        let mut clients = self.clients.write();

        if clients.len() >= self.max_connections {
            tracing::warn!("Max WebSocket connections reached: {}", self.max_connections);
            return None;
        }

        let id = client.id;
        clients.insert(id, client);
        tracing::info!("WebSocket client connected: {}", id);
        Some(id)
    }

    pub fn remove_client(&self, id: &Uuid) {
        let mut clients = self.clients.write();
        if clients.remove(id).is_some() {
            tracing::info!("WebSocket client disconnected: {}", id);
        }
    }

    pub fn get_client(&self, id: &Uuid) -> Option<WsClient> {
        self.clients.read().get(id).cloned()
    }

    pub fn client_count(&self) -> usize {
        self.clients.read().len()
    }

    pub fn get_clients_by_channel(&self, channel: &str) -> Vec<Uuid> {
        let clients = self.clients.read();
        clients.iter()
            .filter(|(_, c)| c.channels.contains(&channel.to_string()))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn handle_message(&self, id: &Uuid, msg: crate::api::ws::message::WsClientMessage) {
        let mut clients = self.clients.write();
        if let Some(client) = clients.get_mut(id) {
            match msg {
                crate::api::ws::message::WsClientMessage::Ping => {
                    client.update_ping();
                }
                crate::api::ws::message::WsClientMessage::Subscribe { channel } => {
                    client.subscribe(&channel);
                    tracing::debug!("Client {} subscribed to {}", id, channel);
                }
                crate::api::ws::message::WsClientMessage::Unsubscribe { channel } => {
                    client.unsubscribe(&channel);
                    tracing::debug!("Client {} unsubscribed from {}", id, channel);
                }
            }
        }
    }
}

impl Default for ClientManager {
    fn default() -> Self {
        Self::new(100)
    }
}

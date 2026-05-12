pub mod types;
pub mod proxy;
pub mod node_sync;
pub mod load_balancer;
pub mod chat_sync;
pub mod config_center;
pub mod failover;
pub mod monitor;
pub mod data_sync;
pub mod rolling_update;
pub mod topology;

pub use types::*;
pub use proxy::*;
pub use node_sync::*;
pub use load_balancer::*;
pub use chat_sync::*;
pub use config_center::*;
pub use failover::*;
pub use monitor::*;
pub use data_sync::*;
pub use rolling_update::*;
pub use topology::*;

use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct ClusterManager {
    state: Arc<ClusterState>,
    broadcast_tx: broadcast::Sender<ClusterEvent>,
}

#[derive(Default)]
pub struct ClusterState {
    pub nodes: RwLock<Vec<NodeInfo>>,
    pub proxy_config: RwLock<ProxyConfig>,
    pub load_balancer: RwLock<LoadBalancerConfig>,
    pub chat_config: RwLock<ChatSyncConfig>,
    pub failover_enabled: RwLock<bool>,
    pub rolling_update_config: RwLock<RollingUpdateConfig>,
}

impl ClusterManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        Self {
            state: Arc::new(ClusterState::default()),
            broadcast_tx,
        }
    }

    pub fn get_state(&self) -> Arc<ClusterState> {
        self.state.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ClusterEvent> {
        self.broadcast_tx.subscribe()
    }

    pub fn broadcast(&self, event: ClusterEvent) {
        let _ = self.broadcast_tx.send(event);
    }

    pub fn add_node(&self, node: NodeInfo) {
        let mut nodes = self.state.nodes.write();
        if !nodes.iter().any(|n| n.id == node.id) {
            nodes.push(node.clone());
            drop(nodes);
            let payload = serde_json::json!(node.clone());
            self.broadcast(ClusterEvent::new(ClusterEventType::NodeAdded, Some(node.id.clone()), payload));
        }
    }

    pub fn remove_node(&self, node_id: &str) {
        let mut nodes = self.state.nodes.write();
        if let Some(pos) = nodes.iter().position(|n| n.id == node_id) {
            let node = nodes.remove(pos);
            drop(nodes);
            let payload = serde_json::json!(node.clone());
            self.broadcast(ClusterEvent::new(ClusterEventType::NodeRemoved, Some(node.id.clone()), payload));
        }
    }

    pub fn update_node_status(&self, node_id: &str, status: NodeStatus) {
        let mut nodes = self.state.nodes.write();
        if let Some(node) = nodes.iter_mut().find(|n| n.id == node_id) {
            node.status = status;
            let node = node.clone();
            drop(nodes);
            let payload = serde_json::json!(node.clone());
            self.broadcast(ClusterEvent::new(ClusterEventType::NodeStatusChanged, Some(node.id.clone()), payload));
        }
    }

    pub fn get_all_nodes(&self) -> Vec<NodeInfo> {
        self.state.nodes.read().clone()
    }

    pub fn get_healthy_nodes(&self) -> Vec<NodeInfo> {
        self.state.nodes.read()
            .iter()
            .filter(|n| n.status == NodeStatus::Online)
            .cloned()
            .collect()
    }
}

impl Default for ClusterManager {
    fn default() -> Self {
        Self::new()
    }
}

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::cluster::types::*;

#[derive(Clone)]
pub struct TopologyManager {
    state: Arc<TopologyState>,
}

struct TopologyState {
    nodes: RwLock<HashMap<String, NodeInfo>>,
    connections: RwLock<Vec<NodeConnection>>,
    layouts: RwLock<HashMap<String, LayoutPosition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyView {
    pub nodes: Vec<NodeView>,
    pub connections: Vec<ConnectionView>,
    pub summary: TopologySummary,
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeView {
    pub id: String,
    pub name: String,
    pub node_type: NodeViewType,
    pub status: NodeStatus,
    pub position: Option<LayoutPosition>,
    pub metrics: NodeMetricsSummary,
    pub connections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeViewType {
    Proxy,
    Game,
    Database,
    Cache,
    Storage,
}

impl NodeViewType {
    pub fn from_node(node: &NodeInfo) -> Self {
        if node.is_proxy {
            NodeViewType::Proxy
        } else {
            NodeViewType::Game
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetricsSummary {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub player_count: u32,
    pub tps: f64,
    pub health_score: f64,
}

impl From<&NodeInfo> for NodeMetricsSummary {
    fn from(node: &NodeInfo) -> Self {
        Self {
            cpu_usage: node.cpu_usage,
            memory_usage: node.memory_usage,
            player_count: node.player_count,
            tps: node.tps,
            health_score: node.health_score(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionView {
    pub id: String,
    pub from: String,
    pub to: String,
    pub connection_type: ConnectionType,
    pub status: ConnectionStatus,
    pub latency_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Active,
    Degraded,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySummary {
    pub total_nodes: u32,
    pub online_nodes: u32,
    pub total_players: u32,
    pub avg_cpu: f64,
    pub avg_memory: f64,
    pub avg_tps: f64,
    pub total_connections: u32,
    pub active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub zone_type: ZoneType,
    pub nodes: Vec<String>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZoneType {
    Region,
    Environment,
    Custom,
}

impl TopologyManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(TopologyState {
                nodes: RwLock::new(HashMap::new()),
                connections: RwLock::new(Vec::new()),
                layouts: RwLock::new(HashMap::new()),
            }),
        }
    }

    pub fn add_node(&self, node: NodeInfo) {
        let mut nodes = self.state.nodes.write();
        nodes.insert(node.id.clone(), node);
    }

    pub fn remove_node(&self, node_id: &str) -> Option<NodeInfo> {
        let mut nodes = self.state.nodes.write();
        let removed = nodes.remove(node_id);
        if removed.is_some() {
            drop(nodes);
            self.remove_node_connections(node_id);
        }
        removed
    }

    pub fn update_node(&self, node: NodeInfo) {
        let mut nodes = self.state.nodes.write();
        nodes.insert(node.id.clone(), node);
    }

    pub fn get_node(&self, node_id: &str) -> Option<NodeInfo> {
        self.state.nodes.read().get(node_id).cloned()
    }

    pub fn get_all_nodes(&self) -> Vec<NodeInfo> {
        self.state.nodes.read().values().cloned().collect()
    }

    pub fn add_connection(&self, connection: NodeConnection) {
        let mut connections = self.state.connections.write();
        if !connections.iter().any(|c| c.from == connection.from && c.to == connection.to) {
            connections.push(connection);
        }
    }

    pub fn remove_connection(&self, from: &str, to: &str) {
        let mut connections = self.state.connections.write();
        connections.retain(|c| !(c.from == from && c.to == to));
    }

    fn remove_node_connections(&self, node_id: &str) {
        let mut connections = self.state.connections.write();
        connections.retain(|c| c.from != node_id && c.to != node_id);
    }

    pub fn get_connections(&self) -> Vec<NodeConnection> {
        self.state.connections.read().clone()
    }

    pub fn update_position(&self, node_id: &str, position: LayoutPosition) {
        self.state.layouts.write().insert(node_id.to_string(), position);
    }

    pub fn get_position(&self, node_id: &str) -> Option<LayoutPosition> {
        self.state.layouts.read().get(node_id).cloned()
    }

    pub fn generate_layout(&self, width: f64, height: f64) -> HashMap<String, LayoutPosition> {
        let nodes = self.get_all_nodes();
        let proxy_nodes: Vec<_> = nodes.iter().filter(|n| n.is_proxy).collect();
        let game_nodes: Vec<_> = nodes.iter().filter(|n| !n.is_proxy).collect();

        let mut positions = HashMap::new();
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let proxy_radius = width / 4.0;
        let game_radius = width / 3.0;

        for (i, node) in proxy_nodes.iter().enumerate() {
            if proxy_nodes.len() == 1 {
                positions.insert(node.id.clone(), LayoutPosition { x: center_x, y: 80.0 });
            } else {
                let angle = (2.0 * std::f64::consts::PI * i as f64) / proxy_nodes.len() as f64 - std::f64::consts::FRAC_PI_2;
                positions.insert(node.id.clone(), LayoutPosition {
                    x: center_x + proxy_radius * angle.cos(),
                    y: 80.0 + proxy_radius * angle.sin().abs(),
                });
            }
        }

        let game_offset_y = 200.0;
        let game_spacing = width / (game_nodes.len() as f64 + 1.0);
        for (i, node) in game_nodes.iter().enumerate() {
            positions.insert(node.id.clone(), LayoutPosition {
                x: game_spacing * (i as f64 + 1.0),
                y: game_offset_y + height / 3.0,
            });
        }

        let mut layouts = self.state.layouts.write();
        for (id, pos) in positions {
            layouts.insert(id, pos);
        }

        layouts.clone()
    }

    pub fn generate_topology_view(&self) -> TopologyView {
        let nodes = self.get_all_nodes();
        let connections = self.get_connections();

        let proxy_nodes: Vec<_> = nodes.iter().filter(|n| n.is_proxy).cloned().collect();
        let game_nodes: Vec<_> = nodes.iter().filter(|n| !n.is_proxy).cloned().collect();

        let node_views: Vec<NodeView> = nodes.iter().map(|n| {
            let node_connections: Vec<String> = connections.iter()
                .filter(|c| c.from == n.id || c.to == n.id)
                .map(|c| if c.from == n.id { c.to.clone() } else { c.from.clone() })
                .collect();

            NodeView {
                id: n.id.clone(),
                name: n.name.clone(),
                node_type: NodeViewType::from_node(n),
                status: n.status.clone(),
                position: self.get_position(&n.id),
                metrics: NodeMetricsSummary::from(n),
                connections: node_connections,
            }
        }).collect();

        let connection_views: Vec<ConnectionView> = connections.iter().map(|c| {
            let from_node = self.get_node(&c.from);
            let to_node = self.get_node(&c.to);
            let both_online = from_node.as_ref().map(|n| n.status == NodeStatus::Online).unwrap_or(false)
                && to_node.as_ref().map(|n| n.status == NodeStatus::Online).unwrap_or(false);

            ConnectionView {
                id: format!("{}-{}", c.from, c.to),
                from: c.from.clone(),
                to: c.to.clone(),
                connection_type: c.connection_type.clone(),
                status: if both_online { ConnectionStatus::Active } else { ConnectionStatus::Disconnected },
                latency_ms: c.latency_ms,
            }
        }).collect();

        let total_players: u32 = nodes.iter().map(|n| n.player_count).sum();
        let avg_cpu = if !nodes.is_empty() { nodes.iter().map(|n| n.cpu_usage).sum::<f64>() / nodes.len() as f64 } else { 0.0 };
        let avg_memory = if !nodes.is_empty() { nodes.iter().map(|n| n.memory_usage).sum::<f64>() / nodes.len() as f64 } else { 0.0 };
        let avg_tps = if !nodes.is_empty() { nodes.iter().map(|n| n.tps).sum::<f64>() / nodes.len() as f64 } else { 20.0 };
        let online_nodes = nodes.iter().filter(|n| n.status == NodeStatus::Online).count() as u32;

        let zones = vec![
            Zone {
                id: "proxy-zone".to_string(),
                name: "代理层".to_string(),
                zone_type: ZoneType::Environment,
                nodes: proxy_nodes.iter().map(|n| n.id.clone()).collect(),
                color: "#5D7C15".to_string(),
            },
            Zone {
                id: "game-zone".to_string(),
                name: "游戏节点层".to_string(),
                zone_type: ZoneType::Environment,
                nodes: game_nodes.iter().map(|n| n.id.clone()).collect(),
                color: "#DEA584".to_string(),
            },
        ];

        TopologyView {
            nodes: node_views,
            connections: connection_views,
            summary: TopologySummary {
                total_nodes: nodes.len() as u32,
                online_nodes,
                total_players,
                avg_cpu,
                avg_memory,
                avg_tps,
                total_connections: connections.len() as u32,
                active_connections: connection_views.iter().filter(|c| c.status == ConnectionStatus::Active).count() as u32,
            },
            zones,
        }
    }

    pub fn get_zone_by_region(&self) -> HashMap<String, Vec<NodeInfo>> {
        let mut zones: HashMap<String, Vec<NodeInfo>> = HashMap::new();
        for node in self.get_all_nodes() {
            let region = node.region.clone().unwrap_or_else(|| "default".to_string());
            zones.entry(region).or_default().push(node);
        }
        zones
    }

    pub fn get_shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let connections = self.get_connections();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        for conn in &connections {
            adjacency.entry(conn.from.clone()).or_default().push(conn.to.clone());
            adjacency.entry(conn.to.clone()).or_default().push(conn.from.clone());
        }

        let mut visited = HashMap::new();
        let mut queue = vec![from.to_string()];
        let mut parent: HashMap<String, String> = HashMap::new();

        visited.insert(from.to_string(), true);

        while let Some(current) = queue.pop() {
            if current == to {
                let mut path = vec![to.to_string()];
                let mut node = to.to_string();
                while let Some(p) = parent.get(&node) {
                    path.push(p.clone());
                    node = p.clone();
                }
                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains_key(neighbor) {
                        visited.insert(neighbor.clone(), true);
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        None
    }
}

impl Default for TopologyManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn calculate_health_color(score: f64) -> String {
    if score >= 80.0 {
        "#5D7C15".to_string()
    } else if score >= 50.0 {
        "#DEA584".to_string()
    } else {
        "#8B2500".to_string()
    }
}

pub fn calculate_status_color(status: &NodeStatus) -> String {
    match status {
        NodeStatus::Online => "#5D7C15".to_string(),
        NodeStatus::Offline => "#8B2500".to_string(),
        NodeStatus::Maintenance => "#DEA584".to_string(),
        NodeStatus::Draining => "#F0C040".to_string(),
    }
}

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::cluster::types::*;

#[derive(Clone)]
pub struct LoadBalancer {
    state: Arc<LoadBalancerState>,
    config: Arc<RwLock<LoadBalancerConfig>>,
}

struct LoadBalancerState {
    server_weights: RwLock<HashMap<String, u32>>,
    connection_counts: RwLock<HashMap<String, u32>>,
    session_table: RwLock<HashMap<String, String>>,
    health_status: RwLock<HashMap<String, bool>>,
}

impl LoadBalancer {
    pub fn new(config: LoadBalancerConfig) -> Self {
        Self {
            state: Arc::new(LoadBalancerState {
                server_weights: RwLock::new(HashMap::new()),
                connection_counts: RwLock::new(HashMap::new()),
                session_table: RwLock::new(HashMap::new()),
                health_status: RwLock::new(HashMap::new()),
            }),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn set_server_weight(&self, server_id: &str, weight: u32) {
        self.state.server_weights.write().insert(server_id.to_string(), weight);
    }

    pub fn get_available_servers(&self) -> Vec<String> {
        let health = self.state.health_status.read();
        health.iter()
            .filter(|(_, healthy)| **healthy)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn select_server(&self, client_id: Option<&str>) -> Option<String> {
        let config = self.config.read().clone();
        let available = self.get_available_servers();

        if available.is_empty() {
            return None;
        }

        match config.strategy {
            LoadBalanceStrategy::RoundRobin => self.round_robin_select(&available),
            LoadBalanceStrategy::LeastConnections => self.least_connections_select(&available),
            LoadBalanceStrategy::WeightedRoundRobin => self.weighted_round_robin_select(&available),
            LoadBalanceStrategy::WeightedLeastConnections => self.weighted_least_connections_select(&available),
            LoadBalanceStrategy::Hash => {
                client_id
                    .map(|id| self.hash_select(&available, id))
                    .flatten()
            }
            LoadBalanceStrategy::Random => self.random_select(&available),
        }
    }

    fn round_robin_select(&self, servers: &[String]) -> Option<String> {
        let mut counts = self.state.connection_counts.write();
        let counts = &mut *counts;

        let mut min_count = u32::MAX;
        let mut selected = None;

        for server in servers {
            let count = counts.entry(server.clone()).or_insert(0);
            if *count < min_count {
                min_count = *count;
                selected = Some(server.clone());
            }
        }

        if let Some(ref s) = selected {
            *counts.entry(s.clone()).or_insert(0) += 1;
        }

        selected
    }

    fn least_connections_select(&self, servers: &[String]) -> Option<String> {
        let counts = self.state.connection_counts.read();
        servers.iter()
            .min_by_key(|s| counts.get(*s).unwrap_or(&0))
            .cloned()
    }

    fn weighted_round_robin_select(&self, servers: &[String]) -> Option<String> {
        let weights = self.state.server_weights.read();
        let mut counts = self.state.connection_counts.write();

        let mut selected = None;
        let mut min_effective = f64::MAX;

        for server in servers {
            let weight = weights.get(server).copied().unwrap_or(1);
            let count = counts.get(server).unwrap_or(&0);
            let effective = (*count as f64) / (weight as f64);

            if effective < min_effective {
                min_effective = effective;
                selected = Some(server.clone());
            }
        }

        if let Some(ref s) = selected {
            *counts.entry(s.clone()).or_insert(0) += 1;
        }

        selected
    }

    fn weighted_least_connections_select(&self, servers: &[String]) -> Option<String> {
        let weights = self.state.server_weights.read();
        let counts = self.state.connection_counts.read();

        servers.iter()
            .map(|s| {
                let weight = weights.get(s).copied().unwrap_or(1);
                let conns = counts.get(s).unwrap_or(&0);
                let lc = (*conns as f64) / (weight as f64);
                (s.clone(), lc)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(s, _)| s)
    }

    fn hash_select(&self, servers: &[String], client_id: &str) -> Option<String> {
        if servers.is_empty() {
            return None;
        }
        let hash = self.simple_hash(client_id);
        let index = (hash % servers.len() as u32) as usize;
        servers.get(index).cloned()
    }

    fn random_select(&self, servers: &[String]) -> Option<String> {
        if servers.is_empty() {
            return None;
        }
        let index = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() % servers.len() as u128) as usize;
        servers.get(index).cloned()
    }

    fn simple_hash(&self, s: &str) -> u32 {
        s.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32).wrapping_mul(31))
    }

    pub fn record_connection(&self, server_id: &str) {
        *self.state.connection_counts.write().entry(server_id.to_string()).or_insert(0) += 1;
    }

    pub fn release_connection(&self, server_id: &str) {
        let mut counts = self.state.connection_counts.write();
        if let Some(count) = counts.get_mut(server_id) {
            *count = count.saturating_sub(1);
        }
    }

    pub fn create_sticky_session(&self, client_id: &str, server_id: &str) {
        let mut sessions = self.state.session_table.write();
        sessions.insert(client_id.to_string(), server_id.to_string());
    }

    pub fn get_sticky_session(&self, client_id: &str) -> Option<String> {
        self.state.session_table.read().get(client_id).cloned()
    }

    pub fn invalidate_session(&self, client_id: &str) {
        self.state.session_table.write().remove(client_id);
    }

    pub fn update_health_status(&self, server_id: &str, healthy: bool) {
        self.state.health_status.write().insert(server_id.to_string(), healthy);
        if !healthy {
            let mut counts = self.state.connection_counts.write();
            counts.insert(server_id.to_string(), 0);
        }
    }

    pub fn get_connection_count(&self, server_id: &str) -> u32 {
        *self.state.connection_counts.read().get(server_id).unwrap_or(&0)
    }

    pub fn get_all_connection_counts(&self) -> HashMap<String, u32> {
        self.state.connection_counts.read().clone()
    }

    pub fn update_config(&self, config: LoadBalancerConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> LoadBalancerConfig {
        self.config.read().clone()
    }

    pub fn get_stats(&self) -> LoadBalancerStats {
        let counts = self.state.connection_counts.read();
        let health = self.state.health_status.read();
        let total_connections: u32 = counts.values().sum();
        let healthy_servers = health.values().filter(|&&h| h).count() as u32;
        let total_servers = health.len() as u32;

        LoadBalancerStats {
            strategy: self.config.read().strategy.clone(),
            total_connections,
            healthy_servers,
            total_servers,
            connection_distribution: counts.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub strategy: LoadBalanceStrategy,
    pub total_connections: u32,
    pub healthy_servers: u32,
    pub total_servers: u32,
    pub connection_distribution: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
    pub check_type: HealthCheckType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthCheckType {
    Tcp,
    Http,
    MinecraftPing,
}

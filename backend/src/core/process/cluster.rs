use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::error::ProcessError;
use crate::config::ServerConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetadata {
    pub id: String,
    pub name: String,
    pub instance_type: InstanceType,
    pub created_at: DateTime<Utc>,
    pub status: InstanceStatus,
    pub config: ServerConfig,
    pub resource_usage: Option<ResourceUsage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstanceType {
    Vanilla,
    Spigot,
    Paper,
    Forge,
    Fabric,
    BungeeCord,
    Velocity,
    Custom,
}

impl Default for InstanceType {
    fn default() -> Self {
        Self::Vanilla
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstanceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
    Unknown,
}

impl Default for InstanceStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub thread_count: u32,
    pub open_files: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStats {
    pub total_instances: usize,
    pub running_instances: usize,
    pub stopped_instances: usize,
    pub failed_instances: usize,
    pub total_cpu_percent: f32,
    pub total_memory_mb: u64,
}

#[derive(Clone)]
pub struct Instance {
    metadata: Arc<RwLock<InstanceMetadata>>,
    health_check_tx: tokio::sync::watch::Sender<bool>,
}

impl Instance {
    pub async fn new(name: String, instance_type: InstanceType, config: ServerConfig) -> Result<Self, ProcessError> {
        let id = Uuid::new_v4().to_string();
        let metadata = InstanceMetadata {
            id: id.clone(),
            name,
            instance_type,
            created_at: Utc::now(),
            status: InstanceStatus::Stopped,
            config,
            resource_usage: None,
        };

        let (health_check_tx, _) = tokio::sync::watch::channel(true);

        Ok(Self {
            metadata: Arc::new(RwLock::new(metadata)),
            health_check_tx,
        })
    }

    pub async fn get_metadata(&self) -> InstanceMetadata {
        self.metadata.read().await.clone()
    }

}

#[derive(Clone)]
pub struct InstanceCluster {
    instances: Arc<RwLock<HashMap<String, Arc<Instance>>>>,
    config: Arc<tokio::sync::RwLock<ClusterConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub max_instances: usize,
    pub default_allocation: ResourceAllocation,
    pub health_check_interval_secs: u64,
    pub auto_restart_enabled: bool,
    pub restart_delay_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub max_memory_mb: u64,
    pub max_cpu_cores: f32,
    pub max_disk_mb: u64,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            max_instances: 10,
            default_allocation: ResourceAllocation {
                max_memory_mb: 4096,
                max_cpu_cores: 2.0,
                max_disk_mb: 10240,
            },
            health_check_interval_secs: 30,
            auto_restart_enabled: true,
            restart_delay_secs: 5,
        }
    }
}

impl InstanceCluster {
    pub fn new(config: ClusterConfig) -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(tokio::sync::RwLock::new(config)),
        }
    }

    pub async fn create_instance(
        &self,
        name: String,
        instance_type: InstanceType,
        config: ServerConfig,
    ) -> Result<Arc<Instance>, ProcessError> {
        let instances = self.instances.read().await;
        if instances.values().any(|i| {
            let metadata = i.metadata.try_read();
            if let Ok(m) = metadata {
                m.name == name
            } else {
                false
            }
        }) {
            return Err(ProcessError::InstanceAlreadyExists(name));
        }
        drop(instances);

        let instance = Arc::new(Instance::new(name, instance_type, config).await?);
        let mut instances = self.instances.write().await;
        instances.insert(instance.metadata.try_read().unwrap().id.clone(), instance.clone());

        Ok(instance)
    }

    pub async fn get_instance(&self, id: &str) -> Option<Arc<Instance>> {
        let instances = self.instances.read().await;
        instances.get(id).cloned()
    }

    pub async fn list_instances(&self) -> Vec<InstanceMetadata> {
        let instances = self.instances.read().await;
        futures::future::join_all(
            instances.values().map(|i| async move { i.get_metadata().await })
        ).await
    }

    pub async fn remove_instance(&self, id: &str) -> Result<(), ProcessError> {
        let mut instances = self.instances.write().await;
        if instances.remove(id).is_some() {
            Ok(())
        } else {
            Err(ProcessError::InstanceNotFound(id.to_string()))
        }
    }

    pub async fn update_instance_status(&self, id: &str, status: InstanceStatus) -> Result<(), ProcessError> {
        let instance = self.get_instance(id).await
            .ok_or_else(|| ProcessError::InstanceNotFound(id.to_string()))?;

        let mut metadata = instance.metadata.write().await;
        metadata.status = status;

        Ok(())
    }

    pub async fn get_cluster_stats(&self) -> ClusterStats {
        let instances = self.instances.read().await;
        let metas: Vec<InstanceMetadata> = futures::future::join_all(
            instances.values().map(|i| async move { i.get_metadata().await })
        ).await;

        let mut stats = ClusterStats {
            total_instances: metas.len(),
            running_instances: 0,
            stopped_instances: 0,
            failed_instances: 0,
            total_cpu_percent: 0.0,
            total_memory_mb: 0,
        };

        for meta in &metas {
            match meta.status {
                InstanceStatus::Running => stats.running_instances += 1,
                InstanceStatus::Stopped => stats.stopped_instances += 1,
                InstanceStatus::Failed => stats.failed_instances += 1,
                _ => {}
            }

            if let Some(usage) = &meta.resource_usage {
                stats.total_cpu_percent += usage.cpu_percent;
                stats.total_memory_mb += usage.memory_mb;
            }
        }

        stats
    }

    pub async fn update_config(&self, config: ClusterConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    pub async fn get_config(&self) -> ClusterConfig {
        self.config.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cluster_creation() {
        let cluster = InstanceCluster::new(ClusterConfig::default());
        assert_eq!(cluster.get_cluster_stats().await.total_instances, 0);
    }

    #[tokio::test]
    async fn test_instance_creation() {
        let cluster = InstanceCluster::new(ClusterConfig::default());
        let config = ServerConfig::default();

        let instance = cluster.create_instance(
            "test-server".to_string(),
            InstanceType::Vanilla,
            config,
        ).await.unwrap();

        let metadata = instance.get_metadata().await;
        assert_eq!(metadata.name, "test-server");
        assert_eq!(metadata.instance_type, InstanceType::Vanilla);
    }

    #[tokio::test]
    async fn test_duplicate_instance_name() {
        let cluster = InstanceCluster::new(ClusterConfig::default());
        let config = ServerConfig::default();

        cluster.create_instance(
            "test-server".to_string(),
            InstanceType::Vanilla,
            config.clone(),
        ).await.unwrap();

        let result = cluster.create_instance(
            "test-server".to_string(),
            InstanceType::Paper,
            config,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_instance_removal() {
        let cluster = InstanceCluster::new(ClusterConfig::default());
        let config = ServerConfig::default();

        let instance = cluster.create_instance(
            "test-server".to_string(),
            InstanceType::Vanilla,
            config,
        ).await.unwrap();

        let id = instance.get_metadata().await.id;
        assert!(cluster.remove_instance(&id).await.is_ok());
        assert!(cluster.get_instance(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_cluster_stats() {
        let cluster = InstanceCluster::new(ClusterConfig::default());
        let config = ServerConfig::default();

        cluster.create_instance(
            "server1".to_string(),
            InstanceType::Vanilla,
            config.clone(),
        ).await.unwrap();

        cluster.create_instance(
            "server2".to_string(),
            InstanceType::Paper,
            config,
        ).await.unwrap();

        let stats = cluster.get_cluster_stats().await;
        assert_eq!(stats.total_instances, 2);
        assert_eq!(stats.stopped_instances, 2);
    }
}

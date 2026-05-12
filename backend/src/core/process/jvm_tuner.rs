use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::error::ProcessError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmProfile {
    pub name: String,
    pub description: String,
    pub args: Vec<String>,
    pub recommended_memory_mb: u64,
    pub recommended_cpu_cores: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmMetrics {
    pub heap_used_mb: u64,
    pub heap_max_mb: u64,
    pub non_heap_used_mb: u64,
    pub thread_count: u32,
    pub gc_count: u64,
    pub gc_time_ms: u64,
    pub cpu_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmConfig {
    pub min_memory_mb: u64,
    pub max_memory_mb: u64,
    pub gc_type: GcType,
    pub extra_args: Vec<String>,
    pub enable_metrics: bool,
    pub metrics_interval_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GcType {
    Serial,
    Parallel,
    Cms,
    G1,
    Zgc,
    Shenandoah,
}

impl Default for GcType {
    fn default() -> Self {
        Self::G1
    }
}

impl JvmConfig {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        args.push(format!("-Xms{}M", self.min_memory_mb));
        args.push(format!("-Xmx{}M", self.max_memory_mb));

        let gc_arg = match self.gc_type {
            GcType::Serial => "-XX:+UseSerialGC",
            GcType::Parallel => "-XX:+UseParallelGC",
            GcType::Cms => "-XX:+UseConcMarkSweepGC",
            GcType::G1 => "-XX:+UseG1GC",
            GcType::Zgc => "-XX:+UseZGC",
            GcType::Shenandoah => "-XX:+UseShenandoahGC",
        };
        args.push(gc_arg.to_string());

        args.extend(self.extra_args.clone());

        args
    }
}

impl Default for JvmConfig {
    fn default() -> Self {
        Self {
            min_memory_mb: 2048,
            max_memory_mb: 4096,
            gc_type: GcType::default(),
            extra_args: Vec::new(),
            enable_metrics: true,
            metrics_interval_secs: 10,
        }
    }
}

pub struct JvmTuner {
    config: Arc<RwLock<JvmConfig>>,
    profiles: Arc<RwLock<HashMap<String, JvmProfile>>>,
    current_profile: Arc<RwLock<Option<String>>>,
}

impl JvmTuner {
    pub fn new(config: JvmConfig) -> Self {
        let profiles = Self::create_default_profiles();

        Self {
            config: Arc::new(RwLock::new(config)),
            profiles: Arc::new(RwLock::new(profiles)),
            current_profile: Arc::new(RwLock::new(None)),
        }
    }

    fn create_default_profiles() -> HashMap<String, JvmProfile> {
        let mut profiles = HashMap::new();

        profiles.insert(
            "low".to_string(),
            JvmProfile {
                name: "low".to_string(),
                description: "Low memory profile for testing".to_string(),
                args: vec![
                    "-Xms512M".to_string(),
                    "-Xmx1024M".to_string(),
                    "-XX:+UseG1GC".to_string(),
                    "-XX:MaxGCPauseMillis=200".to_string(),
                ],
                recommended_memory_mb: 1024,
                recommended_cpu_cores: 1.0,
            },
        );

        profiles.insert(
            "medium".to_string(),
            JvmProfile {
                name: "medium".to_string(),
                description: "Medium memory profile for small servers".to_string(),
                args: vec![
                    "-Xms2G".to_string(),
                    "-Xmx4G".to_string(),
                    "-XX:+UseG1GC".to_string(),
                    "-XX:MaxGCPauseMillis=100".to_string(),
                    "-XX:+ParallelRefProcEnabled".to_string(),
                ],
                recommended_memory_mb: 4096,
                recommended_cpu_cores: 2.0,
            },
        );

        profiles.insert(
            "high".to_string(),
            JvmProfile {
                name: "high".to_string(),
                description: "High performance profile for large servers".to_string(),
                args: vec![
                    "-Xms4G".to_string(),
                    "-Xmx8G".to_string(),
                    "-XX:+UseG1GC".to_string(),
                    "-XX:MaxGCPauseMillis=50".to_string(),
                    "-XX:+ParallelRefProcEnabled".to_string(),
                    "-XX:-UseG1GlobalConcurrentGC".to_string(),
                ],
                recommended_memory_mb: 8192,
                recommended_cpu_cores: 4.0,
            },
        );

        profiles.insert(
            "zgc".to_string(),
            JvmProfile {
                name: "zgc".to_string(),
                description: "ZGC profile for minimal pause times".to_string(),
                args: vec![
                    "-Xms4G".to_string(),
                    "-Xmx8G".to_string(),
                    "-XX:+UseZGC".to_string(),
                    "-XX:+ZGenerations".to_string(),
                ],
                recommended_memory_mb: 8192,
                recommended_cpu_cores: 4.0,
            },
        );

        profiles
    }

    pub async fn get_config(&self) -> JvmConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, config: JvmConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
        info!("JVM configuration updated");
    }

    pub async fn apply_profile(&self, profile_name: &str) -> Result<Vec<String>, ProcessError> {
        let profiles = self.profiles.read().await;
        let profile = profiles
            .get(profile_name)
            .ok_or_else(|| ProcessError::JvmTuningError(format!("Profile not found: {}", profile_name)))?
            .clone();
        drop(profiles);

        let mut config = self.config.write().await;
        config.extra_args = profile.args.clone();
        config.recommended_memory_mb = profile.recommended_memory_mb;

        let mut current = self.current_profile.write().await;
        *current = Some(profile_name.to_string());

        info!("Applied JVM profile: {}", profile_name);
        Ok(config.to_args())
    }

    pub async fn get_current_profile(&self) -> Option<String> {
        self.current_profile.read().await.clone()
    }

    pub async fn list_profiles(&self) -> Vec<JvmProfile> {
        let profiles = self.profiles.read().await;
        profiles.values().cloned().collect()
    }

    pub async fn add_profile(&self, profile: JvmProfile) -> Result<(), ProcessError> {
        let mut profiles = self.profiles.write().await;
        if profiles.contains_key(&profile.name) {
            return Err(ProcessError::JvmTuningError(format!(
                "Profile already exists: {}",
                profile.name
            )));
        }
        profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    pub async fn remove_profile(&self, name: &str) -> Result<(), ProcessError> {
        let mut profiles = self.profiles.write().await;
        if profiles.remove(name).is_none() {
            return Err(ProcessError::JvmTuningError(format!(
                "Profile not found: {}",
                name
            )));
        }
        Ok(())
    }

    pub async fn optimize_for_throughput(&self) -> Result<Vec<String>, ProcessError> {
        let mut config = self.config.write().await;
        config.extra_args = vec![
            "-XX:+UseParallelGC".to_string(),
            "-XX:+UseParallelOldGC".to_string(),
            "-XX:ParallelGCThreads=4".to_string(),
            "-XX:+UseAdaptiveSizePolicy".to_string(),
        ];
        info!("Optimized JVM for throughput");
        Ok(config.to_args())
    }

    pub async fn optimize_for_latency(&self) -> Result<Vec<String>, ProcessError> {
        let mut config = self.config.write().await;
        config.gc_type = GcType::Zgc;
        config.extra_args = vec![
            "-XX:+UseZGC".to_string(),
            "-XX:+ZGenerations".to_string(),
        ];
        info!("Optimized JVM for latency");
        Ok(config.to_args())
    }

    pub async fn adjust_memory(&self, min_mb: u64, max_mb: u64) -> Result<Vec<String>, ProcessError> {
        if min_mb > max_mb {
            return Err(ProcessError::JvmTuningError(
                "Minimum memory cannot exceed maximum memory".to_string(),
            ));
        }

        let mut config = self.config.write().await;
        config.min_memory_mb = min_mb;
        config.max_memory_mb = max_mb;
        info!("Adjusted memory: {}M - {}M", min_mb, max_mb);
        Ok(config.to_args())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jvm_config_default() {
        let config = JvmConfig::default();
        assert_eq!(config.min_memory_mb, 2048);
        assert_eq!(config.max_memory_mb, 4096);
        assert_eq!(config.gc_type, GcType::G1);
    }

    #[test]
    fn test_jvm_config_to_args() {
        let config = JvmConfig {
            min_memory_mb: 1024,
            max_memory_mb: 2048,
            gc_type: GcType::G1,
            extra_args: vec!["-XX:+AggressiveOpts".to_string()],
            enable_metrics: true,
            metrics_interval_secs: 10,
        };

        let args = config.to_args();
        assert!(args.contains(&"-Xms1024M".to_string()));
        assert!(args.contains(&"-Xmx2048M".to_string()));
        assert!(args.contains(&"-XX:+UseG1GC".to_string()));
        assert!(args.contains(&"-XX:+AggressiveOpts".to_string()));
    }

    #[test]
    fn test_gc_type_serialization() {
        let gc = GcType::Zgc;
        let json = serde_json::to_string(&gc).unwrap();
        assert_eq!(json, "\"zgc\"");

        let deserialized: GcType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, GcType::Zgc);
    }

    #[tokio::test]
    async fn test_jvm_tuner_creation() {
        let tuner = JvmTuner::new(JvmConfig::default());
        let profiles = tuner.list_profiles().await;
        assert!(!profiles.is_empty());
    }

    #[tokio::test]
    async fn test_apply_profile() {
        let tuner = JvmTuner::new(JvmConfig::default());
        let result = tuner.apply_profile("medium").await;
        assert!(result.is_ok());

        let profile = tuner.get_current_profile().await;
        assert_eq!(profile, Some("medium".to_string()));
    }

    #[tokio::test]
    async fn test_apply_invalid_profile() {
        let tuner = JvmTuner::new(JvmConfig::default());
        let result = tuner.apply_profile("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_profile() {
        let tuner = JvmTuner::new(JvmConfig::default());
        let profile = JvmProfile {
            name: "custom".to_string(),
            description: "Custom profile".to_string(),
            args: vec!["-Xms1G".to_string(), "-Xmx2G".to_string()],
            recommended_memory_mb: 2048,
            recommended_cpu_cores: 2.0,
        };

        assert!(tuner.add_profile(profile).await.is_ok());
        assert!(tuner.apply_profile("custom").await.is_ok());
    }

    #[tokio::test]
    async fn test_adjust_memory() {
        let tuner = JvmTuner::new(JvmConfig::default());
        let result = tuner.adjust_memory(1024, 2048).await;
        assert!(result.is_ok());

        let config = tuner.get_config().await;
        assert_eq!(config.min_memory_mb, 1024);
        assert_eq!(config.max_memory_mb, 2048);
    }

    #[tokio::test]
    async fn test_adjust_memory_invalid() {
        let tuner = JvmTuner::new(JvmConfig::default());
        let result = tuner.adjust_memory(4096, 2048).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_optimize_throughput() {
        let tuner = JvmTuner::new(JvmConfig::default());
        let result = tuner.optimize_for_throughput().await;
        assert!(result.is_ok());

        let args = result.unwrap();
        assert!(args.contains(&"-XX:+UseParallelGC".to_string()));
    }

    #[tokio::test]
    async fn test_optimize_latency() {
        let tuner = JvmTuner::new(JvmConfig::default());
        let result = tuner.optimize_for_latency().await;
        assert!(result.is_ok());

        let config = tuner.get_config().await;
        assert_eq!(config.gc_type, GcType::Zgc);
    }
}

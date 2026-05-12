use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::error::ProcessError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarmupStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

impl Default for WarmupStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(Debug, Clone)]
pub struct WarmupStage {
    pub name: String,
    pub description: String,
    pub target_duration: Duration,
    pub actual_duration: Option<Duration>,
    pub status: WarmupStatus,
    pub commands: Vec<String>,
}

impl WarmupStage {
    pub fn new(name: String, description: String, target_duration: Duration) -> Self {
        Self {
            name,
            description,
            target_duration,
            actual_duration: None,
            status: WarmupStatus::NotStarted,
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: String) {
        self.commands.push(command);
    }
}

#[derive(Debug, Clone)]
pub struct WarmupProgress {
    pub stage_index: usize,
    pub stage_name: String,
    pub elapsed: Duration,
    pub remaining: Duration,
    pub percent_complete: f32,
    pub overall_percent: f32,
}

#[derive(Debug, Clone)]
pub struct WarmupConfig {
    pub enabled: bool,
    pub stages: Vec<WarmupStageConfig>,
    pub total_timeout: Duration,
    pub retry_on_failure: bool,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupStageConfig {
    pub name: String,
    pub description: String,
    pub target_duration_secs: u64,
    pub commands: Vec<String>,
    pub success_indicators: Vec<String>,
}

use serde::{Deserialize, Serialize};

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stages: vec![
                WarmupStageConfig {
                    name: "server_initialization".to_string(),
                    description: "Server initialization and world loading".to_string(),
                    target_duration_secs: 30,
                    commands: vec![],
                    success_indicators: vec![
                        "Done".to_string(),
                        "Server started".to_string(),
                    ],
                },
                WarmupStageConfig {
                    name: "plugin_loading".to_string(),
                    description: "Plugin initialization".to_string(),
                    target_duration_secs: 15,
                    commands: vec![],
                    success_indicators: vec![
                        "Loaded".to_string(),
                        "enabled".to_string(),
                    ],
                },
                WarmupStageConfig {
                    name: "world_pregeneration".to_string(),
                    description: "World chunk pre-generation".to_string(),
                    target_duration_secs: 60,
                    commands: vec!["forceload add 0 0".to_string()],
                    success_indicators: vec![],
                },
            ],
            total_timeout: Duration::from_secs(120),
            retry_on_failure: true,
            max_retries: 3,
        }
    }
}

struct WarmupState {
    status: WarmupStatus,
    current_stage: usize,
    start_time: Option<Instant>,
    stage_start_time: Option<Instant>,
    retry_count: u32,
}

impl Default for WarmupState {
    fn default() -> Self {
        Self {
            status: WarmupStatus::NotStarted,
            current_stage: 0,
            start_time: None,
            stage_start_time: None,
            retry_count: 0,
        }
    }
}

pub struct WarmupManager {
    config: WarmupConfig,
    stages: Arc<RwLock<Vec<WarmupStage>>>,
    state: Arc<RwLock<WarmupState>>,
}

impl WarmupManager {
    pub fn new(config: WarmupConfig) -> Self {
        let stages = config
            .stages
            .iter()
            .map(|stage_config| {
                let mut stage = WarmupStage::new(
                    stage_config.name.clone(),
                    stage_config.description.clone(),
                    Duration::from_secs(stage_config.target_duration_secs),
                );
                for cmd in &stage_config.commands {
                    stage.add_command(cmd.clone());
                }
                stage
            })
            .collect();

        Self {
            config,
            stages: Arc::new(RwLock::new(stages)),
            state: Arc::new(RwLock::new(WarmupState::default())),
        }
    }

    pub async fn start(&self) -> Result<(), ProcessError> {
        let mut state = self.state.write().await;
        if state.status == WarmupStatus::InProgress {
            return Err(ProcessError::InvalidConfiguration(
                "Warmup already in progress".to_string(),
            ));
        }

        state.status = WarmupStatus::InProgress;
        state.current_stage = 0;
        state.start_time = Some(Instant::now());
        state.stage_start_time = Some(Instant::now());
        state.retry_count = 0;

        info!("Warmup started");
        Ok(())
    }

    pub async fn complete_stage(&self) -> Result<(), ProcessError> {
        let mut state = self.state.write().await;
        let mut stages = self.stages.write().await;

        if state.current_stage >= stages.len() {
            state.status = WarmupStatus::Completed;
            info!("Warmup completed successfully");
            return Ok(());
        }

        let stage = &mut stages[state.current_stage];
        if let Some(start) = state.stage_start_time {
            stage.actual_duration = Some(Instant::now().duration_since(start));
        }
        stage.status = WarmupStatus::Completed;

        state.current_stage += 1;
        state.stage_start_time = Some(Instant::now());

        if state.current_stage >= stages.len() {
            state.status = WarmupStatus::Completed;
            info!("Warmup completed successfully");
        }

        Ok(())
    }

    pub async fn fail(&self) -> Result<(), ProcessError> {
        let mut state = self.state.write().await;

        if state.retry_count < self.config.max_retries {
            state.retry_count += 1;
            state.stage_start_time = Some(Instant::now());
            warn!(
                "Warmup stage failed, retrying ({}/{})",
                state.retry_count, self.config.max_retries
            );
            return Ok(());
        }

        state.status = WarmupStatus::Failed;
        warn!("Warmup failed after {} retries", self.config.max_retries);
        Err(ProcessError::WarmupTimeout)
    }

    pub async fn skip(&self) {
        let mut state = self.state.write().await;
        state.status = WarmupStatus::Skipped;
        info!("Warmup skipped");
    }

    pub async fn get_status(&self) -> WarmupStatus {
        self.state.read().await.status
    }

    pub async fn get_progress(&self) -> Option<WarmupProgress> {
        let state = self.state.read().await;

        if state.status == WarmupStatus::NotStarted {
            return None;
        }

        let stages = self.stages.read().await;
        let current_stage_name = stages
            .get(state.current_stage)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "completed".to_string());

        let (elapsed, remaining, percent_complete) = if let Some(start) = state.stage_start_time {
            let elapsed = Instant::now().duration_since(start);
            let target = stages
                .get(state.current_stage)
                .map(|s| s.target_duration)
                .unwrap_or(Duration::from_secs(0));
            let remaining = if elapsed < target {
                target - elapsed
            } else {
                Duration::from_secs(0)
            };
            let percent_complete = if target.as_secs() > 0 {
                (elapsed.as_secs_f32() / target.as_secs_f32() * 100.0).min(100.0)
            } else {
                100.0
            };
            (elapsed, remaining, percent_complete)
        } else {
            (Duration::from_secs(0), Duration::from_secs(0), 0.0)
        };

        let total_stages = stages.len();
        let overall_percent = if total_stages > 0 {
            (state.current_stage as f32 / total_stages as f32) * 100.0 + percent_complete / total_stages as f32
        } else {
            100.0
        };

        Some(WarmupProgress {
            stage_index: state.current_stage,
            stage_name: current_stage_name,
            elapsed,
            remaining,
            percent_complete,
            overall_percent,
        })
    }

    pub async fn get_current_stage(&self) -> Option<WarmupStage> {
        let state = self.state.read().await;
        let stages = self.stages.read().await;
        stages.get(state.current_stage).cloned()
    }

    pub async fn get_stages(&self) -> Vec<WarmupStage> {
        self.stages.read().await.clone()
    }

    pub async fn is_complete(&self) -> bool {
        self.state.read().await.status == WarmupStatus::Completed
    }

    pub async fn check_timeout(&self) -> Result<(), ProcessError> {
        let state = self.state.read().await;

        if let Some(start) = state.start_time {
            let elapsed = Instant::now().duration_since(start);
            if elapsed > self.config.total_timeout {
                warn!("Warmup timeout after {:?}", elapsed);
                drop(state);
                self.fail().await?;
                return Err(ProcessError::WarmupTimeout);
            }
        }

        Ok(())
    }

    pub fn get_config(&self) -> WarmupConfig {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_warmup_creation() {
        let config = WarmupConfig::default();
        let manager = WarmupManager::new(config);

        assert_eq!(manager.get_status().await, WarmupStatus::NotStarted);
    }

    #[tokio::test]
    async fn test_warmup_start() {
        let config = WarmupConfig::default();
        let manager = WarmupManager::new(config);

        manager.start().await.unwrap();
        assert_eq!(manager.get_status().await, WarmupStatus::InProgress);
    }

    #[tokio::test]
    async fn test_warmup_progress() {
        let config = WarmupConfig::default();
        let manager = WarmupManager::new(config);

        manager.start().await.unwrap();

        let progress = manager.get_progress().await;
        assert!(progress.is_some());

        let progress = progress.unwrap();
        assert_eq!(progress.stage_index, 0);
        assert_eq!(progress.stage_name, "server_initialization");
    }

    #[tokio::test]
    async fn test_warmup_complete_stages() {
        let config = WarmupConfig::default();
        let manager = WarmupManager::new(config);

        manager.start().await.unwrap();

        while !manager.is_complete().await {
            manager.complete_stage().await.unwrap();
        }

        assert!(manager.is_complete().await);
    }

    #[tokio::test]
    async fn test_warmup_skip() {
        let config = WarmupConfig::default();
        let manager = WarmupManager::new(config);

        manager.start().await.unwrap();
        manager.skip().await;

        assert_eq!(manager.get_status().await, WarmupStatus::Skipped);
    }

    #[tokio::test]
    async fn test_warmup_current_stage() {
        let config = WarmupConfig::default();
        let manager = WarmupManager::new(config);

        manager.start().await.unwrap();

        let stage = manager.get_current_stage().await;
        assert!(stage.is_some());
        assert_eq!(stage.unwrap().name, "server_initialization");
    }

    #[tokio::test]
    async fn test_warmup_timeout() {
        let config = WarmupConfig {
            total_timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let manager = WarmupManager::new(config);

        manager.start().await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = manager.check_timeout().await;
        assert!(result.is_err());
    }
}

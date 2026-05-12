use crate::automation::{TaskResult, TaskStatus};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct WarmupScript {
    config: WarmupConfig,
    last_warmup: Option<DateTime<Utc>>,
    warmup_history: RwLock<Vec<WarmupStep>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WarmupConfig {
    pub enabled: bool,
    pub auto_warmup_on_start: bool,
    pub warmup_commands: Vec<WarmupCommand>,
    pub delay_between_commands_ms: u64,
    pub health_check_interval_secs: u64,
    pub max_warmup_time_secs: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WarmupCommand {
    pub command: String,
    pub description: String,
    pub delay_after_ms: Option<u64>,
    pub required: bool,
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_warmup_on_start: true,
            warmup_commands: vec![
                WarmupCommand {
                    command: "list".to_string(),
                    description: "获取玩家列表".to_string(),
                    delay_after_ms: Some(1000),
                    required: false,
                },
                WarmupCommand {
                    command: "timings on".to_string(),
                    description: "启用性能监控".to_string(),
                    delay_after_ms: Some(500),
                    required: false,
                },
                WarmupCommand {
                    command: "reload".to_string(),
                    description: "重载配置".to_string(),
                    delay_after_ms: Some(3000),
                    required: true,
                },
            ],
            delay_between_commands_ms: 500,
            health_check_interval_secs: 5,
            max_warmup_time_secs: 120,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WarmupStep {
    pub command: String,
    pub description: String,
    pub status: WarmupStepStatus,
    pub duration_ms: u64,
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum WarmupStepStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

impl WarmupScript {
    pub fn new(config: WarmupConfig) -> Self {
        Self {
            config,
            last_warmup: None,
            warmup_history: RwLock::new(Vec::new()),
        }
    }

    pub fn update_config(&mut self, config: WarmupConfig) {
        self.config = config;
    }

    pub fn add_command(&mut self, command: WarmupCommand) {
        self.config.warmup_commands.push(command);
    }

    pub fn remove_command(&mut self, index: usize) -> bool {
        if index < self.config.warmup_commands.len() {
            self.config.warmup_commands.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear_commands(&mut self) {
        self.config.warmup_commands.clear();
    }

    pub async fn run_warmup<F, Fut>(&self, executor: F) -> WarmupResult
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<String, String>>,
    {
        let start = std::time::Instant::now();

        let mut steps = Vec::new();
        let mut success_count = 0;
        let mut failed_count = 0;

        for cmd in &self.config.warmup_commands {
            let step_start = std::time::Instant::now();

            steps.push(WarmupStep {
                command: cmd.command.clone(),
                description: cmd.description.clone(),
                status: WarmupStepStatus::Running,
                duration_ms: 0,
                output: None,
            });

            let step_idx = steps.len() - 1;

            info!("Executing warmup command: {}", cmd.command);

            match executor(cmd.command.clone()).await {
                Ok(output) => {
                    let duration = step_start.elapsed().as_millis() as u64;
                    steps[step_idx].status = WarmupStepStatus::Success;
                    steps[step_idx].duration_ms = duration;
                    steps[step_idx].output = Some(output);
                    success_count += 1;
                    info!("Warmup command succeeded: {} ({}ms)", cmd.command, duration);
                }
                Err(e) => {
                    let duration = step_start.elapsed().as_millis() as u64;
                    steps[step_idx].status = if cmd.required {
                        WarmupStepStatus::Failed
                    } else {
                        WarmupStepStatus::Skipped
                    };
                    steps[step_idx].duration_ms = duration;
                    steps[step_idx].output = Some(e.clone());
                    failed_count += 1;

                    warn!("Warmup command failed: {} - {}", cmd.command, e);

                    if cmd.required {
                        break;
                    }
                }
            }

            if let Some(delay) = cmd.delay_after_ms {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(self.config.delay_between_commands_ms)).await;
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        self.last_warmup = Some(Utc::now());

        {
            let mut history = self.warmup_history.write();
            history.push(steps.clone());
            if history.len() > 100 {
                history.remove(0);
            }
        }

        WarmupResult {
            success: failed_count == 0 || !self.config.warmup_commands.iter().any(|c| c.required),
            total_duration_ms: duration_ms,
            steps,
            success_count,
            failed_count,
            timestamp: Utc::now(),
        }
    }

    pub fn get_status(&self) -> TaskStatus {
        TaskStatus {
            id: "warmup".to_string(),
            name: "自动预热脚本".to_string(),
            task_type: "warmup".to_string(),
            enabled: self.config.enabled,
            last_run: self.last_warmup,
            next_run: None,
            last_result: None,
            schedule: "on_start".to_string(),
        }
    }

    pub fn get_history(&self) -> Vec<Vec<WarmupStep>> {
        self.warmup_history.read().clone()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WarmupResult {
    pub success: bool,
    pub total_duration_ms: u64,
    pub steps: Vec<WarmupStep>,
    pub success_count: usize,
    pub failed_count: usize,
    pub timestamp: DateTime<Utc>,
}

pub struct WarmupRunner {
    script: WarmupScript,
    event_tx: mpsc::Sender<WarmupEvent>,
}

#[derive(Debug, Clone)]
pub enum WarmupEvent {
    Started,
    StepCompleted(usize, bool),
    Completed(WarmupResult),
    Failed(String),
}

impl WarmupRunner {
    pub fn new(script: WarmupScript) -> Self {
        let (event_tx, _) = mpsc::channel(100);
        Self { script, event_tx }
    }

    pub fn subscribe(&self) -> mpsc::Receiver<WarmupEvent> {
        self.event_tx.subscribe()
    }

    pub async fn run<F, Fut>(&self, executor: F) -> WarmupResult
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<String, String>>,
    {
        let _ = self.event_tx.send(WarmupEvent::Started).await;

        let result = self.script.run_warmup(executor).await;

        if result.success {
            let _ = self.event_tx.send(WarmupEvent::Completed(result.clone())).await;
        } else {
            let _ = self.event_tx.send(WarmupEvent::Failed("Warmup failed".to_string())).await;
        }

        result
    }
}

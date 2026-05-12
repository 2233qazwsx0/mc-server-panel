use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::error::ProcessError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogStatus {
    Active,
    Paused,
    Triggered,
    Recovering,
}

impl Default for WatchdogStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone)]
pub struct WatchdogEvent {
    pub timestamp: Instant,
    pub event_type: WatchdogEventType,
    pub instance_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogEventType {
    Heartbeat,
    Timeout,
    Recovered,
    CriticalFailure,
}

pub struct WatchdogConfig {
    pub timeout_duration: Duration,
    pub check_interval: Duration,
    pub max_retries: u32,
    pub auto_restart: bool,
    pub restart_delay: Duration,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            timeout_duration: Duration::from_secs(30),
            check_interval: Duration::from_secs(5),
            max_retries: 3,
            auto_restart: true,
            restart_delay: Duration::from_secs(5),
        }
    }
}

#[derive(Default)]
struct WatchdogState {
    status: WatchdogStatus,
    last_heartbeat: Option<Instant>,
    failure_count: u32,
    consecutive_timeouts: u32,
}

pub struct Watchdog {
    config: WatchdogConfig,
    state: Arc<RwLock<WatchdogState>>,
    event_tx: tokio::sync::broadcast::Sender<WatchdogEvent>,
    instance_id: String,
}

impl Watchdog {
    pub fn new(instance_id: String, config: WatchdogConfig) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(100);

        Self {
            config,
            state: Arc::new(RwLock::new(WatchdogState::default())),
            event_tx,
            instance_id,
        }
    }

    pub async fn start(&self) {
        let mut state = self.state.write().await;
        state.status = WatchdogStatus::Active;
        state.last_heartbeat = Some(Instant::now());
        info!("Watchdog started for instance: {}", self.instance_id);
    }

    pub async fn stop(&self) {
        let mut state = self.state.write().await;
        state.status = WatchdogStatus::Paused;
        info!("Watchdog stopped for instance: {}", self.instance_id);
    }

    pub async fn heartbeat(&self) {
        let mut state = self.state.write().await;
        state.last_heartbeat = Some(Instant::now());
        state.consecutive_timeouts = 0;

        if state.status == WatchdogStatus::Triggered {
            state.status = WatchdogStatus::Recovering;
            let _ = self.event_tx.send(WatchdogEvent {
                timestamp: Instant::now(),
                event_type: WatchdogEventType::Recovered,
                instance_id: self.instance_id.clone(),
                message: "Instance recovered after timeout".to_string(),
            });
        }

        state.status = WatchdogStatus::Active;
    }

    pub async fn check(&self) -> Result<bool, ProcessError> {
        let state = self.state.read().await;

        if state.status == WatchdogStatus::Paused {
            return Ok(true);
        }

        if let Some(last_heartbeat) = state.last_heartbeat {
            let elapsed = Instant::now().duration_since(last_heartbeat);

            if elapsed > self.config.timeout_duration {
                drop(state);

                let mut state = self.state.write().await;
                state.consecutive_timeouts += 1;
                state.status = WatchdogStatus::Triggered;

                let _ = self.event_tx.send(WatchdogEvent {
                    timestamp: Instant::now(),
                    event_type: WatchdogEventType::Timeout,
                    instance_id: self.instance_id.clone(),
                    message: format!(
                        "Watchdog timeout after {:?} (attempt {}/{})",
                        elapsed,
                        state.consecutive_timeouts,
                        self.config.max_retries
                    ),
                });

                warn!(
                    "Watchdog timeout detected for instance {}: {:?} elapsed",
                    self.instance_id,
                    elapsed
                );

                if state.consecutive_timeouts >= self.config.max_retries {
                    let _ = self.event_tx.send(WatchdogEvent {
                        timestamp: Instant::now(),
                        event_type: WatchdogEventType::CriticalFailure,
                        instance_id: self.instance_id.clone(),
                        message: "Maximum retry attempts reached".to_string(),
                    });

                    return Err(ProcessError::WatchdogTimeout(self.instance_id.clone()));
                }

                return Ok(false);
            }
        }

        let _ = self.event_tx.send(WatchdogEvent {
            timestamp: Instant::now(),
            event_type: WatchdogEventType::Heartbeat,
            instance_id: self.instance_id.clone(),
            message: "Heartbeat check passed".to_string(),
        });

        Ok(true)
    }

    pub async fn get_status(&self) -> WatchdogStatus {
        self.state.read().await.status
    }

    pub async fn get_failure_count(&self) -> u32 {
        self.state.read().await.failure_count
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<WatchdogEvent> {
        self.event_tx.subscribe()
    }

    pub fn get_config(&self) -> WatchdogConfig {
        self.config.clone()
    }

    pub async fn update_config(&mut self, config: WatchdogConfig) {
        self.config = config;
    }

    pub fn should_auto_restart(&self) -> bool {
        self.config.auto_restart
    }

    pub fn get_restart_delay(&self) -> Duration {
        self.config.restart_delay
    }
}

pub struct WatchdogManager {
    watchdogs: Arc<RwLock<std::collections::HashMap<String, Arc<Watchdog>>>>,
}

impl WatchdogManager {
    pub fn new() -> Self {
        Self {
            watchdogs: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn register_watchdog(&self, instance_id: String, watchdog: Arc<Watchdog>) {
        let mut watchdogs = self.watchdogs.write().await;
        watchdogs.insert(instance_id, watchdog);
    }

    pub async fn unregister_watchdog(&self, instance_id: &str) {
        let mut watchdogs = self.watchdogs.write().await;
        watchdogs.remove(instance_id);
    }

    pub async fn check_all(&self) -> Vec<(String, Result<bool, ProcessError>)> {
        let watchdogs = self.watchdogs.read().await;
        let mut results = Vec::new();

        for (id, watchdog) in watchdogs.iter() {
            let result = watchdog.check().await;
            results.push((id.clone(), result));
        }

        results
    }
}

impl Default for WatchdogManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_watchdog_creation() {
        let watchdog = Watchdog::new(
            "test-instance".to_string(),
            WatchdogConfig::default(),
        );

        assert_eq!(watchdog.get_status().await, WatchdogStatus::Paused);
    }

    #[tokio::test]
    async fn test_watchdog_start_and_heartbeat() {
        let watchdog = Watchdog::new(
            "test-instance".to_string(),
            WatchdogConfig::default(),
        );

        watchdog.start().await;
        assert_eq!(watchdog.get_status().await, WatchdogStatus::Active);

        watchdog.heartbeat().await;
        assert_eq!(watchdog.get_status().await, WatchdogStatus::Active);
    }

    #[tokio::test]
    async fn test_watchdog_check_passes() {
        let config = WatchdogConfig {
            timeout_duration: Duration::from_secs(10),
            ..Default::default()
        };
        let watchdog = Watchdog::new("test-instance".to_string(), config);

        watchdog.start().await;
        let result = watchdog.check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_watchdog_timeout() {
        let config = WatchdogConfig {
            timeout_duration: Duration::from_millis(50),
            max_retries: 2,
            ..Default::default()
        };
        let watchdog = Watchdog::new("test-instance".to_string(), config);

        watchdog.start().await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = watchdog.check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());

        let status = watchdog.get_status().await;
        assert_eq!(status, WatchdogStatus::Triggered);
    }

    #[tokio::test]
    async fn test_watchdog_max_retries() {
        let config = WatchdogConfig {
            timeout_duration: Duration::from_millis(50),
            max_retries: 1,
            ..Default::default()
        };
        let watchdog = Watchdog::new("test-instance".to_string(), config);

        watchdog.start().await;

        tokio::time::sleep(Duration::from_millis(100)).await;
        let _ = watchdog.check().await;

        let result = watchdog.check().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_watchdog_recovery() {
        let watchdog = Watchdog::new(
            "test-instance".to_string(),
            WatchdogConfig::default(),
        );

        watchdog.start().await;

        watchdog.heartbeat().await;
        assert_eq!(watchdog.get_status().await, WatchdogStatus::Active);
    }

    #[tokio::test]
    async fn test_watchdog_manager() {
        let manager = WatchdogManager::new();
        assert!(manager.check_all().await.is_empty());

        let watchdog = Arc::new(Watchdog::new(
            "instance1".to_string(),
            WatchdogConfig::default(),
        ));

        manager.register_watchdog("instance1".to_string(), watchdog.clone()).await;

        let results = manager.check_all().await;
        assert_eq!(results.len(), 1);
    }
}

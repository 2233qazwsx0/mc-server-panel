use crate::automation::{
    RestartStrategyConfig, TaskResult, TaskStatus,
};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct RestartStrategy {
    config: RestartStrategyConfig,
    last_restart: Option<DateTime<Utc>>,
    restart_count: Arc<RwLock<u32>>,
    cooldown_until: Arc<RwLock<Option<DateTime<Utc>>>>,
    shutdown_tx: broadcast::Sender<()>,
}

impl RestartStrategy {
    pub fn new(config: RestartStrategyConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(10);
        Self {
            config,
            last_restart: None,
            restart_count: Arc::new(RwLock::new(0)),
            cooldown_until: Arc::new(RwLock::new(None)),
            shutdown_tx,
        }
    }

    pub fn update_config(&mut self, config: RestartStrategyConfig) {
        self.config = config;
    }

    pub fn subscribe_shutdown(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    pub fn check_and_restart(
        &self,
        cpu_usage: f64,
        memory_usage_percent: f64,
        tps: f64,
        is_server_running: bool,
    ) -> Option<TaskResult> {
        if !self.config.enabled {
            return None;
        }

        if !is_server_running {
            if self.config.restart_on_crash {
                return Some(self.perform_restart("Server crashed", None));
            }
            return None;
        }

        if self.is_in_cooldown() {
            return None;
        }

        if self.config.restart_on_low_memory && memory_usage_percent >= self.config.memory_threshold_percent as f64 {
            return Some(self.perform_restart(
                "Low memory",
                Some(format!("Memory usage: {:.1}%", memory_usage_percent)),
            ));
        }

        if self.config.restart_on_low_tps && tps < self.config.tps_threshold {
            return Some(self.perform_restart(
                "Low TPS",
                Some(format!("TPS: {:.1} (threshold: {:.1})", tps, self.config.tps_threshold)),
            ));
        }

        None
    }

    fn perform_restart(&self, reason: &str, detail: Option<String>) -> TaskResult {
        let now = Utc::now();
        let cooldown = now + chrono::Duration::seconds(self.config.cooldown_seconds as i64);

        {
            let mut cooldown_lock = self.cooldown_until.write();
            *cooldown_lock = Some(cooldown);
        }

        {
            let mut count = self.restart_count.write();
            *count += 1;
        }

        let message = if let Some(d) = detail {
            format!("{}: {}", reason, d)
        } else {
            reason.to_string()
        };

        let _ = self.shutdown_tx.send(());

        self.last_restart = Some(now);

        info!("Auto-restart triggered: {}", message);

        TaskResult {
            success: true,
            message,
            duration_ms: 0,
            timestamp: now,
        }
    }

    fn is_in_cooldown(&self) -> bool {
        let cooldown = self.cooldown_until.read();
        if let Some(cooldown_time) = *cooldown {
            cooldown_time > Utc::now()
        } else {
            false
        }
    }

    pub fn get_status(&self) -> TaskStatus {
        TaskStatus {
            id: "restart_strategy".to_string(),
            name: "自动重启策略".to_string(),
            task_type: "restart_strategy".to_string(),
            enabled: self.config.enabled,
            last_run: self.last_restart,
            next_run: None,
            last_result: None,
            schedule: "on_condition".to_string(),
        }
    }

    pub fn get_stats(&self) -> RestartStats {
        RestartStats {
            total_restarts: *self.restart_count.read(),
            last_restart: self.last_restart,
            in_cooldown: self.is_in_cooldown(),
            cooldown_until: *self.cooldown_until.read(),
        }
    }

    pub fn reset_cooldown(&self) {
        let mut cooldown = self.cooldown_until.write();
        *cooldown = None;
    }

    pub fn force_restart(&self, reason: &str) -> TaskResult {
        self.perform_restart(reason, None)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RestartStats {
    pub total_restarts: u32,
    pub last_restart: Option<DateTime<Utc>>,
    pub in_cooldown: bool,
    pub cooldown_until: Option<DateTime<Utc>>,
}

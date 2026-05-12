use crate::automation::{
    BackupInfo, DiskInfo, TaskResult, TaskStatus, VersionInfo,
};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CronScheduler {
    tasks: Arc<RwLock<HashMap<String, CronTask>>>,
    event_tx: mpsc::Sender<CronEvent>,
}

#[derive(Debug, Clone)]
pub struct CronTask {
    pub id: String,
    pub name: String,
    pub task_type: CronTaskType,
    pub schedule: String,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_result: Option<TaskResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CronTaskType {
    Backup,
    LogCleanup,
    DiskCheck,
    UpdateCheck,
    Warmup,
    Custom(String),
}

impl std::fmt::Display for CronTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CronTaskType::Backup => write!(f, "backup"),
            CronTaskType::LogCleanup => write!(f, "log_cleanup"),
            CronTaskType::DiskCheck => write!(f, "disk_check"),
            CronTaskType::UpdateCheck => write!(f, "update_check"),
            CronTaskType::Warmup => write!(f, "warmup"),
            CronTaskType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CronEvent {
    TaskStarted(String),
    TaskCompleted(String, TaskResult),
    TaskFailed(String, String),
}

impl CronScheduler {
    pub fn new() -> Self {
        let (event_tx, _event_rx) = mpsc::channel(100);
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    pub fn subscribe(&self) -> mpsc::Receiver<CronEvent> {
        self.event_tx.subscribe()
    }

    pub fn add_task(&self, task: CronTask) {
        let mut tasks = self.tasks.write();
        let id = task.id.clone();
        tasks.insert(id, task);
        info!("Cron task added");
    }

    pub fn remove_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write();
        if tasks.remove(task_id).is_some() {
            info!("Cron task removed: {}", task_id);
            true
        } else {
            warn!("Cron task not found: {}", task_id);
            false
        }
    }

    pub fn get_task(&self, task_id: &str) -> Option<CronTask> {
        let tasks = self.tasks.read();
        tasks.get(task_id).cloned()
    }

    pub fn list_tasks(&self) -> Vec<CronTask> {
        let tasks = self.tasks.read();
        tasks.values().cloned().collect()
    }

    pub fn enable_task(&self, task_id: &str, enabled: bool) -> bool {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(task_id) {
            task.enabled = enabled;
            info!("Cron task {} {}", if enabled { "enabled" } else { "disabled" }, task_id);
            true
        } else {
            false
        }
    }

    pub fn update_schedule(&self, task_id: &str, schedule: &str) -> bool {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(task_id) {
            task.schedule = schedule.to_string();
            info!("Cron task {} schedule updated to {}", task_id, schedule);
            true
        } else {
            false
        }
    }

    pub fn record_task_start(&self, task_id: &str) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(task_id) {
            task.last_run = Some(Utc::now());
            let _ = self.event_tx.try_send(CronEvent::TaskStarted(task_id.to_string()));
        }
    }

    pub fn record_task_result(&self, task_id: &str, result: TaskResult) {
        let mut tasks = self.tasks.write();
        if let Some(task) = tasks.get_mut(task_id) {
            task.last_result = Some(result.clone());
            if result.success {
                let _ = self.event_tx.try_send(CronEvent::TaskCompleted(task_id.to_string(), result));
            } else {
                let _ = self.event_tx.try_send(CronEvent::TaskFailed(task_id.to_string(), result.message.clone()));
            }
        }
    }

    pub fn get_status_list(&self) -> Vec<TaskStatus> {
        let tasks = self.tasks.read();
        tasks
            .values()
            .map(|t| TaskStatus {
                id: t.id.clone(),
                name: t.name.clone(),
                task_type: t.task_type.to_string(),
                enabled: t.enabled,
                last_run: t.last_run,
                next_run: t.next_run,
                last_result: t.last_result.clone(),
                schedule: t.schedule.clone(),
            })
            .collect()
    }

    pub fn create_backup_task(&self, schedule: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let task = CronTask {
            id: id.clone(),
            name: "定时自动备份".to_string(),
            task_type: CronTaskType::Backup,
            schedule: schedule.to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
            last_result: None,
        };
        self.add_task(task);
        id
    }

    pub fn create_log_cleanup_task(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let task = CronTask {
            id: id.clone(),
            name: "日志自动清理".to_string(),
            task_type: CronTaskType::LogCleanup,
            schedule: "0 3 * * *".to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
            last_result: None,
        };
        self.add_task(task);
        id
    }

    pub fn create_disk_check_task(&self, schedule: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let task = CronTask {
            id: id.clone(),
            name: "磁盘空间检查".to_string(),
            task_type: CronTaskType::DiskCheck,
            schedule: schedule.to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
            last_result: None,
        };
        self.add_task(task);
        id
    }

    pub fn create_update_check_task(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let task = CronTask {
            id: id.clone(),
            name: "更新检查".to_string(),
            task_type: CronTaskType::UpdateCheck,
            schedule: "0 */6 * * *".to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
            last_result: None,
        };
        self.add_task(task);
        id
    }

    pub fn create_custom_task(&self, name: &str, schedule: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let task = CronTask {
            id: id.clone(),
            name: name.to_string(),
            task_type: CronTaskType::Custom(name.to_string()),
            schedule: schedule.to_string(),
            enabled: true,
            last_run: None,
            next_run: None,
            last_result: None,
        };
        self.add_task(task);
        id
    }

    pub fn parse_cron_expression(&self, expr: &str) -> Result<CronExpression, String> {
        CronExpression::parse(expr)
    }
}

impl Default for CronScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CronExpression {
    pub minute: String,
    pub hour: String,
    pub day_of_month: String,
    pub month: String,
    pub day_of_week: String,
}

#[derive(Debug, Clone)]
pub enum CronFieldValue {
    Any,
    Value(u8),
    Range(u8, u8),
    Step(u8),
    List(Vec<u8>),
}

impl CronExpression {
    pub fn parse(expr: &str) -> Result<Self, String> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(format!("Invalid cron expression: expected 5 fields, got {}", parts.len()));
        }

        Ok(Self {
            minute: parts[0].to_string(),
            hour: parts[1].to_string(),
            day_of_month: parts[2].to_string(),
            month: parts[3].to_string(),
            day_of_week: parts[4].to_string(),
        })
    }

    pub fn matches(&self, datetime: &chrono::DateTime<chrono::Utc>) -> bool {
        self.matches_minute(datetime.minute().clone())
            && self.matches_hour(datetime.hour() as u8)
            && self.matches_day_of_month(datetime.day() as u8)
            && self.matches_month(datetime.month() as u8)
            && self.matches_day_of_week(datetime.weekday().num_days_from_sunday() as u8)
    }

    fn matches_minute(&self, minute: u32) -> bool {
        self.field_matches(&self.minute, 0, 59, minute as u8)
    }

    fn matches_hour(&self, hour: u8) -> bool {
        self.field_matches(&self.hour, 0, 23, hour)
    }

    fn matches_day_of_month(&self, day: u8) -> bool {
        self.field_matches(&self.day_of_month, 1, 31, day)
    }

    fn matches_month(&self, month: u8) -> bool {
        self.field_matches(&self.month, 1, 12, month)
    }

    fn matches_day_of_week(&self, dow: u8) -> bool {
        self.field_matches(&self.day_of_week, 0, 6, dow)
    }

    fn field_matches(&self, field: &str, min: u8, max: u8, value: u8) -> bool {
        if field == "*" {
            return true;
        }

        if field.contains('/') {
            let parts: Vec<&str> = field.split('/').collect();
            if parts.len() != 2 {
                return false;
            }
            let base = if parts[0] == "*" { min } else { parts[0].parse().unwrap_or(min) };
            let step: u8 = parts[1].parse().unwrap_or(1);
            return (value - base) % step == 0 && value >= base;
        }

        if field.contains('-') {
            let parts: Vec<&str> = field.split('-').collect();
            if parts.len() != 2 {
                return false;
            }
            let start: u8 = parts[0].parse().unwrap_or(min);
            let end: u8 = parts[1].parse().unwrap_or(max);
            return value >= start && value <= end;
        }

        if field.contains(',') {
            let values: Vec<u8> = field
                .split(',')
                .filter_map(|s| s.parse().ok())
                .collect();
            return values.contains(&value);
        }

        if let Ok(v) = field.parse::<u8>() {
            return v == value;
        }

        false
    }

    pub fn next_run(&self, from: chrono::DateTime<chrono::Utc>) -> Option<chrono::DateTime<chrono::Utc>> {
        let mut current = from + chrono::Duration::minutes(1);

        for _ in 0..366 * 24 * 60 {
            if self.matches(&current) {
                return Some(current);
            }
            current = current + chrono::Duration::minutes(1);
        }

        None
    }
}

pub mod cron_integration {
    use super::*;
    use tokio::sync::broadcast;

    pub struct CronRunner {
        scheduler: Arc<CronScheduler>,
        shutdown_rx: broadcast::Receiver<()>,
    }

    impl CronRunner {
        pub fn new(scheduler: Arc<CronScheduler>) -> (Self, broadcast::Sender<()>) {
            let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
            (Self { scheduler, shutdown_rx }, shutdown_tx)
        }

        pub async fn run(&mut self) {
            info!("Cron scheduler started");

            loop {
                tokio::select! {
                    _ = self.shutdown_rx.recv() => {
                        info!("Cron scheduler shutting down");
                        break;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        self.check_and_run_tasks().await;
                    }
                }
            }
        }

        async fn check_and_run_tasks(&self) {
            let now = Utc::now();
            let tasks = self.scheduler.list_tasks();

            for task in tasks {
                if !task.enabled {
                    continue;
                }

                if let Ok(expr) = self.scheduler.parse_cron_expression(&task.schedule) {
                    if expr.matches(&now) {
                        info!("Cron task triggered: {} ({})", task.name, task.id);
                        self.scheduler.record_task_start(&task.id);
                    }
                }
            }
        }
    }
}

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use sysinfo::{Pid, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub system: SystemMetrics,
    pub process: Option<ProcessMetrics>,
    pub server_status: ServerStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

impl SystemMetrics {
    pub fn new(system: &System) -> Self {
        let memory_used = system.used_memory();
        let memory_total = system.total_memory();
        let memory_percent = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        Self {
            timestamp: Utc::now(),
            cpu_usage: system.global_cpu_usage(),
            memory_used,
            memory_total,
            memory_percent,
        }
    }

    pub fn empty() -> Self {
        Self {
            timestamp: Utc::now(),
            cpu_usage: 0.0,
            memory_used: 0,
            memory_total: 0,
            memory_percent: 0.0,
        }
    }

    pub fn memory_used_gb(&self) -> f64 {
        self.memory_used as f64 / 1024.0 / 1024.0 / 1024.0
    }

    pub fn memory_total_gb(&self) -> f64 {
        self.memory_total as f64 / 1024.0 / 1024.0 / 1024.0
    }
}

impl ProcessMetrics {
    pub fn from_system(system: &System, pid: u32) -> Option<Self> {
        let process = system.process(Pid::from_u32(pid))?;

        Some(Self {
            pid,
            cpu_usage: process.cpu_usage(),
            memory_used: process.memory(),
            name: process.name().to_string_lossy().to_string(),
            status: format!("{:?}", process.status()),
        })
    }
}

#[derive(Clone)]
pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
    history: Arc<RwLock<VecDeque<SystemMetrics>>>,
    history_size: usize,
}

impl SystemMonitor {
    pub fn new(history_size: usize) -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(history_size))),
            history_size,
        }
    }

    pub fn refresh(&self) {
        let mut sys = self.system.write();
        sys.refresh_all();
    }

    pub async fn collect(&self, server_pid: Option<u32>) -> MetricsSnapshot {
        {
            let mut sys = self.system.write();
            sys.refresh_all();
        }

        let system_metrics = self.get_system_metrics().await;

        let process_metrics = if let Some(pid) = server_pid {
            self.get_process_metrics(pid).await
        } else {
            None
        };

        let server_status = if process_metrics.is_some() {
            ServerStatus::Running
        } else {
            ServerStatus::Stopped
        };

        MetricsSnapshot {
            system: system_metrics.clone(),
            process: process_metrics,
            server_status,
        }
    }

    pub async fn get_system_metrics(&self) -> SystemMetrics {
        let sys = self.system.read();
        SystemMetrics::new(&sys)
    }

    pub async fn get_process_metrics(&self, pid: u32) -> Option<ProcessMetrics> {
        let sys = self.system.read();
        ProcessMetrics::from_system(&sys, pid)
    }

    pub async fn record(&self) -> SystemMetrics {
        let mut sys = self.system.write();
        sys.refresh_all();

        let metrics = SystemMetrics::new(&sys);

        let mut history = self.history.write();
        if history.len() >= self.history_size {
            history.pop_front();
        }
        history.push_back(metrics.clone());

        metrics
    }

    pub async fn get_history(&self, duration_secs: u64) -> Vec<SystemMetrics> {
        let history = self.history.read();
        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);

        history.iter()
            .filter(|m| m.timestamp > cutoff)
            .cloned()
            .collect()
    }

    pub async fn get_all_history(&self) -> Vec<SystemMetrics> {
        let history = self.history.read();
        history.iter().cloned().collect()
    }

    pub fn system_info(&self) -> SystemInfo {
        let sys = self.system.read();
        SystemInfo {
            name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            cpu_count: sys.cpus().len() as u32,
            total_memory: sys.total_memory(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub name: String,
    pub os: String,
    pub hostname: String,
    pub cpu_count: u32,
    pub total_memory: u64,
}

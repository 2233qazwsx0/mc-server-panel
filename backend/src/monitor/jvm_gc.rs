use crate::monitor::types::{GcMetrics, MetricType};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use sysinfo::{Pid, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmGcConfig {
    pub process_name: String,
    pub refresh_interval_secs: u64,
    pub history_size: usize,
}

impl Default for JvmGcConfig {
    fn default() -> Self {
        Self {
            process_name: "java".to_string(),
            refresh_interval_secs: 5,
            history_size: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmHeapStats {
    pub heap_init: u64,
    pub heap_used: u64,
    pub heap_committed: u64,
    pub heap_max: u64,
    pub non_heap_init: u64,
    pub non_heap_used: u64,
    pub non_heap_committed: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmMemoryPool {
    pub name: String,
    pub pool_type: String,
    pub initial: u64,
    pub used: u64,
    pub committed: u64,
    pub maximum: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmThreadInfo {
    pub thread_id: u64,
    pub thread_name: String,
    pub thread_state: String,
    pub blocked_count: u64,
    pub waited_count: u64,
    pub lock_name: Option<String>,
    pub lock_owner_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmGcInfo {
    pub gc_name: String,
    pub gc_cause: String,
    pub gc_action: String,
    pub pause_time_ms: f64,
    pub before_used: u64,
    pub after_used: u64,
    pub before_total: u64,
    pub after_total: u64,
    pub duration_ms: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmMetrics {
    pub pid: u32,
    pub uptime_secs: u64,
    pub heap: JvmHeapStats,
    pub memory_pools: Vec<JvmMemoryPool>,
    pub thread_count: u32,
    pub peak_thread_count: u32,
    pub daemon_thread_count: u32,
    pub gc_stats: Vec<JvmGcInfo>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone)]
pub struct JvmGcMonitor {
    gc_history: Arc<RwLock<VecDeque<GcMetrics>>>,
    heap_history: Arc<RwLock<VecDeque<JvmHeapStats>>>,
    config: JvmGcConfig,
    last_gc_events: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl JvmGcMonitor {
    pub fn new(config: JvmGcConfig) -> Self {
        Self {
            gc_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.history_size))),
            heap_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.history_size))),
            config,
            last_gc_events: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(JvmGcConfig::default())
    }

    pub fn get_heap_stats(&self, pid: u32, system: &System) -> Option<JvmHeapStats> {
        let process = system.process(Pid::from_u32(pid))?;

        let memory = process.memory();
        let total_memory = system.total_memory();

        let heap_ratio = if total_memory > 0 {
            (memory as f64 / total_memory as f64).min(0.8)
        } else {
            0.5
        };

        Some(JvmHeapStats {
            heap_init: memory / 2,
            heap_used: (memory as f64 * heap_ratio) as u64,
            heap_committed: memory,
            heap_max: total_memory,
            non_heap_init: memory / 10,
            non_heap_used: memory / 20,
            non_heap_committed: memory / 10,
            timestamp: Utc::now(),
        })
    }

    pub fn record_gc_event(&self, gc_metrics: GcMetrics) {
        let mut history = self.gc_history.write();
        if history.len() >= self.config.history_size {
            history.pop_front();
        }
        history.push_back(gc_metrics);
    }

    pub fn record_heap_snapshot(&self, heap_stats: JvmHeapStats) {
        let mut history = self.heap_history.write();
        if history.len() >= self.config.history_size {
            history.pop_front();
        }
        history.push_back(heap_stats);
    }

    pub fn get_gc_history(&self, limit: usize) -> Vec<GcMetrics> {
        let history = self.gc_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_heap_history(&self, limit: usize) -> Vec<JvmHeapStats> {
        let history = self.heap_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_total_gc_pause(&self, duration_secs: u64) -> f64 {
        let history = self.gc_history.read();
        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);

        history
            .iter()
            .filter(|gc| gc.timestamp > cutoff)
            .map(|gc| gc.pause_ms)
            .sum()
    }

    pub fn get_gc_count(&self, duration_secs: u64) -> usize {
        let history = self.gc_history.read();
        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);

        history.iter().filter(|gc| gc.timestamp > cutoff).count()
    }

    pub fn get_gc_pause_rate(&self, duration_secs: u64) -> f64 {
        let total_pause = self.get_total_gc_pause(duration_secs);
        total_pause / duration_secs as f64
    }

    pub fn is_gc_healthy(&self) -> bool {
        let gc_count = self.get_gc_count(60);
        let total_pause = self.get_total_gc_pause(60);

        gc_count < 100 && total_pause < 5000.0
    }

    pub fn simulate_gc_event(
        &self,
        gc_type: &str,
        pause_ms: f64,
        before_heap: u64,
        after_heap: u64,
    ) -> GcMetrics {
        let gc_metrics = GcMetrics {
            gc_type: gc_type.to_string(),
            pause_ms,
            before_heap,
            after_heap,
            timestamp: Utc::now(),
        };

        self.record_gc_event(gc_metrics.clone());
        gc_metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jvm_gc_monitor_creation() {
        let monitor = JvmGcMonitor::with_default_config();
        assert_eq!(monitor.config.process_name, "java");
        assert_eq!(monitor.config.refresh_interval_secs, 5);
    }

    #[test]
    fn test_record_gc_event() {
        let monitor = JvmGcMonitor::with_default_config();
        let gc = monitor.simulate_gc_event("GC", 100.0, 1000000, 500000);

        assert_eq!(gc.gc_type, "GC");
        assert_eq!(gc.pause_ms, 100.0);
        assert_eq!(gc.before_heap, 1000000);
        assert_eq!(gc.after_heap, 500000);
    }

    #[test]
    fn test_get_gc_history() {
        let monitor = JvmGcMonitor::with_default_config();

        for i in 0..5 {
            monitor.simulate_gc_event(&format!("GC-{}", i), i as f64 * 10.0, 1000, 500);
        }

        let history = monitor.get_gc_history(3);
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_gc_health_check() {
        let monitor = JvmGcMonitor::with_default_config();

        for i in 0..3 {
            monitor.simulate_gc_event(&format!("GC-{}", i), 50.0, 1000, 500);
        }

        assert!(monitor.is_gc_healthy());
    }

    #[test]
    fn test_gc_count_tracking() {
        let monitor = JvmGcMonitor::with_default_config();

        monitor.simulate_gc_event("GC", 100.0, 1000, 500);
        monitor.simulate_gc_event("GC", 100.0, 1000, 500);
        monitor.simulate_gc_event("GC", 100.0, 1000, 500);

        let count = monitor.get_gc_count(60);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_total_gc_pause() {
        let monitor = JvmGcMonitor::with_default_config();

        monitor.simulate_gc_event("GC", 100.0, 1000, 500);
        monitor.simulate_gc_event("GC", 200.0, 1000, 500);
        monitor.simulate_gc_event("GC", 300.0, 1000, 500);

        let total_pause = monitor.get_total_gc_pause(60);
        assert_eq!(total_pause, 600.0);
    }
}

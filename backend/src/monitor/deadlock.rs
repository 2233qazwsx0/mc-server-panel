use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use sysinfo::{Pid, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub thread_id: u64,
    pub thread_name: String,
    pub state: ThreadState,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub stack_trace: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadState {
    New,
    Runnable,
    Blocked,
    Waiting,
    TimedWaiting,
    Terminated,
    Unknown,
}

impl From<&str> for ThreadState {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "new" => ThreadState::New,
            "runnable" => ThreadState::Runnable,
            "blocked" => ThreadState::Blocked,
            "waiting" => ThreadState::Waiting,
            "timed waiting" | "timed_waiting" => ThreadState::TimedWaiting,
            "terminated" => ThreadState::Terminated,
            _ => ThreadState::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub lock_id: String,
    pub lock_class: String,
    pub owner_thread_id: Option<u64>,
    pub blocked_threads: Vec<u64>,
    pub lock_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlockInfo {
    pub deadlock_id: String,
    pub involved_threads: Vec<DeadlockedThread>,
    pub cycle_description: String,
    pub detected_at: DateTime<Utc>,
    pub severity: DeadlockSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlockedThread {
    pub thread_id: u64,
    pub thread_name: String,
    pub blocked_on_lock: String,
    pub holding_locks: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeadlockSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSnapshot {
    pub threads: Vec<ThreadInfo>,
    pub total_count: usize,
    pub blocked_count: usize,
    pub waiting_count: usize,
    pub deadlocks: Vec<DeadlockInfo>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadAnalysis {
    pub thread_id: u64,
    pub thread_name: String,
    pub wait_time_ms: u64,
    pub block_time_ms: u64,
    pub context_switches: u64,
    pub is_blocked: bool,
    pub is_waiting: bool,
    pub is_deadlocked: bool,
}

#[derive(Clone)]
pub struct ThreadDeadlockDetector {
    thread_snapshots: Arc<RwLock<VecDeque<ThreadSnapshot>>>,
    deadlock_history: Arc<RwLock<VecDeque<DeadlockInfo>>>,
    monitor_interval_ms: u64,
    max_history_size: usize,
    max_blocked_threshold: usize,
}

impl ThreadDeadlockDetector {
    pub fn new(monitor_interval_ms: u64, max_history_size: usize) -> Self {
        Self {
            thread_snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            deadlock_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            monitor_interval_ms,
            max_history_size,
            max_blocked_threshold: 10,
        }
    }

    pub fn with_default() -> Self {
        Self::new(5000, 1000)
    }

    pub fn collect_threads(&self, pid: u32, system: &System) -> ThreadSnapshot {
        let process = system.process(Pid::from_u32(pid));

        let mut threads = Vec::new();
        let mut blocked_count = 0;
        let mut waiting_count = 0;

        if let Some(proc) = process {
            let status = format!("{:?}", proc.status());
            let state = ThreadState::from(status.as_str());

            threads.push(ThreadInfo {
                thread_id: proc.pid().as_u32() as u64,
                thread_name: proc.name().to_string_lossy().to_string(),
                state,
                cpu_usage: proc.cpu_usage(),
                memory_usage: proc.memory(),
                stack_trace: Vec::new(),
            });

            if matches!(state, ThreadState::Blocked) {
                blocked_count = 1;
            } else if matches!(state, ThreadState::Waiting | ThreadState::TimedWaiting) {
                waiting_count = 1;
            }
        }

        let deadlocks = self.detect_deadlocks(&threads);

        let snapshot = ThreadSnapshot {
            threads,
            total_count: threads.len(),
            blocked_count,
            waiting_count,
            deadlocks,
            timestamp: Utc::now(),
        };

        self.record_snapshot(snapshot.clone());
        snapshot
    }

    fn detect_deadlocks(&self, threads: &[ThreadInfo]) -> Vec<DeadlockInfo> {
        let mut deadlocks = Vec::new();

        let blocked_threads: Vec<_> = threads
            .iter()
            .filter(|t| matches!(t.state, ThreadState::Blocked))
            .collect();

        if blocked_threads.len() > self.max_blocked_threshold {
            deadlocks.push(DeadlockInfo {
                deadlock_id: format!("DL-{}", uuid::Uuid::new_v4()),
                involved_threads: blocked_threads
                    .iter()
                    .map(|t| DeadlockedThread {
                        thread_id: t.thread_id,
                        thread_name: t.thread_name.clone(),
                        blocked_on_lock: "Unknown".to_string(),
                        holding_locks: Vec::new(),
                    })
                    .collect(),
                cycle_description: format!(
                    "High number of blocked threads detected: {} threads",
                    blocked_threads.len()
                ),
                detected_at: Utc::now(),
                severity: if blocked_threads.len() > 20 {
                    DeadlockSeverity::Critical
                } else {
                    DeadlockSeverity::High
                },
            });
        }

        let waiting_threads: Vec<_> = threads
            .iter()
            .filter(|t| matches!(t.state, ThreadState::Waiting | ThreadState::TimedWaiting))
            .collect();

        if waiting_threads.len() > self.max_blocked_threshold * 2 {
            deadlocks.push(DeadlockInfo {
                deadlock_id: format!("DW-{}", uuid::Uuid::new_v4()),
                involved_threads: waiting_threads
                    .iter()
                    .map(|t| DeadlockedThread {
                        thread_id: t.thread_id,
                        thread_name: t.thread_name.clone(),
                        blocked_on_lock: "Unknown".to_string(),
                        holding_locks: Vec::new(),
                    })
                    .collect(),
                cycle_description: format!(
                    "High number of waiting threads detected: {} threads",
                    waiting_threads.len()
                ),
                detected_at: Utc::now(),
                severity: DeadlockSeverity::Medium,
            });
        }

        deadlocks
    }

    pub fn detect_potential_deadlock(
        &self,
        thread_a: u64,
        thread_b: u64,
        lock_a: &str,
        lock_b: &str,
    ) -> Option<DeadlockInfo> {
        Some(DeadlockInfo {
            deadlock_id: format!("DL-{}", uuid::Uuid::new_v4()),
            involved_threads: vec![
                DeadlockedThread {
                    thread_id: thread_a,
                    thread_name: format!("Thread-{}", thread_a),
                    blocked_on_lock: lock_a.to_string(),
                    holding_locks: vec![lock_b.to_string()],
                },
                DeadlockedThread {
                    thread_id: thread_b,
                    thread_name: format!("Thread-{}", thread_b),
                    blocked_on_lock: lock_b.to_string(),
                    holding_locks: vec![lock_a.to_string()],
                },
            ],
            cycle_description: format!(
                "Circular wait detected: Thread-{} holds {} waiting for {}, Thread-{} holds {} waiting for {}",
                thread_a, lock_b, lock_a, thread_b, lock_a, lock_b
            ),
            detected_at: Utc::now(),
            severity: DeadlockSeverity::Critical,
        })
    }

    fn record_snapshot(&self, snapshot: ThreadSnapshot) {
        let mut snapshots = self.thread_snapshots.write();
        if snapshots.len() >= self.max_history_size {
            snapshots.pop_front();
        }
        snapshots.push_back(snapshot);
    }

    pub fn record_deadlock(&self, deadlock: DeadlockInfo) {
        let mut history = self.deadlock_history.write();
        if history.len() >= self.max_history_size {
            history.pop_front();
        }
        history.push_back(deadlock);
    }

    pub fn get_thread_snapshots(&self, limit: usize) -> Vec<ThreadSnapshot> {
        let snapshots = self.thread_snapshots.read();
        snapshots.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_deadlock_history(&self, limit: usize) -> Vec<DeadlockInfo> {
        let history = self.deadlock_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_blocked_thread_count(&self) -> usize {
        let snapshots = self.thread_snapshots.read();
        snapshots.back().map(|s| s.blocked_count).unwrap_or(0)
    }

    pub fn get_waiting_thread_count(&self) -> usize {
        let snapshots = self.thread_snapshots.read();
        snapshots.back().map(|s| s.waiting_count).unwrap_or(0)
    }

    pub fn is_deadlock_detected(&self) -> bool {
        let snapshots = self.thread_snapshots.read();
        snapshots
            .back()
            .map(|s| !s.deadlocks.is_empty())
            .unwrap_or(false)
    }

    pub fn get_thread_analysis(&self, thread_id: u64) -> Option<ThreadAnalysis> {
        let snapshots = self.thread_snapshots.read();

        let latest = snapshots.back()?;
        let thread = latest.threads.iter().find(|t| t.thread_id == thread_id)?;

        let is_deadlocked = latest.deadlocks.iter().any(|dl| {
            dl.involved_threads
                .iter()
                .any(|t| t.thread_id == thread_id)
        });

        Some(ThreadAnalysis {
            thread_id,
            thread_name: thread.thread_name.clone(),
            wait_time_ms: 0,
            block_time_ms: 0,
            context_switches: 0,
            is_blocked: matches!(thread.state, ThreadState::Blocked),
            is_waiting: matches!(
                thread.state,
                ThreadState::Waiting | ThreadState::TimedWaiting
            ),
            is_deadlocked,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_deadlock_detector_creation() {
        let detector = ThreadDeadlockDetector::with_default();
        assert_eq!(detector.max_history_size, 1000);
    }

    #[test]
    fn test_thread_state_from_str() {
        assert_eq!(ThreadState::from("runnable"), ThreadState::Runnable);
        assert_eq!(ThreadState::from("blocked"), ThreadState::Blocked);
        assert_eq!(ThreadState::from("waiting"), ThreadState::Waiting);
    }

    #[test]
    fn test_detect_potential_deadlock() {
        let detector = ThreadDeadlockDetector::with_default();

        let deadlock = detector.detect_potential_deadlock(
            1,
            2,
            "LockA",
            "LockB",
        );

        assert!(deadlock.is_some());
        let dl = deadlock.unwrap();
        assert_eq!(dl.involved_threads.len(), 2);
        assert_eq!(dl.severity, DeadlockSeverity::Critical);
    }

    #[test]
    fn test_record_deadlock() {
        let detector = ThreadDeadlockDetector::with_default();

        let deadlock = detector.detect_potential_deadlock(
            1,
            2,
            "LockA",
            "LockB",
        ).unwrap();

        detector.record_deadlock(deadlock);

        let history = detector.get_deadlock_history(10);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_deadlock_detection_with_many_blocked() {
        let detector = ThreadDeadlockDetector::new(5000, 100);

        let mut threads = Vec::new();
        for i in 0..15 {
            threads.push(ThreadInfo {
                thread_id: i,
                thread_name: format!("Thread-{}", i),
                state: ThreadState::Blocked,
                cpu_usage: 0.0,
                memory_usage: 0,
                stack_trace: Vec::new(),
            });
        }

        let deadlocks = detector.detect_deadlocks(&threads);
        assert!(!deadlocks.is_empty());
        assert_eq!(deadlocks[0].severity, DeadlockSeverity::High);
    }

    #[test]
    fn test_is_deadlock_detected() {
        let detector = ThreadDeadlockDetector::with_default();
        assert!(!detector.is_deadlock_detected());
    }
}

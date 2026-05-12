use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BruteForceConfig {
    pub max_attempts: u32,
    pub lockout_duration_secs: u64,
    pub alert_threshold: u32,
    pub reset_duration_secs: u64,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            lockout_duration_secs: 900,
            alert_threshold: 3,
            reset_duration_secs: 3600,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttemptRecord {
    pub ip: String,
    pub attempts: Vec<AttemptInfo>,
    pub locked_until: Option<Instant>,
    pub total_attempts: u32,
    pub successful_auths: u32,
    pub first_attempt: Instant,
    pub last_attempt: Instant,
}

#[derive(Debug, Clone)]
pub struct AttemptInfo {
    pub timestamp: Instant,
    pub success: bool,
    pub method: String,
}

impl AttemptRecord {
    pub fn new(ip: String) -> Self {
        let now = Instant::now();
        Self {
            ip,
            attempts: Vec::new(),
            locked_until: None,
            total_attempts: 0,
            successful_auths: 0,
            first_attempt: now,
            last_attempt: now,
        }
    }

    pub fn is_locked(&self) -> bool {
        if let Some(until) = self.locked_until {
            return Instant::now() < until;
        }
        false
    }

    pub fn get_remaining_lock_time(&self) -> Option<Duration> {
        self.locked_until
            .map(|until| until.saturating_duration_since(Instant::now()))
            .filter(|d| !d.is_zero())
    }

    pub fn lock(&mut self, duration: Duration) {
        self.locked_until = Some(Instant::now() + duration);
    }

    pub fn record_attempt(&mut self, success: bool, method: &str) {
        self.attempts.push(AttemptInfo {
            timestamp: Instant::now(),
            success,
            method: method.to_string(),
        });
        self.total_attempts += 1;
        self.last_attempt = Instant::now();
        if success {
            self.successful_auths += 1;
        }
    }

    pub fn get_failed_attempts(&self) -> u32 {
        self.attempts
            .iter()
            .filter(|a| !a.success)
            .filter(|a| a.timestamp > Instant::now() - Duration::from_secs(3600))
            .count() as u32
    }

    pub fn cleanup_old_attempts(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(86400);
        self.attempts.retain(|a| a.timestamp > cutoff);
    }
}

#[derive(Clone)]
pub struct BruteForceGuard {
    records: Arc<RwLock<HashMap<String, AttemptRecord>>>,
    config: Arc<RwLock<BruteForceConfig>>,
    alerts: Arc<RwLock<Vec<BruteForceAlert>>>,
}

impl BruteForceGuard {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(BruteForceConfig::default())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn record_attempt(&self, ip: &str, success: bool, method: &str) -> BruteForceResult {
        let ip = ip.to_string();
        let mut records = self.records.write();
        let record = records
            .entry(ip.clone())
            .or_insert_with(|| AttemptRecord::new(ip.clone()));

        if record.is_locked() {
            return BruteForceResult::Locked {
                remaining_secs: record
                    .get_remaining_lock_time()
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                reason: "too many failed attempts".to_string(),
            };
        }

        record.record_attempt(success, method);
        let config = self.config.read().clone();

        if success {
            if record.get_failed_attempts() > 0
                && record.get_failed_attempts() < config.max_attempts
            {
                record.locked_until = None;
            }
            return BruteForceResult::Success;
        }

        let failed_count = record.get_failed_attempts();

        if failed_count >= config.max_attempts {
            record.lock(Duration::from_secs(config.lockout_duration_secs));
            self.generate_alert(&ip, failed_count, true);
            return BruteForceResult::Locked {
                remaining_secs: config.lockout_duration_secs,
                reason: format!("exceeded {} failed attempts", config.max_attempts),
            };
        }

        if failed_count >= config.alert_threshold {
            self.generate_alert(&ip, failed_count, false);
        }

        BruteForceResult::Failed {
            attempts_remaining: config.max_attempts - failed_count,
            locked: false,
        }
    }

    pub fn check_ip(&self, ip: &str) -> bool {
        let records = self.records.read();
        if let Some(record) = records.get(ip) {
            !record.is_locked()
        } else {
            true
        }
    }

    pub fn unlock_ip(&self, ip: &str) -> Result<(), String> {
        let mut records = self.records.write();
        if let Some(record) = records.get_mut(ip) {
            record.locked_until = None;
            record.attempts.clear();
            Ok(())
        } else {
            Err("IP not found in records".to_string())
        }
    }

    pub fn unlock_all(&self) {
        let mut records = self.records.write();
        for record in records.values_mut() {
            record.locked_until = None;
        }
    }

    pub fn remove_ip(&self, ip: &str) {
        self.records.write().remove(ip);
    }

    pub fn get_record(&self, ip: &str) -> Option<AttemptRecord> {
        self.records.read().get(ip).cloned()
    }

    pub fn get_all_records(&self) -> Vec<AttemptRecord> {
        self.records.read().values().cloned().collect()
    }

    pub fn get_locked_ips(&self) -> Vec<LockedIpInfo> {
        self.records
            .read()
            .iter()
            .filter(|(_, r)| r.is_locked())
            .map(|(ip, r)| LockedIpInfo {
                ip: ip.clone(),
                locked_until: r.locked_until.map(|t| {
                    chrono::DateTime::<chrono::Utc>::from_timestamp(t.elapsed().as_secs() as i64, 0)
                        .unwrap_or_else(chrono::Utc::now)
                }),
                failed_attempts: r.get_failed_attempts(),
                total_attempts: r.total_attempts,
            })
            .collect()
    }

    pub fn get_alerts(&self) -> Vec<BruteForceAlert> {
        self.alerts.read().clone()
    }

    pub fn clear_alerts(&self) {
        self.alerts.write().clear();
    }

    pub fn get_stats(&self) -> BruteForceStats {
        let records = self.records.read();
        let locked_count = records.values().filter(|r| r.is_locked()).count();
        let total_attempts: u64 = records.values().map(|r| r.total_attempts as u64).sum();
        let total_failures: u64 = records
            .values()
            .map(|r| r.get_failed_attempts() as u64)
            .sum();

        BruteForceStats {
            tracked_ips: records.len(),
            locked_ips: locked_count,
            total_attempts,
            total_failures,
            alerts_count: self.alerts.read().len(),
        }
    }

    pub fn set_config(&self, config: BruteForceConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> BruteForceConfig {
        self.config.read().clone()
    }

    pub fn cleanup_old_records(&self) {
        let cutoff = Instant::now() - Duration::from_secs(86400);
        let mut records = self.records.write();
        records.retain(|_, r| {
            r.cleanup_old_attempts();
            r.last_attempt > cutoff || r.is_locked()
        });
    }

    fn generate_alert(&self, ip: &str, attempts: u32, locked: bool) {
        let alert = BruteForceAlert {
            ip: ip.to_string(),
            timestamp: chrono::Utc::now(),
            failed_attempts: attempts,
            action_taken: if locked {
                "locked".to_string()
            } else {
                "warning".to_string()
            },
            severity: if locked {
                AlertSeverity::High
            } else {
                AlertSeverity::Medium
            },
        };
        self.alerts.write().push(alert);

        if self.alerts.read().len() > 1000 {
            self.alerts.write().drain(0..500);
        }
    }
}

impl Default for BruteForceGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BruteForceResult {
    Success,
    Failed {
        attempts_remaining: u32,
        locked: bool,
    },
    Locked {
        remaining_secs: u64,
        reason: String,
    },
}

impl BruteForceResult {
    pub fn is_success(&self) -> bool {
        matches!(self, BruteForceResult::Success)
    }

    pub fn is_locked(&self) -> bool {
        matches!(self, BruteForceResult::Locked { .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BruteForceAlert {
    pub ip: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub failed_attempts: u32,
    pub action_taken: String,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedIpInfo {
    pub ip: String,
    pub locked_until: Option<chrono::DateTime<chrono::Utc>>,
    pub failed_attempts: u32,
    pub total_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BruteForceStats {
    pub tracked_ips: usize,
    pub locked_ips: usize,
    pub total_attempts: u64,
    pub total_failures: u64,
    pub alerts_count: usize,
}

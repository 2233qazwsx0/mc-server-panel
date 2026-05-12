use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
    pub block_duration_secs: u64,
    pub cleanup_interval_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_size: 10,
            block_duration_secs: 300,
            cleanup_interval_secs: 60,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestRecord {
    pub ip: String,
    pub minute_count: u32,
    pub hour_count: u32,
    pub minute_timestamps: Vec<Instant>,
    pub hour_timestamps: Vec<Instant>,
    pub blocked_until: Option<Instant>,
    pub first_seen: Instant,
    pub last_seen: Instant,
}

impl RequestRecord {
    pub fn new(ip: String) -> Self {
        let now = Instant::now();
        Self {
            ip,
            minute_count: 0,
            hour_count: 0,
            minute_timestamps: Vec::new(),
            hour_timestamps: Vec::new(),
            blocked_until: None,
            first_seen: now,
            last_seen: now,
        }
    }

    pub fn is_blocked(&self) -> bool {
        if let Some(until) = self.blocked_until {
            return Instant::now() < until;
        }
        false
    }

    pub fn block(&mut self, duration: Duration) {
        self.blocked_until = Some(Instant::now() + duration);
    }

    pub fn cleanup_old_timestamps(&mut self) {
        let minute_cutoff = Instant::now() - Duration::from_secs(60);
        let hour_cutoff = Instant::now() - Duration::from_secs(3600);

        self.minute_timestamps.retain(|t| *t > minute_cutoff);
        self.hour_timestamps.retain(|t| *t > hour_cutoff);

        self.minute_count = self.minute_timestamps.len() as u32;
        self.hour_count = self.hour_timestamps.len() as u32;
    }
}

#[derive(Clone)]
pub struct DdosGuard {
    records: Arc<RwLock<HashMap<String, RequestRecord>>>,
    config: Arc<RwLock<RateLimitConfig>>,
    total_requests: Arc<RwLock<u64>>,
    blocked_ips: Arc<RwLock<HashMap<String, Instant>>>,
}

impl DdosGuard {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(RateLimitConfig::default())),
            total_requests: Arc::new(RwLock::new(0)),
            blocked_ips: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn check_request(&self, ip: &str) -> DdosCheckResult {
        let ip = match self.normalize_ip(ip) {
            Ok(ip) => ip,
            Err(e) => return DdosCheckResult::Invalid { reason: e },
        };

        *self.total_requests.write() += 1;

        let mut records = self.records.write();
        let record = records.entry(ip.clone()).or_insert_with(|| RequestRecord::new(ip.clone()));

        if record.is_blocked() {
            return DdosCheckResult::Blocked {
                remaining_secs: record
                    .blocked_until
                    .map(|t| t.saturating_duration_since(Instant::now()).as_secs())
                    .unwrap_or(0),
                reason: "rate limit exceeded".to_string(),
            };
        }

        record.cleanup_old_timestamps();
        record.last_seen = Instant::now();
        record.minute_timestamps.push(Instant::now());
        record.minute_count += 1;
        record.hour_timestamps.push(Instant::now());
        record.hour_count += 1;

        let config = self.config.read().clone();

        if record.minute_count > config.requests_per_minute {
            record.block(Duration::from_secs(config.block_duration_secs));
            self.blocked_ips.write().insert(ip.clone(), Instant::now());
            return DdosCheckResult::RateLimited {
                limit_type: "minute".to_string(),
                current: record.minute_count,
                limit: config.requests_per_minute,
            };
        }

        if record.hour_count > config.requests_per_hour {
            record.block(Duration::from_secs(config.block_duration_secs));
            self.blocked_ips.write().insert(ip.clone(), Instant::now());
            return DdosCheckResult::RateLimited {
                limit_type: "hour".to_string(),
                current: record.hour_count,
                limit: config.requests_per_hour,
            };
        }

        let burst_size = record.minute_timestamps.len();
        if burst_size as u32 > config.burst_size {
            let recent_count = record
                .minute_timestamps
                .iter()
                .filter(|t| t.elapsed() < Duration::from_secs(10))
                .count();

            if recent_count as u32 > config.burst_size {
                record.block(Duration::from_secs(config.block_duration_secs));
                self.blocked_ips.write().insert(ip.clone(), Instant::now());
                return DdosCheckResult::RateLimited {
                    limit_type: "burst".to_string(),
                    current: recent_count as u32,
                    limit: config.burst_size,
                };
            }
        }

        DdosCheckResult::Allowed
    }

    pub fn get_record(&self, ip: &str) -> Option<RequestRecord> {
        self.records.read().get(ip).cloned()
    }

    pub fn get_all_records(&self) -> Vec<RequestRecord> {
        self.records.read().values().cloned().collect()
    }

    pub fn unblock_ip(&self, ip: &str) -> Result<(), String> {
        let ip = self.normalize_ip(ip)?;
        let mut records = self.records.write();
        if let Some(record) = records.get_mut(&ip) {
            record.blocked_until = None;
        }
        self.blocked_ips.write().remove(&ip);
        Ok(())
    }

    pub fn clear_ip(&self, ip: &str) -> Result<(), String> {
        let ip = self.normalize_ip(ip)?;
        self.records.write().remove(&ip);
        self.blocked_ips.write().remove(&ip);
        Ok(())
    }

    pub fn set_config(&self, config: RateLimitConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> RateLimitConfig {
        self.config.read().clone()
    }

    pub fn get_stats(&self) -> DdosStats {
        let records = self.records.read();
        let blocked_ips = self.blocked_ips.read();
        let total_requests = *self.total_requests.read();

        let active_ips = records.len();
        let currently_blocked = blocked_ips.len();

        let high_traffic_ips: Vec<_> = records
            .iter()
            .filter(|(_, r)| r.minute_count > 30 || r.hour_count > 200)
            .map(|(ip, r)| HighTrafficIp {
                ip: ip.clone(),
                minute_requests: r.minute_count,
                hour_requests: r.hour_count,
            })
            .collect();

        DdosStats {
            total_requests,
            active_ips,
            currently_blocked,
            high_traffic_ips,
        }
    }

    pub fn cleanup_expired(&self) {
        let cutoff = Instant::now() - Duration::from_secs(3600);
        let mut records = self.records.write();
        records.retain(|_, r| {
            r.cleanup_old_timestamps();
            r.last_seen > cutoff || r.is_blocked()
        });

        let mut blocked = self.blocked_ips.write();
        blocked.retain(|_, t| *t > cutoff);
    }

    fn normalize_ip(&self, ip: &str) -> Result<String, String> {
        ip.parse::<IpAddr>()
            .map(|a| a.to_string())
            .map_err(|_| format!("Invalid IP address: {}", ip))
    }
}

impl Default for DdosGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DdosCheckResult {
    Allowed,
    RateLimited {
        limit_type: String,
        current: u32,
        limit: u32,
    },
    Blocked {
        remaining_secs: u64,
        reason: String,
    },
    Invalid {
        reason: String,
    },
}

impl DdosCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, DdosCheckResult::Allowed)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(
            self,
            DdosCheckResult::Blocked { .. } | DdosCheckResult::RateLimited { .. }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosStats {
    pub total_requests: u64,
    pub active_ips: usize,
    pub currently_blocked: usize,
    pub high_traffic_ips: Vec<HighTrafficIp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighTrafficIp {
    pub ip: String,
    pub minute_requests: u32,
    pub hour_requests: u32,
}

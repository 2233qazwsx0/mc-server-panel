use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceStats {
    pub interface: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub rx_packets_per_sec: u64,
    pub tx_packets_per_sec: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthUsage {
    pub interface: String,
    pub current_rx_bps: u64,
    pub current_tx_bps: u64,
    pub avg_rx_bps: u64,
    pub avg_tx_bps: u64,
    pub peak_rx_bps: u64,
    pub peak_tx_bps: u64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub utilization_percent: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnectionInfo {
    pub protocol: Protocol,
    pub local_addr: String,
    pub remote_addr: Option<String>,
    pub state: ConnectionState,
    pub process_id: Option<u32>,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Established,
    Listen,
    TimeWait,
    CloseWait,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    Closing,
    Closed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAlertConfig {
    pub alert_type: NetworkAlertType,
    pub threshold_bps: u64,
    pub interface: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkAlertType {
    RxBandwidthHigh,
    TxBandwidthHigh,
    TotalBandwidthHigh,
    PacketLoss,
    ErrorRate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub interfaces: Vec<NetworkInterfaceStats>,
    pub total_rx_bps: u64,
    pub total_tx_bps: u64,
    pub active_connections: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone)]
pub struct NetworkMonitor {
    interface_history: Arc<RwLock<VecDeque<NetworkInterfaceStats>>>,
    bandwidth_history: Arc<RwLock<HashMap<String, VecDeque<BandwidthUsage>>>>,
    connection_history: Arc<RwLock<VecDeque<NetworkConnectionInfo>>>, 
    max_history_size: usize,
    thresholds: Arc<RwLock<Vec<NetworkAlertConfig>>>,
    last_sample: Arc<RwLock<Option<DateTime<Utc>>>>,
    last_stats: Arc<RwLock<HashMap<String, (u64, u64, u64, u64)>>>,
    peak_rx: Arc<RwLock<u64>>,
    peak_tx: Arc<RwLock<u64>>,
}

impl NetworkMonitor {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            interface_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            bandwidth_history: Arc::new(RwLock::new(HashMap::new())),
            connection_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            max_history_size,
            thresholds: Arc::new(RwLock::new(Vec::new())),
            last_sample: Arc::new(RwLock::new(None)),
            last_stats: Arc::new(RwLock::new(HashMap::new())),
            peak_rx: Arc::new(RwLock::new(0)),
            peak_tx: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_default() -> Self {
        Self::new(1000)
    }

    pub fn collect_network_stats(&self, _system: &System) -> NetworkSnapshot {
        let now = Utc::now();
        let mut interfaces = Vec::new();

        let networks = sysinfo::Networks::new_with_refreshed_list();
        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;

        for (interface_name, data) in networks.iter() {
            let rx_bytes = data.received();
            let tx_bytes = data.transmitted();
            let rx_packets = data.packets_received();
            let tx_packets = data.packets_transmitted();

            let mut current = NetworkInterfaceStats {
                interface: interface_name.clone(),
                rx_bytes_per_sec: 0,
                tx_bytes_per_sec: 0,
                rx_packets_per_sec: 0,
                tx_packets_per_sec: 0,
                rx_errors: data.errors_on_received(),
                tx_errors: data.errors_on_transmitted(),
                rx_dropped: data.dropped_on_received(),
                tx_dropped: data.dropped_on_transmitted(),
                timestamp: now,
            };

            let mut last_time = self.last_sample.write();
            let mut last_stats_map = self.last_stats.write();

            if let Some(last) = *last_time {
                let elapsed = (now - last).num_milliseconds() as f64 / 1000.0;
                if elapsed > 0.0 {
                    if let Some((_, _, prev_rx_packets, prev_tx_packets)) =
                        last_stats_map.get(interface_name)
                    {
                        let rx_diff = rx_bytes.saturating_sub(data.received());
                        let tx_diff = tx_bytes.saturating_sub(data.transmitted());
                        let rx_pkt_diff = rx_packets.saturating_sub(*prev_rx_packets);
                        let tx_pkt_diff = tx_packets.saturating_sub(*prev_tx_packets);

                        current.rx_bytes_per_sec = (rx_diff as f64 / elapsed) as u64;
                        current.tx_bytes_per_sec = (tx_diff as f64 / elapsed) as u64;
                        current.rx_packets_per_sec = (rx_pkt_diff as f64 / elapsed) as u64;
                        current.tx_packets_per_sec = (tx_pkt_diff as f64 / elapsed) as u64;
                    }
                }
            }

            last_stats_map.insert(
                interface_name.clone(),
                (rx_bytes, tx_bytes, rx_packets, tx_packets),
            );
            *last_time = Some(now);

            drop(last_time);
            drop(last_stats_map);

            total_rx += current.rx_bytes_per_sec;
            total_tx += current.tx_bytes_per_sec;

            {
                let mut history = self.interface_history.write();
                if history.len() >= self.max_history_size {
                    history.pop_front();
                }
                history.push_back(current.clone());
            }

            {
                let mut bandwidths = self.bandwidth_history.write();
                let entry = bandwidths
                    .entries
                    .entry(interface_name.clone())
                    .or_insert_with(|| VecDeque::with_capacity(self.max_history_size));
                if entry.len() >= self.max_history_size {
                    entry.pop_front();
                }

                let mut peak_rx = self.peak_rx.write();
                let mut peak_tx = self.peak_tx.write();
                if current.rx_bytes_per_sec > *peak_rx {
                    *peak_rx = current.rx_bytes_per_sec;
                }
                if current.tx_bytes_per_sec > *peak_tx {
                    *peak_tx = current.tx_bytes_per_sec;
                }

                let usage = BandwidthUsage {
                    interface: interface_name.clone(),
                    current_rx_bps: current.rx_bytes_per_sec,
                    current_tx_bps: current.tx_bytes_per_sec,
                    avg_rx_bps: 0,
                    avg_tx_bps: 0,
                    peak_rx_bps: *peak_rx,
                    peak_tx_bps: *peak_tx,
                    total_rx_bytes: rx_bytes,
                    total_tx_bytes: tx_bytes,
                    utilization_percent: 0.0,
                    timestamp: now,
                };
                entry.push_back(usage);
            }

            interfaces.push(current);
        }

        NetworkSnapshot {
            interfaces,
            total_rx_bps: total_rx,
            total_tx_bps: total_tx,
            active_connections: 0,
            timestamp: now,
        }
    }

    pub fn get_interface_stats(&self, interface: &str) -> Option<NetworkInterfaceStats> {
        let history = self.interface_history.read();
        history.iter().rev().find(|s| s.interface == interface).cloned()
    }

    pub fn get_bandwidth_usage(&self, interface: &str) -> Option<BandwidthUsage> {
        let bandwidths = self.bandwidth_history.read();
        let history = bandwidths.get(interface)?;
        history.back().cloned()
    }

    pub fn get_bandwidth_history(&self, interface: &str, limit: usize) -> Vec<BandwidthUsage> {
        let bandwidths = self.bandwidth_history.read();
        let history = match bandwidths.get(interface) {
            Some(h) => h,
            None => return Vec::new(),
        };
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_total_bandwidth(&self, duration_secs: u64) -> (u64, u64) {
        let history = self.interface_history.read();
        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);

        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;

        for stat in history.iter() {
            if stat.timestamp > cutoff {
                total_rx += stat.rx_bytes_per_sec;
                total_tx += stat.tx_bytes_per_sec;
            }
        }

        (total_rx, total_tx)
    }

    pub fn get_peak_bandwidth(&self) -> (u64, u64) {
        let peak_rx = self.peak_rx.read();
        let peak_tx = self.peak_tx.read();
        (*peak_rx, *peak_tx)
    }

    pub fn reset_peak_bandwidth(&self) {
        let mut peak_rx = self.peak_rx.write();
        let mut peak_tx = self.peak_tx.write();
        *peak_rx = 0;
        *peak_tx = 0;
    }

    pub fn add_alert_config(&self, config: NetworkAlertConfig) {
        let mut thresholds = self.thresholds.write();
        thresholds.push(config);
    }

    pub fn check_alerts(&self, snapshot: &NetworkSnapshot) -> Vec<(NetworkAlertType, u64, u64)> {
        let thresholds = self.thresholds.read();
        let mut alerts = Vec::new();

        for config in thresholds.iter() {
            if !config.enabled {
                continue;
            }

            match config.alert_type {
                NetworkAlertType::RxBandwidthHigh => {
                    let rx = if let Some(ref iface) = config.interface {
                        snapshot
                            .interfaces
                            .iter()
                            .find(|s| s.interface == *iface)
                            .map(|s| s.rx_bytes_per_sec)
                            .unwrap_or(0)
                    } else {
                        snapshot.total_rx_bps
                    };

                    if rx > config.threshold_bps {
                        alerts.push((NetworkAlertType::RxBandwidthHigh, rx, config.threshold_bps));
                    }
                }
                NetworkAlertType::TxBandwidthHigh => {
                    let tx = if let Some(ref iface) = config.interface {
                        snapshot
                            .interfaces
                            .iter()
                            .find(|s| s.interface == *iface)
                            .map(|s| s.tx_bytes_per_sec)
                            .unwrap_or(0)
                    } else {
                        snapshot.total_tx_bps
                    };

                    if tx > config.threshold_bps {
                        alerts.push((NetworkAlertType::TxBandwidthHigh, tx, config.threshold_bps));
                    }
                }
                NetworkAlertType::TotalBandwidthHigh => {
                    let total = snapshot.total_rx_bps + snapshot.total_tx_bps;
                    if total > config.threshold_bps {
                        alerts.push((
                            NetworkAlertType::TotalBandwidthHigh,
                            total,
                            config.threshold_bps,
                        ));
                    }
                }
                NetworkAlertType::ErrorRate => {
                    for iface in &snapshot.interfaces {
                        let total_errors = iface.rx_errors + iface.tx_errors;
                        if total_errors > 0 && config.interface.as_ref() == Some(&iface.interface) {
                            alerts.push((
                                NetworkAlertType::ErrorRate,
                                total_errors,
                                config.threshold_bps,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        alerts
    }

    pub fn get_connection_stats(&self) -> HashMap<ConnectionState, usize> {
        let connections = self.connection_history.read();
        let mut stats = HashMap::new();

        for conn in connections.iter() {
            *stats.entry(conn.state).or_insert(0) += 1;
        }

        stats
    }

    pub fn format_bandwidth(bps: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bps >= GB {
            format!("{:.2} GB/s", bps as f64 / GB as f64)
        } else if bps >= MB {
            format!("{:.2} MB/s", bps as f64 / MB as f64)
        } else if bps >= KB {
            format!("{:.2} KB/s", bps as f64 / KB as f64)
        } else {
            format!("{} B/s", bps)
        }
    }

    pub fn get_network_summary(&self) -> NetworkSummary {
        let history = self.interface_history.read();
        let latest = history.back();

        let (total_rx, total_tx) = if let Some(stats) = latest {
            (stats.rx_bytes_per_sec, stats.tx_bytes_per_sec)
        } else {
            (0, 0)
        };

        let (peak_rx, peak_tx) = self.get_peak_bandwidth();

        NetworkSummary {
            current_rx_bps: total_rx,
            current_tx_bps: total_tx,
            peak_rx_bps: peak_rx,
            peak_tx_bps: peak_tx,
            formatted_rx: Self::format_bandwidth(total_rx),
            formatted_tx: Self::format_bandwidth(total_tx),
            formatted_peak_rx: Self::format_bandwidth(peak_rx),
            formatted_peak_tx: Self::format_bandwidth(peak_tx),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub current_rx_bps: u64,
    pub current_tx_bps: u64,
    pub peak_rx_bps: u64,
    pub peak_tx_bps: u64,
    pub formatted_rx: String,
    pub formatted_tx: String,
    pub formatted_peak_rx: String,
    pub formatted_peak_tx: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_monitor_creation() {
        let monitor = NetworkMonitor::with_default();
        assert_eq!(monitor.max_history_size, 1000);
    }

    #[test]
    fn test_format_bandwidth() {
        assert_eq!(NetworkMonitor::format_bandwidth(500), "500 B/s");
        assert_eq!(NetworkMonitor::format_bandwidth(1024), "1.00 KB/s");
        assert_eq!(NetworkMonitor::format_bandwidth(1024 * 1024), "1.00 MB/s");
        assert_eq!(
            NetworkMonitor::format_bandwidth(1024 * 1024 * 1024),
            "1.00 GB/s"
        );
    }

    #[test]
    fn test_get_peak_bandwidth() {
        let monitor = NetworkMonitor::with_default();
        let (rx, tx) = monitor.get_peak_bandwidth();
        assert_eq!(rx, 0);
        assert_eq!(tx, 0);
    }

    #[test]
    fn test_reset_peak_bandwidth() {
        let monitor = NetworkMonitor::with_default();
        monitor.reset_peak_bandwidth();
        let (rx, tx) = monitor.get_peak_bandwidth();
        assert_eq!(rx, 0);
        assert_eq!(tx, 0);
    }

    #[test]
    fn test_add_alert_config() {
        let monitor = NetworkMonitor::with_default();

        monitor.add_alert_config(NetworkAlertConfig {
            alert_type: NetworkAlertType::RxBandwidthHigh,
            threshold_bps: 100 * 1024 * 1024,
            interface: Some("eth0".to_string()),
            enabled: true,
        });

        let thresholds = monitor.thresholds.read();
        assert_eq!(thresholds.len(), 1);
    }

    #[test]
    fn test_check_alerts() {
        let monitor = NetworkMonitor::with_default();

        monitor.add_alert_config(NetworkAlertConfig {
            alert_type: NetworkAlertType::TotalBandwidthHigh,
            threshold_bps: 10 * 1024 * 1024,
            interface: None,
            enabled: true,
        });

        let snapshot = NetworkSnapshot {
            interfaces: vec![],
            total_rx_bps: 5 * 1024 * 1024,
            total_tx_bps: 5 * 1024 * 1024,
            active_connections: 0,
            timestamp: Utc::now(),
        };

        let alerts = monitor.check_alerts(&snapshot);
        assert!(alerts.is_empty());

        let high_snapshot = NetworkSnapshot {
            interfaces: vec![],
            total_rx_bps: 10 * 1024 * 1024,
            total_tx_bps: 10 * 1024 * 1024,
            active_connections: 0,
            timestamp: Utc::now(),
        };

        let alerts = monitor.check_alerts(&high_snapshot);
        assert_eq!(alerts.len(), 1);
    }
}

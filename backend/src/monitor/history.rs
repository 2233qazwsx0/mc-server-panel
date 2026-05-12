use crate::monitor::types::{MetricDataPoint, MetricType, PerformanceSnapshot};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsAggregation {
    pub metric_type: MetricType,
    pub aggregation_type: AggregationType,
    pub value: f64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationType {
    Min,
    Max,
    Avg,
    Sum,
    Count,
    P50,
    P95,
    P99,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsQuery {
    pub metric_types: Vec<MetricType>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub aggregation: Option<AggregationType>,
    pub interval_secs: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsTimeSeries {
    pub metric_type: MetricType,
    pub data_points: Vec<DataPoint>,
    pub stats: MetricsStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub sum: f64,
    pub count: usize,
    pub std_dev: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsExport {
    pub format: ExportFormat,
    pub metrics: Vec<MetricType>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
    Prometheus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsRetention {
    pub metric_type: MetricType,
    pub retention_days: u32,
    pub aggregation_interval_secs: u64,
}

#[derive(Clone)]
pub struct MetricsHistory {
    snapshots: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
    data_points: Arc<RwLock<HashMap<MetricType, VecDeque<MetricDataPoint>>>>,
    retention_policy: Arc<RwLock<Vec<MetricsRetention>>>,
    max_history_size: usize,
    aggregations: Arc<RwLock<HashMap<MetricType, VecDeque<MetricsAggregation>>>>,
}

impl MetricsHistory {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            data_points: Arc::new(RwLock::new(HashMap::new())),
            retention_policy: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
            aggregations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_default() -> Self {
        Self::new(10000)
    }

    pub fn record_snapshot(&self, snapshot: PerformanceSnapshot) {
        let mut snapshots = self.snapshots.write();
        if snapshots.len() >= self.max_history_size {
            snapshots.pop_front();
        }
        snapshots.push_back(snapshot.clone());
    }

    pub fn record_metric(&self, metric_type: MetricType, value: f64, tags: Option<HashMap<String, String>>) {
        let mut data = self.data_points.write();

        let entry = data.entry(metric_type).or_insert_with(|| {
            VecDeque::with_capacity(self.max_history_size)
        });

        if entry.len() >= self.max_history_size {
            entry.pop_front();
        }

        entry.push_back(MetricDataPoint {
            timestamp: Utc::now(),
            metric_type,
            value,
            tags: tags.unwrap_or_default(),
        });
    }

    pub fn record_batch(&self, metrics: &HashMap<MetricType, f64>) {
        for (metric_type, value) in metrics {
            self.record_metric(*metric_type, *value, None);
        }
    }

    pub fn get_snapshot(&self, timestamp: &DateTime<Utc>) -> Option<PerformanceSnapshot> {
        let snapshots = self.snapshots.read();
        snapshots.iter().find(|s| s.timestamp == *timestamp).cloned()
    }

    pub fn get_latest_snapshot(&self) -> Option<PerformanceSnapshot> {
        let snapshots = self.snapshots.read();
        snapshots.back().cloned()
    }

    pub fn get_snapshots(&self, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Vec<PerformanceSnapshot> {
        let snapshots = self.snapshots.read();
        snapshots
            .iter()
            .filter(|s| s.timestamp >= *start && s.timestamp <= *end)
            .cloned()
            .collect()
    }

    pub fn get_metric_history(
        &self,
        metric_type: MetricType,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Vec<MetricDataPoint> {
        let data = self.data_points.read();
        let points = match data.get(&metric_type) {
            Some(p) => p,
            None => return Vec::new(),
        };

        points
            .iter()
            .filter(|p| p.timestamp >= *start && p.timestamp <= *end)
            .cloned()
            .collect()
    }

    pub fn get_latest_value(&self, metric_type: MetricType) -> Option<f64> {
        let data = self.data_points.read();
        data.get(&metric_type).and_then(|points| points.back().map(|p| p.value))
    }

    pub fn calculate_stats(&self, metric_type: MetricType, duration_secs: u64) -> Option<MetricsStats> {
        let data = self.data_points.read();
        let points = data.get(&metric_type)?;

        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);
        let filtered: Vec<_> = points
            .iter()
            .filter(|p| p.timestamp > cutoff)
            .collect();

        if filtered.is_empty() {
            return None;
        }

        let values: Vec<f64> = filtered.iter().map(|p| p.value).collect();
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let variance = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Some(MetricsStats {
            min,
            max,
            avg,
            sum,
            count,
            std_dev,
        })
    }

    pub fn aggregate(
        &self,
        metric_type: MetricType,
        aggregation: AggregationType,
        duration_secs: u64,
    ) -> Option<MetricsAggregation> {
        let data = self.data_points.read();
        let points = data.get(&metric_type)?;

        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);
        let filtered: Vec<_> = points
            .iter()
            .filter(|p| p.timestamp > cutoff)
            .collect();

        if filtered.is_empty() {
            return None;
        }

        let values: Vec<f64> = filtered.iter().map(|p| p.value).collect();
        let value = match aggregation {
            AggregationType::Min => values.iter().cloned().fold(f64::INFINITY, f64::min),
            AggregationType::Max => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            AggregationType::Avg => values.iter().sum::<f64>() / values.len() as f64,
            AggregationType::Sum => values.iter().sum(),
            AggregationType::Count => values.len() as f64,
            AggregationType::P50 => {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = (sorted.len() as f64 * 0.5) as usize;
                sorted[idx.min(sorted.len() - 1)]
            }
            AggregationType::P95 => {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = (sorted.len() as f64 * 0.95) as usize;
                sorted[idx.min(sorted.len() - 1)]
            }
            AggregationType::P99 => {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = (sorted.len() as f64 * 0.99) as usize;
                sorted[idx.min(sorted.len() - 1)]
            }
        };

        Some(MetricsAggregation {
            metric_type,
            aggregation_type: aggregation,
            value,
            start_time: cutoff,
            end_time: Utc::now(),
        })
    }

    pub fn query(&self, query: MetricsQuery) -> Vec<MetricsTimeSeries> {
        let mut result = Vec::new();

        for metric_type in &query.metric_types {
            let data = self.data_points.read();
            let points = match data.get(metric_type) {
                Some(p) => p,
                None => continue,
            };

            let filtered: Vec<_> = points
                .iter()
                .filter(|p| p.timestamp >= query.start_time && p.timestamp <= query.end_time)
                .cloned()
                .collect();

            let data_points: Vec<DataPoint> = if let Some(interval) = query.interval_secs {
                Self::resample_points(&filtered, interval)
            } else {
                filtered.iter().map(|p| DataPoint {
                    timestamp: p.timestamp,
                    value: p.value,
                }).collect()
            };

            let values: Vec<f64> = data_points.iter().map(|p| p.value).collect();
            let stats = if !values.is_empty() {
                let sum: f64 = values.iter().sum();
                let avg = sum / values.len() as f64;
                let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let variance = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;

                MetricsStats {
                    min,
                    max,
                    avg,
                    sum,
                    count: values.len(),
                    std_dev: variance.sqrt(),
                }
            } else {
                MetricsStats {
                    min: 0.0,
                    max: 0.0,
                    avg: 0.0,
                    sum: 0.0,
                    count: 0,
                    std_dev: 0.0,
                }
            };

            result.push(MetricsTimeSeries {
                metric_type: *metric_type,
                data_points,
                stats,
            });
        }

        result
    }

    fn resample_points(points: &[MetricDataPoint], interval_secs: u64) -> Vec<DataPoint> {
        if points.is_empty() {
            return Vec::new();
        }

        let mut buckets: HashMap<i64, Vec<f64>> = HashMap::new();
        let interval_ms = (interval_secs * 1000) as i64;

        for point in points {
            let bucket = point.timestamp.timestamp_millis() / interval_ms;
            buckets.entry(bucket).or_default().push(point.value);
        }

        let mut result: Vec<DataPoint> = buckets
            .into_iter()
            .map(|(bucket, values)| {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                DataPoint {
                    timestamp: DateTime::from_timestamp_millis(bucket * interval_ms).unwrap_or_else(Utc::now),
                    value: avg,
                }
            })
            .collect();

        result.sort_by_key(|p| p.timestamp);
        result
    }

    pub fn export(&self, export: MetricsExport) -> Result<String, String> {
        match export.format {
            ExportFormat::Json => self.export_json(export),
            ExportFormat::Csv => self.export_csv(export),
            ExportFormat::Prometheus => self.export_prometheus(export),
        }
    }

    fn export_json(&self, export: MetricsExport) -> Result<String, String> {
        let data = self.data_points.read();
        let mut result = Vec::new();

        for metric_type in &export.metrics {
            let points = match data.get(metric_type) {
                Some(p) => p,
                None => continue,
            };

            let filtered: Vec<_> = points
                .iter()
                .filter(|p| p.timestamp >= export.start_time && p.timestamp <= export.end_time)
                .collect();

            let series = serde_json::json!({
                "metric": metric_type.as_str(),
                "points": filtered.iter().map(|p| {
                    serde_json::json!({
                        "timestamp": p.timestamp.to_rfc3339(),
                        "value": p.value
                    })
                }).collect::<Vec<_>>()
            });

            result.push(series);
        }

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    fn export_csv(&self, export: MetricsExport) -> Result<String, String> {
        let data = self.data_points.read();
        let mut output = String::from("timestamp,metric,value\n");

        for metric_type in &export.metrics {
            let points = match data.get(metric_type) {
                Some(p) => p,
                None => continue,
            };

            let filtered: Vec<_> = points
                .iter()
                .filter(|p| p.timestamp >= export.start_time && p.timestamp <= export.end_time)
                .collect();

            for point in filtered {
                output.push_str(&format!(
                    "{},{},{}\n",
                    point.timestamp.to_rfc3339(),
                    metric_type.as_str(),
                    point.value
                ));
            }
        }

        Ok(output)
    }

    fn export_prometheus(&self, export: MetricsExport) -> Result<String, String> {
        let data = self.data_points.read();
        let mut output = String::new();

        for metric_type in &export.metrics {
            let points = match data.get(metric_type) {
                Some(p) => p,
                None => continue,
            };

            let filtered: Vec<_> = points
                .iter()
                .filter(|p| p.timestamp >= export.start_time && p.timestamp <= export.end_time)
                .collect();

            for point in filtered {
                let metric_name = format!("mc_server_{}", metric_type.as_str().replace('_', "_"));
                output.push_str(&format!(
                    "{} {} {}\n",
                    metric_name,
                    point.value,
                    point.timestamp.timestamp()
                ));
            }
        }

        Ok(output)
    }

    pub fn set_retention(&self, metric_type: MetricType, retention_days: u32, interval_secs: u64) {
        let mut policy = self.retention_policy.write();

        if let Some(existing) = policy.iter_mut().find(|r| r.metric_type == metric_type) {
            existing.retention_days = retention_days;
            existing.aggregation_interval_secs = interval_secs;
        } else {
            policy.push(MetricsRetention {
                metric_type,
                retention_days,
                aggregation_interval_secs: interval_secs,
            });
        }
    }

    pub fn cleanup_old_data(&self) -> usize {
        let mut snapshots = self.snapshots.write();
        let mut removed = 0;

        let cutoff = Utc::now() - chrono::Duration::days(7);

        while snapshots.front().map(|s| s.timestamp < cutoff).unwrap_or(false) {
            snapshots.pop_front();
            removed += 1;
        }

        removed
    }

    pub fn get_history_size(&self) -> usize {
        let snapshots = self.snapshots.read();
        snapshots.len()
    }

    pub fn get_metric_count(&self) -> usize {
        let data = self.data_points.read();
        data.len()
    }

    pub fn clear_metric(&self, metric_type: MetricType) {
        let mut data = self.data_points.write();
        data.remove(&metric_type);
    }

    pub fn clear_all(&self) {
        let mut snapshots = self.snapshots.write();
        let mut data = self.data_points.write();
        let mut aggregations = self.aggregations.write();

        snapshots.clear();
        data.clear();
        aggregations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_history_creation() {
        let history = MetricsHistory::with_default();
        assert_eq!(history.max_history_size, 10000);
    }

    #[test]
    fn test_record_and_get_metric() {
        let history = MetricsHistory::with_default();

        history.record_metric(MetricType::CpuUsage, 50.0, None);
        history.record_metric(MetricType::MemoryPercent, 75.0, None);

        let latest_cpu = history.get_latest_value(MetricType::CpuUsage);
        let latest_mem = history.get_latest_value(MetricType::MemoryPercent);

        assert_eq!(latest_cpu, Some(50.0));
        assert_eq!(latest_mem, Some(75.0));
    }

    #[test]
    fn test_calculate_stats() {
        let history = MetricsHistory::with_default();

        for i in 1..=10 {
            history.record_metric(MetricType::CpuUsage, i as f64 * 10.0, None);
        }

        let stats = history.calculate_stats(MetricType::CpuUsage, 60);
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 100.0);
        assert!((stats.avg - 55.0).abs() < 0.01);
    }

    #[test]
    fn test_aggregate() {
        let history = MetricsHistory::with_default();

        for i in 1..=100 {
            history.record_metric(MetricType::CpuUsage, i as f64, None);
        }

        let min_agg = history.aggregate(MetricType::CpuUsage, AggregationType::Min, 60);
        let max_agg = history.aggregate(MetricType::CpuUsage, AggregationType::Max, 60);
        let avg_agg = history.aggregate(MetricType::CpuUsage, AggregationType::Avg, 60);

        assert_eq!(min_agg.map(|a| a.value), Some(1.0));
        assert_eq!(max_agg.map(|a| a.value), Some(100.0));
        assert!((avg_agg.unwrap().value - 50.5).abs() < 0.1);
    }

    #[test]
    fn test_query() {
        let history = MetricsHistory::with_default();

        let now = Utc::now();
        let start = now - chrono::Duration::minutes(5);
        let end = now + chrono::Duration::minutes(1);

        history.record_metric(MetricType::CpuUsage, 50.0, None);
        history.record_metric(MetricType::MemoryPercent, 60.0, None);

        let query = MetricsQuery {
            metric_types: vec![MetricType::CpuUsage, MetricType::MemoryPercent],
            start_time: start,
            end_time: end,
            aggregation: None,
            interval_secs: Some(60),
            limit: Some(100),
        };

        let result = history.query(query);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_export_json() {
        let history = MetricsHistory::with_default();
        history.record_metric(MetricType::CpuUsage, 50.0, None);

        let export = MetricsExport {
            format: ExportFormat::Json,
            metrics: vec![MetricType::CpuUsage],
            start_time: Utc::now() - chrono::Duration::hours(1),
            end_time: Utc::now() + chrono::Duration::hours(1),
        };

        let result = history.export(export);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("cpu_usage"));
    }

    #[test]
    fn test_export_csv() {
        let history = MetricsHistory::with_default();
        history.record_metric(MetricType::CpuUsage, 50.0, None);

        let export = MetricsExport {
            format: ExportFormat::Csv,
            metrics: vec![MetricType::CpuUsage],
            start_time: Utc::now() - chrono::Duration::hours(1),
            end_time: Utc::now() + chrono::Duration::hours(1),
        };

        let result = history.export(export);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("timestamp,metric,value"));
    }
}

use crate::monitor::types::{BaselineMetrics, MetricType};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineConfig {
    pub metric_type: MetricType,
    pub learning_window_secs: u64,
    pub min_samples: usize,
    pub auto_update: bool,
    pub update_interval_secs: u64,
    pub sensitivity: BaselineSensitivity,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineSensitivity {
    Low,
    Medium,
    High,
}

impl BaselineSensitivity {
    pub fn deviation_threshold(&self) -> f64 {
        match self {
            BaselineSensitivity::Low => 3.0,
            BaselineSensitivity::Medium => 2.0,
            BaselineSensitivity::High => 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineProfile {
    pub id: String,
    pub name: String,
    pub metric_type: MetricType,
    pub baselines: HashMap<String, BaselineMetrics>,
    pub time_windows: Vec<TimeWindowBaseline>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindowBaseline {
    pub window_name: String,
    pub start_hour: u8,
    pub end_hour: u8,
    pub baseline: BaselineMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub id: String,
    pub metric_type: MetricType,
    pub baseline_id: String,
    pub current_value: f64,
    pub expected_min: f64,
    pub expected_max: f64,
    pub deviation_score: f64,
    pub severity: AnomalySeverity,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    pub metric_type: MetricType,
    pub total_samples: usize,
    pub learning_window_secs: u64,
    pub last_update: DateTime<Utc>,
    pub convergence_score: f64,
}

#[derive(Clone)]
pub struct BaselineLearner {
    profiles: Arc<RwLock<HashMap<String, BaselineProfile>>>,
    samples: Arc<RwLock<HashMap<MetricType, VecDeque<f64>>>>,
    configs: Arc<RwLock<HashMap<MetricType, BaselineConfig>>>,
    anomaly_history: Arc<RwLock<VecDeque<AnomalyAlert>>>,
    learning_stats: Arc<RwLock<HashMap<MetricType, LearningStats>>>,
    max_samples: usize,
    max_anomaly_history: usize,
}

impl BaselineLearner {
    pub fn new(max_samples: usize, max_anomaly_history: usize) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            samples: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            anomaly_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_anomaly_history))),
            learning_stats: Arc::new(RwLock::new(HashMap::new())),
            max_samples,
            max_anomaly_history,
        }
    }

    pub fn with_default() -> Self {
        Self::new(10000, 1000)
    }

    pub fn configure(&self, config: BaselineConfig) {
        let mut configs = self.configs.write();
        configs.insert(config.metric_type, config);
    }

    pub fn get_config(&self, metric_type: MetricType) -> Option<BaselineConfig> {
        let configs = self.configs.read();
        configs.get(&metric_type).cloned()
    }

    pub fn add_sample(&self, metric_type: MetricType, value: f64) {
        let mut samples = self.samples.write();
        let entry = samples.entry(metric_type).or_insert_with(|| {
            VecDeque::with_capacity(self.max_samples)
        });

        if entry.len() >= self.max_samples {
            entry.pop_front();
        }
        entry.push_back(value);
    }

    pub fn add_samples(&self, metric_type: MetricType, values: &[f64]) {
        for value in values {
            self.add_sample(metric_type, *value);
        }
    }

    pub fn calculate_baseline(&self, metric_type: MetricType) -> Option<BaselineMetrics> {
        let samples = self.samples.read();
        let values = samples.get(&metric_type)?;

        if values.is_empty() {
            return None;
        }

        let values_vec: Vec<f64> = values.iter().cloned().collect();
        let n = values_vec.len() as f64;

        let sum: f64 = values_vec.iter().sum();
        let mean = sum / n;

        let variance = values_vec.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let mut sorted = values_vec.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = sorted.first().copied().unwrap_or(0.0);
        let max = sorted.last().copied().unwrap_or(0.0);

        let p50_idx = ((n * 0.5) as usize).min(sorted.len() - 1);
        let p95_idx = ((n * 0.95) as usize).min(sorted.len() - 1);
        let p99_idx = ((n * 0.99) as usize).min(sorted.len() - 1);

        Some(BaselineMetrics {
            metric_type,
            mean,
            std_dev,
            min,
            max,
            p50: sorted[p50_idx],
            p95: sorted[p95_idx],
            p99: sorted[p99_idx],
            sample_count: values_vec.len(),
            updated_at: Utc::now(),
        })
    }

    pub fn update_baseline(&self, profile_id: &str) -> Result<BaselineMetrics, String> {
        let profiles = self.profiles.read();
        let profile = profiles.get(profile_id).ok_or_else(|| "Profile not found".to_string())?;
        let metric_type = profile.metric_type;
        drop(profiles);

        let baseline = self.calculate_baseline(metric_type).ok_or_else(|| "Not enough samples".to_string())?;

        let mut profiles = self.profiles.write();
        let profile = profiles.get_mut(profile_id).ok_or_else(|| "Profile not found".to_string())?;

        profile.baselines.insert("global".to_string(), baseline.clone());
        profile.updated_at = Utc::now();

        Ok(baseline)
    }

    pub fn create_profile(&self, name: &str, metric_type: MetricType) -> BaselineProfile {
        let profile = BaselineProfile {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            metric_type,
            baselines: HashMap::new(),
            time_windows: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut profiles = self.profiles.write();
        profiles.insert(profile.id.clone(), profile.clone());

        profile
    }

    pub fn get_profile(&self, id: &str) -> Option<BaselineProfile> {
        let profiles = self.profiles.read();
        profiles.get(id).cloned()
    }

    pub fn get_all_profiles(&self) -> Vec<BaselineProfile> {
        let profiles = self.profiles.read();
        profiles.values().cloned().collect()
    }

    pub fn delete_profile(&self, id: &str) -> Option<BaselineProfile> {
        let mut profiles = self.profiles.write();
        profiles.remove(id)
    }

    pub fn calculate_deviation(&self, metric_type: MetricType, value: f64) -> Option<f64> {
        let baseline = self.calculate_baseline(metric_type)?;

        if baseline.std_dev == 0.0 {
            return Some(0.0);
        }

        let deviation = (value - baseline.mean).abs() / baseline.std_dev;
        Some(deviation)
    }

    pub fn is_anomaly(&self, metric_type: MetricType, value: f64) -> Option<AnomalyDetection> {
        let baseline = self.calculate_baseline(metric_type)?;
        let deviation = self.calculate_deviation(metric_type, value)?;

        let configs = self.configs.read();
        let config = configs.get(&metric_type);
        let threshold = config
            .map(|c| c.sensitivity.deviation_threshold())
            .unwrap_or(2.0);

        let is_anomaly = deviation > threshold;

        let expected_min = baseline.mean - (baseline.std_dev * threshold);
        let expected_max = baseline.mean + (baseline.std_dev * threshold);

        Some(AnomalyDetection {
            metric_type,
            current_value: value,
            expected_range: (expected_min, expected_max),
            deviation_score: deviation,
            is_anomaly,
            timestamp: Utc::now(),
        })
    }

    pub fn record_anomaly(&self, anomaly: AnomalyAlert) {
        let mut history = self.anomaly_history.write();
        if history.len() >= self.max_anomaly_history {
            history.pop_front();
        }
        history.push_back(anomaly);
    }

    pub fn detect_and_record_anomaly(&self, metric_type: MetricType, value: f64) -> Option<AnomalyAlert> {
        let detection = self.is_anomaly(metric_type, value)?;

        if !detection.is_anomaly {
            return None;
        }

        let profiles = self.profiles.read();
        let profile = profiles.values().find(|p| p.metric_type == metric_type);
        let baseline_id = profile.map(|p| p.id.clone()).unwrap_or_default();
        drop(profiles);

        let severity = if detection.deviation_score > 4.0 {
            AnomalySeverity::Critical
        } else if detection.deviation_score > 3.0 {
            AnomalySeverity::High
        } else if detection.deviation_score > 2.0 {
            AnomalySeverity::Medium
        } else {
            AnomalySeverity::Low
        };

        let anomaly = AnomalyAlert {
            id: uuid::Uuid::new_v4().to_string(),
            metric_type,
            baseline_id,
            current_value: value,
            expected_min: detection.expected_range.0,
            expected_max: detection.expected_range.1,
            deviation_score: detection.deviation_score,
            severity,
            timestamp: Utc::now(),
            acknowledged: false,
        };

        self.record_anomaly(anomaly.clone());
        Some(anomaly)
    }

    pub fn get_anomaly_history(&self, limit: usize) -> Vec<AnomalyAlert> {
        let history = self.anomaly_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_recent_anomalies(&self, duration_secs: u64) -> Vec<AnomalyAlert> {
        let history = self.anomaly_history.read();
        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);

        history
            .iter()
            .filter(|a| a.timestamp > cutoff)
            .cloned()
            .collect()
    }

    pub fn acknowledge_anomaly(&self, anomaly_id: &str) -> Result<(), String> {
        let mut history = self.anomaly_history.write();
        let anomaly = history.iter_mut().find(|a| a.id == anomaly_id)
            .ok_or_else(|| "Anomaly not found".to_string())?;
        anomaly.acknowledged = true;
        Ok(())
    }

    pub fn calculate_time_window_baseline(
        &self,
        metric_type: MetricType,
        window_name: &str,
        start_hour: u8,
        end_hour: u8,
    ) -> Option<BaselineMetrics> {
        let samples = self.samples.read();
        let values = samples.get(&metric_type)?;

        let current_hour = chrono::Utc::now().format("%H").to_string().parse::<u8>().unwrap_or(0);
        if current_hour < start_hour || current_hour >= end_hour {
            return None;
        }

        let window_values: Vec<f64> = values.iter().copied().collect();

        if window_values.is_empty() {
            return None;
        }

        let n = window_values.len() as f64;
        let sum: f64 = window_values.iter().sum();
        let mean = sum / n;

        let variance = window_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let mut sorted = window_values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sorted_len = sorted.len();
        let p50_idx = ((sorted_len as f64 * 0.5) as usize).min(sorted_len.saturating_sub(1));
        let p95_idx = ((sorted_len as f64 * 0.95) as usize).min(sorted_len.saturating_sub(1));
        let p99_idx = ((sorted_len as f64 * 0.99) as usize).min(sorted_len.saturating_sub(1));

        Some(BaselineMetrics {
            metric_type,
            mean,
            std_dev,
            min: *sorted.first().unwrap_or(&0.0),
            max: *sorted.last().unwrap_or(&0.0),
            p50: sorted[p50_idx],
            p95: sorted[p95_idx],
            p99: sorted[p99_idx],
            sample_count: window_values.len(),
            updated_at: chrono::Utc::now(),
        })
    }

    pub fn get_learning_stats(&self, metric_type: MetricType) -> Option<LearningStats> {
        let samples = self.samples.read();
        let count = samples.get(&metric_type).map(|v| v.len()).unwrap_or(0);
        drop(samples);

        let configs = self.configs.read();
        let config = configs.get(&metric_type);

        let stats = LearningStats {
            metric_type,
            total_samples: count,
            learning_window_secs: config.map(|c| c.learning_window_secs).unwrap_or(3600),
            last_update: Utc::now(),
            convergence_score: self.calculate_convergence_score(metric_type),
        };

        Some(stats)
    }

    fn calculate_convergence_score(&self, metric_type: MetricType) -> f64 {
        let samples = self.samples.read();
        let values = match samples.get(&metric_type) {
            Some(v) => v,
            None => return 0.0,
        };

        if values.len() < 100 {
            return 0.0;
        }

        let values_vec: Vec<f64> = values.iter().cloned().collect();
        let n = values_vec.len() as f64;

        let mean: f64 = values_vec.iter().sum::<f64>() / n;
        let variance = values_vec.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        if mean == 0.0 {
            return 0.0;
        }

        let cv = std_dev / mean.abs();
        (1.0 - cv.min(1.0)).max(0.0)
    }

    pub fn clear_samples(&self, metric_type: MetricType) {
        let mut samples = self.samples.write();
        samples.remove(&metric_type);
    }

    pub fn clear_all_samples(&self) {
        let mut samples = self.samples.write();
        samples.clear();
    }

    pub fn get_sample_count(&self, metric_type: MetricType) -> usize {
        let samples = self.samples.read();
        samples.get(&metric_type).map(|v| v.len()).unwrap_or(0)
    }

    pub fn setup_default_configs(&self) {
        let default_configs = vec![
            BaselineConfig {
                metric_type: MetricType::CpuUsage,
                learning_window_secs: 3600,
                min_samples: 100,
                auto_update: true,
                update_interval_secs: 300,
                sensitivity: BaselineSensitivity::Medium,
                enabled: true,
            },
            BaselineConfig {
                metric_type: MetricType::MemoryPercent,
                learning_window_secs: 3600,
                min_samples: 100,
                auto_update: true,
                update_interval_secs: 300,
                sensitivity: BaselineSensitivity::Medium,
                enabled: true,
            },
            BaselineConfig {
                metric_type: MetricType::Tps,
                learning_window_secs: 7200,
                min_samples: 200,
                auto_update: true,
                update_interval_secs: 600,
                sensitivity: BaselineSensitivity::High,
                enabled: true,
            },
        ];

        for config in default_configs {
            self.configure(config);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_learner_creation() {
        let learner = BaselineLearner::with_default();
        assert_eq!(learner.max_samples, 10000);
    }

    #[test]
    fn test_add_samples() {
        let learner = BaselineLearner::with_default();
        learner.add_samples(MetricType::CpuUsage, &[10.0, 20.0, 30.0, 40.0, 50.0]);

        let count = learner.get_sample_count(MetricType::CpuUsage);
        assert_eq!(count, 5);
    }

    #[test]
    fn test_calculate_baseline() {
        let learner = BaselineLearner::with_default();

        let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        learner.add_samples(MetricType::CpuUsage, &values);

        let baseline = learner.calculate_baseline(MetricType::CpuUsage);
        assert!(baseline.is_some());

        let b = baseline.unwrap();
        assert!((b.mean - 50.5).abs() < 0.1);
        assert_eq!(b.min, 1.0);
        assert_eq!(b.max, 100.0);
    }

    #[test]
    fn test_deviation_calculation() {
        let learner = BaselineLearner::with_default();

        let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        learner.add_samples(MetricType::CpuUsage, &values);

        let deviation = learner.calculate_deviation(MetricType::CpuUsage, 50.5);
        assert!(deviation.is_some());
        assert!(deviation.unwrap() < 0.1);
    }

    #[test]
    fn test_anomaly_detection() {
        let learner = BaselineLearner::with_default();
        learner.setup_default_configs();

        let values: Vec<f64> = (10..=50).map(|i| i as f64).collect();
        learner.add_samples(MetricType::CpuUsage, &values);

        let normal_value = 30.0;
        let anomaly = learner.detect_and_record_anomaly(MetricType::CpuUsage, normal_value);
        assert!(anomaly.is_none());

        let abnormal_value = 90.0;
        let anomaly = learner.detect_and_record_anomaly(MetricType::CpuUsage, abnormal_value);
        assert!(anomaly.is_some());

        let detected = anomaly.unwrap();
        assert_eq!(detected.severity, AnomalySeverity::High);
    }

    #[test]
    fn test_profile_creation() {
        let learner = BaselineLearner::with_default();

        let profile = learner.create_profile("CPU Profile", MetricType::CpuUsage);
        assert_eq!(profile.name, "CPU Profile");
        assert_eq!(profile.metric_type, MetricType::CpuUsage);
    }

    #[test]
    fn test_anomaly_history() {
        let learner = BaselineLearner::with_default();
        learner.setup_default_configs();

        let values: Vec<f64> = (10..=50).map(|i| i as f64).collect();
        learner.add_samples(MetricType::CpuUsage, &values);

        learner.detect_and_record_anomaly(MetricType::CpuUsage, 95.0).unwrap();

        let history = learner.get_anomaly_history(10);
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_convergence_score() {
        let learner = BaselineLearner::with_default();

        let stable_values: Vec<f64> = vec![50.0; 100];
        learner.add_samples(MetricType::CpuUsage, &stable_values);

        let score = learner.get_learning_stats(MetricType::CpuUsage)
            .map(|s| s.convergence_score)
            .unwrap_or(0.0);

        assert!(score > 0.9);
    }
}

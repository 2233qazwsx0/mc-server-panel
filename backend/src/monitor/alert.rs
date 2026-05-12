use crate::monitor::types::{Alert, AlertLevel, AlertThreshold, MetricType, ThresholdOperator};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AlertError {
    #[error("Threshold not found: {0}")]
    ThresholdNotFound(String),
    #[error("Invalid threshold configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Alert not found: {0}")]
    AlertNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub id: String,
    pub name: String,
    pub metric_type: MetricType,
    pub operator: ThresholdOperator,
    pub threshold_value: f64,
    pub alert_level: AlertLevel,
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub duration_seconds: u64,
    pub notify_once: bool,
}

impl ThresholdConfig {
    pub fn new(
        name: String,
        metric_type: MetricType,
        operator: ThresholdOperator,
        threshold_value: f64,
        alert_level: AlertLevel,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            metric_type,
            operator,
            threshold_value,
            alert_level,
            enabled: true,
            cooldown_seconds: 300,
            duration_seconds: 0,
            notify_once: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub metric_type: MetricType,
    pub conditions: Vec<AlertCondition>,
    pub alert_level: AlertLevel,
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    pub metric_type: MetricType,
    pub operator: ThresholdOperator,
    pub threshold: f64,
    pub logical_operator: Option<LogicalOperator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Clone)]
pub struct AlertManager {
    thresholds: Arc<RwLock<HashMap<String, AlertThreshold>>>,
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    cooldown_tracker: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    alert_rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    max_history_size: usize,
}

impl AlertManager {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            thresholds: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            cooldown_tracker: Arc::new(RwLock::new(HashMap::new())),
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            max_history_size,
        }
    }

    pub fn with_default() -> Self {
        Self::new(1000)
    }

    pub fn add_threshold(&self, threshold: AlertThreshold) -> Result<(), AlertError> {
        let mut thresholds = self.thresholds.write();
        thresholds.insert(threshold.id.clone(), threshold);
        Ok(())
    }

    pub fn remove_threshold(&self, id: &str) -> Result<AlertThreshold, AlertError> {
        let mut thresholds = self.thresholds.write();
        thresholds.remove(id).ok_or_else(|| {
            AlertError::ThresholdNotFound(id.to_string())
        })
    }

    pub fn get_threshold(&self, id: &str) -> Option<AlertThreshold> {
        let thresholds = self.thresholds.read();
        thresholds.get(id).cloned()
    }

    pub fn get_all_thresholds(&self) -> Vec<AlertThreshold> {
        let thresholds = self.thresholds.read();
        thresholds.values().cloned().collect()
    }

    pub fn update_threshold(&self, threshold: AlertThreshold) -> Result<(), AlertError> {
        let mut thresholds = self.thresholds.write();
        if !thresholds.contains_key(&threshold.id) {
            return Err(AlertError::ThresholdNotFound(threshold.id.clone()));
        }
        thresholds.insert(threshold.id.clone(), threshold);
        Ok(())
    }

    pub fn enable_threshold(&self, id: &str) -> Result<(), AlertError> {
        let mut thresholds = self.thresholds.write();
        let threshold = thresholds.get_mut(id).ok_or_else(|| {
            AlertError::ThresholdNotFound(id.to_string())
        })?;
        threshold.enabled = true;
        Ok(())
    }

    pub fn disable_threshold(&self, id: &str) -> Result<(), AlertError> {
        let mut thresholds = self.thresholds.write();
        let threshold = thresholds.get_mut(id).ok_or_else(|| {
            AlertError::ThresholdNotFound(id.to_string())
        })?;
        threshold.enabled = false;
        Ok(())
    }

    fn is_in_cooldown(&self, threshold_id: &str) -> bool {
        let cooldowns = self.cooldown_tracker.read();
        if let Some(last_alert) = cooldowns.get(threshold_id) {
            let thresholds = self.thresholds.read();
            if let Some(threshold) = thresholds.get(threshold_id) {
                let elapsed = Utc::now() - *last_alert;
                return elapsed.num_seconds() < threshold.cooldown_seconds as i64;
            }
        }
        false
    }

    fn set_cooldown(&self, threshold_id: &str) {
        let mut cooldowns = self.cooldown_tracker.write();
        cooldowns.insert(threshold_id.to_string(), Utc::now());
    }

    fn evaluate_condition(value: f64, operator: ThresholdOperator, threshold: f64) -> bool {
        match operator {
            ThresholdOperator::GreaterThan => value > threshold,
            ThresholdOperator::LessThan => value < threshold,
            ThresholdOperator::GreaterThanOrEqual => value >= threshold,
            ThresholdOperator::LessThanOrEqual => value <= threshold,
            ThresholdOperator::Equal => (value - threshold).abs() < f64::EPSILON,
        }
    }

    pub fn check_threshold(
        &self,
        metric_type: MetricType,
        value: f64,
    ) -> Option<Alert> {
        let thresholds = self.thresholds.read();

        for threshold in thresholds.values() {
            if !threshold.enabled {
                continue;
            }

            if threshold.metric_type != metric_type {
                continue;
            }

            if !Self::evaluate_condition(value, threshold.operator, threshold.threshold_value) {
                continue;
            }

            drop(thresholds);

            if self.is_in_cooldown(&threshold.id) {
                return None;
            }

            let alert = Alert::new(
                Uuid::new_v4().to_string(),
                threshold.alert_level,
                threshold.metric_type.as_str().to_string(),
                format!(
                    "{} {} {} (current: {:.2}, threshold: {:.2})",
                    threshold.name,
                    match threshold.operator {
                        ThresholdOperator::GreaterThan => "exceeded",
                        ThresholdOperator::LessThan => "below",
                        ThresholdOperator::GreaterThanOrEqual => "reached or exceeded",
                        ThresholdOperator::LessThanOrEqual => "at or below",
                        ThresholdOperator::Equal => "equals",
                    },
                    threshold.threshold_value,
                    value,
                    threshold.threshold_value
                ),
                value,
                threshold.threshold_value,
            );

            self.set_cooldown(&threshold.id);
            return Some(alert);
        }

        None
    }

    pub fn check_all_thresholds(&self, metrics: &HashMap<MetricType, f64>) -> Vec<Alert> {
        let mut alerts = Vec::new();

        for (metric_type, value) in metrics {
            if let Some(alert) = self.check_threshold(*metric_type, *value) {
                alerts.push(alert);
            }
        }

        alerts
    }

    pub fn record_alert(&self, alert: Alert) {
        let mut history = self.alert_history.write();
        if history.len() >= self.max_history_size {
            history.pop_front();
        }
        history.push_back(alert);
    }

    pub fn acknowledge_alert(&self, alert_id: &str, acknowledged_by: &str) -> Result<Alert, AlertError> {
        let mut history = self.alert_history.write();

        for alert in history.iter_mut() {
            if alert.id == alert_id {
                alert.acknowledged = true;
                alert.acknowledged_by = Some(acknowledged_by.to_string());
                alert.acknowledged_at = Some(Utc::now());
                return Ok(alert.clone());
            }
        }

        Err(AlertError::AlertNotFound(alert_id.to_string()))
    }

    pub fn get_active_alerts(&self) -> Vec<Alert> {
        let history = self.alert_history.read();
        history.iter().filter(|a| !a.acknowledged).cloned().collect()
    }

    pub fn get_alert_history(&self, limit: usize) -> Vec<Alert> {
        let history = self.alert_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_alerts_by_level(&self, level: AlertLevel) -> Vec<Alert> {
        let history = self.alert_history.read();
        history.iter().filter(|a| a.level == level).cloned().collect()
    }

    pub fn get_alerts_by_metric(&self, metric_type: &str) -> Vec<Alert> {
        let history = self.alert_history.read();
        history
            .iter()
            .filter(|a| a.metric_type == metric_type)
            .cloned()
            .collect()
    }

    pub fn clear_alert_history(&self) {
        let mut history = self.alert_history.write();
        history.clear();
    }

    pub fn add_rule(&self, rule: AlertRule) {
        let mut rules = self.alert_rules.write();
        rules.insert(rule.id.clone(), rule);
    }

    pub fn remove_rule(&self, id: &str) -> Option<AlertRule> {
        let mut rules = self.alert_rules.write();
        rules.remove(id)
    }

    pub fn get_all_rules(&self) -> Vec<AlertRule> {
        let rules = self.alert_rules.read();
        rules.values().cloned().collect()
    }

    pub fn evaluate_rule(&self, rule: &AlertRule, metrics: &HashMap<MetricType, f64>) -> bool {
        if !rule.enabled {
            return false;
        }

        let mut results = Vec::new();

        for condition in &rule.conditions {
            if let Some(&value) = metrics.get(&condition.metric_type) {
                let result = Self::evaluate_condition(value, condition.operator, condition.threshold);
                results.push(result);
            } else {
                results.push(false);
            }
        }

        if results.is_empty() {
            return false;
        }

        let first_result = results[0];
        let mut final_result = first_result;

        for result in results.iter().skip(1) {
            if let Some(last_condition) = rule.conditions.get(0) {
                match last_condition.logical_operator {
                    Some(LogicalOperator::And) => final_result = final_result && *result,
                    Some(LogicalOperator::Or) => final_result = final_result || *result,
                    None => final_result = final_result && *result,
                }
            }
        }

        final_result
    }

    pub fn create_default_thresholds(&self) {
        let defaults = vec![
            AlertThreshold {
                id: "cpu_warning".to_string(),
                name: "CPU Usage Warning".to_string(),
                metric_type: MetricType::CpuUsage,
                operator: ThresholdOperator::GreaterThan,
                threshold_value: 80.0,
                alert_level: AlertLevel::Warning,
                enabled: true,
                cooldown_seconds: 300,
                created_at: Utc::now(),
            },
            AlertThreshold {
                id: "cpu_critical".to_string(),
                name: "CPU Usage Critical".to_string(),
                metric_type: MetricType::CpuUsage,
                operator: ThresholdOperator::GreaterThan,
                threshold_value: 95.0,
                alert_level: AlertLevel::Critical,
                enabled: true,
                cooldown_seconds: 60,
                created_at: Utc::now(),
            },
            AlertThreshold {
                id: "memory_warning".to_string(),
                name: "Memory Usage Warning".to_string(),
                metric_type: MetricType::MemoryPercent,
                operator: ThresholdOperator::GreaterThan,
                threshold_value: 80.0,
                alert_level: AlertLevel::Warning,
                enabled: true,
                cooldown_seconds: 300,
                created_at: Utc::now(),
            },
            AlertThreshold {
                id: "memory_critical".to_string(),
                name: "Memory Usage Critical".to_string(),
                metric_type: MetricType::MemoryPercent,
                operator: ThresholdOperator::GreaterThan,
                threshold_value: 95.0,
                alert_level: AlertLevel::Critical,
                enabled: true,
                cooldown_seconds: 60,
                created_at: Utc::now(),
            },
        ];

        for threshold in defaults {
            let _ = self.add_threshold(threshold);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_manager_creation() {
        let manager = AlertManager::with_default();
        assert_eq!(manager.max_history_size, 1000);
    }

    #[test]
    fn test_add_and_get_threshold() {
        let manager = AlertManager::with_default();

        let threshold = AlertThreshold {
            id: "test".to_string(),
            name: "Test Threshold".to_string(),
            metric_type: MetricType::CpuUsage,
            operator: ThresholdOperator::GreaterThan,
            threshold_value: 90.0,
            alert_level: AlertLevel::Warning,
            enabled: true,
            cooldown_seconds: 60,
            created_at: Utc::now(),
        };

        manager.add_threshold(threshold.clone()).unwrap();
        let retrieved = manager.get_threshold("test").unwrap();
        assert_eq!(retrieved.name, "Test Threshold");
    }

    #[test]
    fn test_evaluate_condition() {
        assert!(AlertManager::evaluate_condition(95.0, ThresholdOperator::GreaterThan, 90.0));
        assert!(AlertManager::evaluate_condition(50.0, ThresholdOperator::LessThan, 80.0));
        assert!(AlertManager::evaluate_condition(90.0, ThresholdOperator::GreaterThanOrEqual, 90.0));
        assert!(AlertManager::evaluate_condition(100.0, ThresholdOperator::LessThanOrEqual, 100.0));
        assert!(AlertManager::evaluate_condition(50.0, ThresholdOperator::Equal, 50.0));
    }

    #[test]
    fn test_check_threshold() {
        let manager = AlertManager::with_default();

        let threshold = AlertThreshold {
            id: "cpu".to_string(),
            name: "CPU Alert".to_string(),
            metric_type: MetricType::CpuUsage,
            operator: ThresholdOperator::GreaterThan,
            threshold_value: 90.0,
            alert_level: AlertLevel::Warning,
            enabled: true,
            cooldown_seconds: 60,
            created_at: Utc::now(),
        };

        manager.add_threshold(threshold).unwrap();

        let alert = manager.check_threshold(MetricType::CpuUsage, 95.0);
        assert!(alert.is_some());
        assert_eq!(alert.unwrap().level, AlertLevel::Warning);
    }

    #[test]
    fn test_no_alert_when_below_threshold() {
        let manager = AlertManager::with_default();

        let threshold = AlertThreshold {
            id: "cpu".to_string(),
            name: "CPU Alert".to_string(),
            metric_type: MetricType::CpuUsage,
            operator: ThresholdOperator::GreaterThan,
            threshold_value: 90.0,
            alert_level: AlertLevel::Warning,
            enabled: true,
            cooldown_seconds: 60,
            created_at: Utc::now(),
        };

        manager.add_threshold(threshold).unwrap();

        let alert = manager.check_threshold(MetricType::CpuUsage, 50.0);
        assert!(alert.is_none());
    }

    #[test]
    fn test_cooldown() {
        let manager = AlertManager::with_default();

        let threshold = AlertThreshold {
            id: "cpu".to_string(),
            name: "CPU Alert".to_string(),
            metric_type: MetricType::CpuUsage,
            operator: ThresholdOperator::GreaterThan,
            threshold_value: 90.0,
            alert_level: AlertLevel::Warning,
            enabled: true,
            cooldown_seconds: 3600,
            created_at: Utc::now(),
        };

        manager.add_threshold(threshold).unwrap();

        let alert1 = manager.check_threshold(MetricType::CpuUsage, 95.0);
        assert!(alert1.is_some());

        let alert2 = manager.check_threshold(MetricType::CpuUsage, 95.0);
        assert!(alert2.is_none());
    }

    #[test]
    fn test_acknowledge_alert() {
        let manager = AlertManager::with_default();

        let threshold = AlertThreshold {
            id: "cpu".to_string(),
            name: "CPU Alert".to_string(),
            metric_type: MetricType::CpuUsage,
            operator: ThresholdOperator::GreaterThan,
            threshold_value: 90.0,
            alert_level: AlertLevel::Warning,
            enabled: true,
            cooldown_seconds: 60,
            created_at: Utc::now(),
        };

        manager.add_threshold(threshold).unwrap();

        let alert = manager.check_threshold(MetricType::CpuUsage, 95.0).unwrap();
        manager.record_alert(alert);

        let acknowledged = manager.acknowledge_alert(&alert.id, "admin");
        assert!(acknowledged.is_ok());
        assert!(acknowledged.unwrap().acknowledged);
    }

    #[test]
    fn test_check_all_thresholds() {
        let manager = AlertManager::with_default();

        manager.add_threshold(AlertThreshold {
            id: "cpu".to_string(),
            name: "CPU Alert".to_string(),
            metric_type: MetricType::CpuUsage,
            operator: ThresholdOperator::GreaterThan,
            threshold_value: 90.0,
            alert_level: AlertLevel::Warning,
            enabled: true,
            cooldown_seconds: 60,
            created_at: Utc::now(),
        }).unwrap();

        manager.add_threshold(AlertThreshold {
            id: "memory".to_string(),
            name: "Memory Alert".to_string(),
            metric_type: MetricType::MemoryPercent,
            operator: ThresholdOperator::GreaterThan,
            threshold_value: 80.0,
            alert_level: AlertLevel::Critical,
            enabled: true,
            cooldown_seconds: 60,
            created_at: Utc::now(),
        }).unwrap();

        let mut metrics = HashMap::new();
        metrics.insert(MetricType::CpuUsage, 95.0);
        metrics.insert(MetricType::MemoryPercent, 85.0);

        let alerts = manager.check_all_thresholds(&metrics);
        assert_eq!(alerts.len(), 2);
    }

    #[test]
    fn test_create_default_thresholds() {
        let manager = AlertManager::with_default();
        manager.create_default_thresholds();

        let thresholds = manager.get_all_thresholds();
        assert!(thresholds.len() >= 4);
    }
}

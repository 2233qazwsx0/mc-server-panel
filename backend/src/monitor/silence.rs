use crate::monitor::types::{Alert, AlertLevel};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilenceRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub match_conditions: SilenceMatchConditions,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_recurring: bool,
    pub recurrence_pattern: Option<RecurrencePattern>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilenceMatchConditions {
    pub alert_levels: Option<Vec<AlertLevel>>,
    pub metric_types: Option<Vec<String>>,
    pub alert_ids: Option<Vec<String>>,
    pub patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrencePattern {
    pub frequency: RecurrenceFrequency,
    pub interval: u32,
    pub days_of_week: Option<Vec<u8>>,
    pub days_of_month: Option<Vec<u8>>,
    pub time_range: Option<TimeRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecurrenceFrequency {
    Minutely,
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_hour: u8,
    pub start_minute: u8,
    pub end_hour: u8,
    pub end_minute: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub levels: Vec<EscalationLevel>,
    pub repeat_count: u32,
    pub created_at: DateTime<Utc>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u32,
    pub notify_after_minutes: u32,
    pub recipients: Vec<EscalationRecipient>,
    pub escalation_actions: Vec<EscalationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRecipient {
    pub recipient_type: RecipientType,
    pub identifier: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipientType {
    User,
    Group,
    Webhook,
    Email,
    Sms,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationAction {
    pub action_type: EscalationActionType,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationActionType {
    Notify,
    CreateTicket,
    RunCommand,
    SendEmail,
    SendSms,
    ExecuteWebhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationEvent {
    pub id: String,
    pub alert_id: String,
    pub policy_id: String,
    pub current_level: u32,
    pub notified_recipients: Vec<EscalationRecipient>,
    pub timestamp: DateTime<Utc>,
    pub status: EscalationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EscalationStatus {
    Pending,
    Notified,
    Resolved,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertState {
    pub alert: Alert,
    pub silence_rule_id: Option<String>,
    pub escalation_policy_id: Option<String>,
    pub escalation_level: u32,
    pub last_escalation: Option<DateTime<Utc>>,
    pub silenced_until: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct SilenceManager {
    rules: Arc<RwLock<Vec<SilenceRule>>>,
    active_silences: Arc<RwLock<VecDeque<ActiveSilence>>>,
    max_history_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSilence {
    pub rule_id: String,
    pub alert_id: String,
    pub silenced_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EscalationManager {
    policies: Arc<RwLock<HashMap<String, EscalationPolicy>>>,
    active_escalations: Arc<RwLock<HashMap<String, EscalationEvent>>>,
    escalation_history: Arc<RwLock<VecDeque<EscalationEvent>>>,
    max_history_size: usize,
}

impl SilenceManager {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            active_silences: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            max_history_size,
        }
    }

    pub fn with_default() -> Self {
        Self::new(1000)
    }

    pub fn add_rule(&self, rule: SilenceRule) {
        let mut rules = self.rules.write();
        rules.push(rule);
    }

    pub fn remove_rule(&self, rule_id: &str) -> Option<SilenceRule> {
        let mut rules = self.rules.write();
        rules.retain(|r| r.id != rule_id);
        rules.pop()
    }

    pub fn get_rule(&self, rule_id: &str) -> Option<SilenceRule> {
        let rules = self.rules.read();
        rules.iter().find(|r| r.id == rule_id).cloned()
    }

    pub fn get_all_rules(&self) -> Vec<SilenceRule> {
        let rules = self.rules.read();
        rules.clone()
    }

    pub fn enable_rule(&self, rule_id: &str) -> Result<(), String> {
        let mut rules = self.rules.write();
        let rule = rules.iter_mut().find(|r| r.id == rule_id)
            .ok_or_else(|| "Rule not found".to_string())?;
        rule.enabled = true;
        Ok(())
    }

    pub fn disable_rule(&self, rule_id: &str) -> Result<(), String> {
        let mut rules = self.rules.write();
        let rule = rules.iter_mut().find(|r| r.id == rule_id)
            .ok_or_else(|| "Rule not found".to_string())?;
        rule.enabled = false;
        Ok(())
    }

    pub fn is_silenced(&self, alert: &Alert) -> bool {
        let rules = self.rules.read();
        let now = Utc::now();

        for rule in rules.iter().filter(|r| r.enabled) {
            if now < rule.start_time || now > rule.end_time {
                continue;
            }

            if Self::matches_conditions(alert, &rule.match_conditions) {
                return true;
            }
        }

        false
    }

    fn matches_conditions(alert: &Alert, conditions: &SilenceMatchConditions) -> bool {
        if let Some(ref levels) = conditions.alert_levels {
            if !levels.contains(&alert.level) {
                return false;
            }
        }

        if let Some(ref types) = conditions.metric_types {
            if !types.contains(&alert.metric_type) {
                return false;
            }
        }

        if let Some(ref ids) = conditions.alert_ids {
            if !ids.contains(&alert.id) {
                return false;
            }
        }

        true
    }

    pub fn silence_alert(&self, alert: &Alert, rule: &SilenceRule, duration_secs: u64) -> ActiveSilence {
        let silence = ActiveSilence {
            rule_id: rule.id.clone(),
            alert_id: alert.id.clone(),
            silenced_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(duration_secs as i64),
        };

        let mut active = self.active_silences.write();
        if active.len() >= self.max_history_size {
            active.pop_front();
        }
        active.push_back(silence.clone());

        silence
    }

    pub fn get_active_silences(&self) -> Vec<ActiveSilence> {
        let silences = self.active_silences.read();
        let now = Utc::now();

        silences
            .iter()
            .filter(|s| s.expires_at > now)
            .cloned()
            .collect()
    }

    pub fn create_maintenance_window(
        &self,
        name: &str,
        description: &str,
        duration_hours: u32,
        created_by: &str,
    ) -> SilenceRule {
        let now = Utc::now();
        let end = now + Duration::hours(duration_hours as i64);

        let rule = SilenceRule {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            match_conditions: SilenceMatchConditions {
                alert_levels: None,
                metric_types: None,
                alert_ids: None,
                patterns: None,
            },
            start_time: now,
            end_time: end,
            is_recurring: false,
            recurrence_pattern: None,
            created_at: now,
            created_by: created_by.to_string(),
            enabled: true,
        };

        self.add_rule(rule.clone());
        rule
    }

    pub fn cleanup_expired(&self) -> usize {
        let mut silences = self.active_silences.write();
        let now = Utc::now();
        let initial_len = silences.len();

        silences.retain(|s| s.expires_at > now);

        initial_len - silences.len()
    }
}

impl EscalationManager {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            active_escalations: Arc::new(RwLock::new(HashMap::new())),
            escalation_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            max_history_size,
        }
    }

    pub fn with_default() -> Self {
        Self::new(1000)
    }

    pub fn add_policy(&self, policy: EscalationPolicy) {
        let mut policies = self.policies.write();
        policies.insert(policy.id.clone(), policy);
    }

    pub fn remove_policy(&self, policy_id: &str) -> Option<EscalationPolicy> {
        let mut policies = self.policies.write();
        policies.remove(policy_id)
    }

    pub fn get_policy(&self, policy_id: &str) -> Option<EscalationPolicy> {
        let policies = self.policies.read();
        policies.get(policy_id).cloned()
    }

    pub fn get_all_policies(&self) -> Vec<EscalationPolicy> {
        let policies = self.policies.read();
        policies.values().cloned().collect()
    }

    pub fn enable_policy(&self, policy_id: &str) -> Result<(), String> {
        let mut policies = self.policies.write();
        let policy = policies.get_mut(policy_id)
            .ok_or_else(|| "Policy not found".to_string())?;
        policy.enabled = true;
        Ok(())
    }

    pub fn disable_policy(&self, policy_id: &str) -> Result<(), String> {
        let mut policies = self.policies.write();
        let policy = policies.get_mut(policy_id)
            .ok_or_else(|| "Policy not found".to_string())?;
        policy.enabled = false;
        Ok(())
    }

    pub fn start_escalation(&self, alert: &Alert, policy_id: &str) -> Result<EscalationEvent, String> {
        let policies = self.policies.read();
        let policy = policies.get(policy_id)
            .ok_or_else(|| "Policy not found".to_string())?;

        if !policy.enabled {
            return Err("Policy is disabled".to_string());
        }

        let event = EscalationEvent {
            id: Uuid::new_v4().to_string(),
            alert_id: alert.id.clone(),
            policy_id: policy_id.to_string(),
            current_level: 0,
            notified_recipients: Vec::new(),
            timestamp: Utc::now(),
            status: EscalationStatus::Pending,
        };

        let mut active = self.active_escalations.write();
        active.insert(alert.id.clone(), event.clone());

        drop(active);

        let mut history = self.escalation_history.write();
        if history.len() >= self.max_history_size {
            history.pop_front();
        }
        history.push_back(event.clone());

        Ok(event)
    }

    pub fn escalate(&self, alert_id: &str) -> Result<EscalationEvent, String> {
        let mut active = self.active_escalations.write();
        let event = active.get_mut(alert_id)
            .ok_or_else(|| "Escalation not found".to_string())?;

        let policies = self.policies.read();
        let policy = policies.get(&event.policy_id)
            .ok_or_else(|| "Policy not found".to_string())?;

        event.current_level += 1;

        if event.current_level >= policy.levels.len() as u32 {
            event.status = EscalationStatus::Resolved;
        } else {
            let level = &policy.levels[event.current_level as usize];
            event.notified_recipients = level.recipients.clone();
            event.status = EscalationStatus::Notified;
            event.timestamp = Utc::now();
        }

        let event_clone = event.clone();
        drop(active);

        let mut history = self.escalation_history.write();
        history.push_back(event_clone.clone());

        Ok(event_clone)
    }

    pub fn resolve_escalation(&self, alert_id: &str) -> Result<(), String> {
        let mut active = self.active_escalations.write();
        let event = active.get_mut(alert_id)
            .ok_or_else(|| "Escalation not found".to_string())?;
        event.status = EscalationStatus::Resolved;
        event.timestamp = Utc::now();
        Ok(())
    }

    pub fn cancel_escalation(&self, alert_id: &str) -> Result<(), String> {
        let mut active = self.active_escalations.write();
        let event = active.get_mut(alert_id)
            .ok_or_else(|| "Escalation not found".to_string())?;
        event.status = EscalationStatus::Cancelled;
        event.timestamp = Utc::now();
        Ok(())
    }

    pub fn get_active_escalation(&self, alert_id: &str) -> Option<EscalationEvent> {
        let active = self.active_escalations.read();
        active.get(alert_id).cloned()
    }

    pub fn get_escalation_history(&self, limit: usize) -> Vec<EscalationEvent> {
        let history = self.escalation_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn check_escalation_needed(&self, alert: &Alert) -> bool {
        let now = Utc::now();
        let threshold = now - Duration::minutes(15);

        let active = self.active_escalations.read();
        if let Some(event) = active.get(&alert.id) {
            if event.status == EscalationStatus::Pending {
                return event.timestamp < threshold;
            }
        }

        false
    }

    pub fn create_default_policies(&self) {
        let critical_policy = EscalationPolicy {
            id: "critical_escalation".to_string(),
            name: "Critical Alert Escalation".to_string(),
            description: "Escalation policy for critical alerts".to_string(),
            levels: vec![
                EscalationLevel {
                    level: 0,
                    notify_after_minutes: 5,
                    recipients: vec![
                        EscalationRecipient {
                            recipient_type: RecipientType::Webhook,
                            identifier: "webhook_1".to_string(),
                            name: "Primary On-Call".to_string(),
                        },
                    ],
                    escalation_actions: vec![
                        EscalationAction {
                            action_type: EscalationActionType::Notify,
                            config: HashMap::new(),
                        },
                    ],
                },
                EscalationLevel {
                    level: 1,
                    notify_after_minutes: 15,
                    recipients: vec![
                        EscalationRecipient {
                            recipient_type: RecipientType::Group,
                            identifier: "ops_team".to_string(),
                            name: "Operations Team".to_string(),
                        },
                    ],
                    escalation_actions: vec![
                        EscalationAction {
                            action_type: EscalationActionType::CreateTicket,
                            config: HashMap::new(),
                        },
                    ],
                },
            ],
            repeat_count: 2,
            created_at: Utc::now(),
            enabled: true,
        };

        self.add_policy(critical_policy);

        let warning_policy = EscalationPolicy {
            id: "warning_escalation".to_string(),
            name: "Warning Alert Escalation".to_string(),
            description: "Escalation policy for warning alerts".to_string(),
            levels: vec![
                EscalationLevel {
                    level: 0,
                    notify_after_minutes: 30,
                    recipients: vec![
                        EscalationRecipient {
                            recipient_type: RecipientType::Webhook,
                            identifier: "webhook_2".to_string(),
                            name: "Secondary On-Call".to_string(),
                        },
                    ],
                    escalation_actions: vec![
                        EscalationAction {
                            action_type: EscalationActionType::Notify,
                            config: HashMap::new(),
                        },
                    ],
                },
            ],
            repeat_count: 1,
            created_at: Utc::now(),
            enabled: true,
        };

        self.add_policy(warning_policy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_manager_creation() {
        let manager = SilenceManager::with_default();
        assert_eq!(manager.max_history_size, 1000);
    }

    #[test]
    fn test_add_and_get_rule() {
        let manager = SilenceManager::with_default();
        let rule = manager.create_maintenance_window("Test", "Test maintenance", 1, "admin");

        let retrieved = manager.get_rule(&rule.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test");
    }

    #[test]
    fn test_is_silenced() {
        let manager = SilenceManager::with_default();

        let rule = SilenceRule {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            match_conditions: SilenceMatchConditions {
                alert_levels: Some(vec![AlertLevel::Critical]),
                metric_types: None,
                alert_ids: None,
                patterns: None,
            },
            start_time: Utc::now() - Duration::hours(1),
            end_time: Utc::now() + Duration::hours(1),
            is_recurring: false,
            recurrence_pattern: None,
            created_at: Utc::now(),
            created_by: "admin".to_string(),
            enabled: true,
        };

        manager.add_rule(rule);

        let critical_alert = Alert::new(
            "alert1".to_string(),
            AlertLevel::Critical,
            "cpu_usage".to_string(),
            "Critical".to_string(),
            99.0,
            95.0,
        );

        let warning_alert = Alert::new(
            "alert2".to_string(),
            AlertLevel::Warning,
            "cpu_usage".to_string(),
            "Warning".to_string(),
            85.0,
            80.0,
        );

        assert!(manager.is_silenced(&critical_alert));
        assert!(!manager.is_silenced(&warning_alert));
    }

    #[test]
    fn test_escalation_manager_creation() {
        let manager = EscalationManager::with_default();
        assert_eq!(manager.max_history_size, 1000);
    }

    #[test]
    fn test_add_and_get_policy() {
        let manager = EscalationManager::with_default();
        manager.create_default_policies();

        let policies = manager.get_all_policies();
        assert!(!policies.is_empty());
    }

    #[test]
    fn test_start_escalation() {
        let manager = EscalationManager::with_default();
        manager.create_default_policies();

        let alert = Alert::new(
            "alert1".to_string(),
            AlertLevel::Critical,
            "cpu_usage".to_string(),
            "Critical CPU".to_string(),
            99.0,
            95.0,
        );

        let result = manager.start_escalation(&alert, "critical_escalation");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().current_level, 0);
    }

    #[test]
    fn test_escalate() {
        let manager = EscalationManager::with_default();
        manager.create_default_policies();

        let alert = Alert::new(
            "alert1".to_string(),
            AlertLevel::Critical,
            "cpu_usage".to_string(),
            "Critical CPU".to_string(),
            99.0,
            95.0,
        );

        manager.start_escalation(&alert, "critical_escalation").unwrap();
        let escalated = manager.escalate(&alert.id);
        assert!(escalated.is_ok());
        assert_eq!(escalated.unwrap().current_level, 1);
    }
}

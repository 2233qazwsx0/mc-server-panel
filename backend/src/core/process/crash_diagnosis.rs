use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::error::ProcessError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub instance_id: String,
    pub exit_code: i32,
    pub crash_type: CrashType,
    pub summary: String,
    pub details: CrashDetails,
    pub recommendations: Vec<String>,
    pub raw_output: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrashType {
    OutOfMemory,
    StackOverflow,
    SigTerm,
    SigKill,
    SegmentationFault,
    JvmError,
    PluginError,
    WorldCorruption,
    DiskFull,
    NetworkError,
    Unknown,
}

impl Default for CrashType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashDetails {
    pub java_version: Option<String>,
    pub memory_total_mb: Option<u64>,
    pub memory_used_mb: Option<u64>,
    pub error_message: Option<String>,
    pub stack_trace: Option<String>,
    pub system_info: HashMap<String, String>,
}

impl Default for CrashDetails {
    fn default() -> Self {
        Self {
            java_version: None,
            memory_total_mb: None,
            memory_used_mb: None,
            error_message: None,
            stack_trace: None,
            system_info: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub crash_type: CrashType,
    pub confidence: f32,
    pub summary: String,
    pub details: CrashDetails,
    pub recommendations: Vec<String>,
    pub recovery_suggestion: RecoverySuggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySuggestion {
    pub action: RecoveryAction,
    pub steps: Vec<String>,
    pub estimated_recovery_time_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecoveryAction {
    Restart,
    IncreaseMemory,
    RestoreBackup,
    UpdateServer,
    CheckPlugins,
    RepairWorld,
    CheckDisk,
    NoAction,
}

impl Default for RecoveryAction {
    fn default() -> Self {
        Self::Restart
    }
}

pub struct CrashDiagnosis {
    crash_reports: RwLock<Vec<CrashReport>>,
    crash_log_path: PathBuf,
    patterns: HashMap<CrashType, Vec<CrashPattern>>,
}

#[derive(Debug, Clone)]
struct CrashPattern {
    pub pattern: Regex,
    pub crash_type: CrashType,
    pub confidence: f32,
}

impl CrashDiagnosis {
    pub fn new(crash_log_path: PathBuf) -> Self {
        Self {
            crash_reports: RwLock::new(Vec::new()),
            crash_log_path,
            patterns: Self::initialize_patterns(),
        }
    }

    fn initialize_patterns() -> HashMap<CrashType, Vec<CrashPattern>> {
        let mut patterns = HashMap::new();

        patterns.insert(
            CrashType::OutOfMemory,
            vec![
                CrashPattern {
                    pattern: Regex::new(r"OutOfMemoryError").unwrap(),
                    crash_type: CrashType::OutOfMemory,
                    confidence: 1.0,
                },
                CrashPattern {
                    pattern: Regex::new(r"java\.lang\.OutOfMemoryError").unwrap(),
                    crash_type: CrashType::OutOfMemory,
                    confidence: 1.0,
                },
                CrashPattern {
                    pattern: Regex::new(r"Heap space").unwrap(),
                    crash_type: CrashType::OutOfMemory,
                    confidence: 0.9,
                },
            ],
        );

        patterns.insert(
            CrashType::StackOverflow,
            vec![
                CrashPattern {
                    pattern: Regex::new(r"StackOverflowError").unwrap(),
                    crash_type: CrashType::StackOverflow,
                    confidence: 1.0,
                },
            ],
        );

        patterns.insert(
            CrashType::SigTerm,
            vec![
                CrashPattern {
                    pattern: Regex::new(r"SIGTERM").unwrap(),
                    crash_type: CrashType::SigTerm,
                    confidence: 1.0,
                },
                CrashPattern {
                    pattern: Regex::new(r"Received interrupt signal").unwrap(),
                    crash_type: CrashType::SigTerm,
                    confidence: 0.8,
                },
            ],
        );

        patterns.insert(
            CrashType::SegmentationFault,
            vec![
                CrashPattern {
                    pattern: Regex::new(r"SIGSEGV").unwrap(),
                    crash_type: CrashType::SegmentationFault,
                    confidence: 1.0,
                },
                CrashPattern {
                    pattern: Regex::new(r"Segmentation fault").unwrap(),
                    crash_type: CrashType::SegmentationFault,
                    confidence: 1.0,
                },
            ],
        );

        patterns.insert(
            CrashType::DiskFull,
            vec![
                CrashPattern {
                    pattern: Regex::new(r"No space left on device").unwrap(),
                    crash_type: CrashType::DiskFull,
                    confidence: 1.0,
                },
                CrashPattern {
                    pattern: Regex::new(r"Disk write operations failed").unwrap(),
                    crash_type: CrashType::DiskFull,
                    confidence: 0.9,
                },
            ],
        );

        patterns.insert(
            CrashType::PluginError,
            vec![
                CrashPattern {
                    pattern: Regex::new(r"Exception in thread.*Plugin").unwrap(),
                    crash_type: CrashType::PluginError,
                    confidence: 0.9,
                },
                CrashPattern {
                    pattern: Regex::new(r"Caused by:.*Plugin").unwrap(),
                    crash_type: CrashType::PluginError,
                    confidence: 0.8,
                },
            ],
        );

        patterns
    }

    pub async fn diagnose(
        &self,
        instance_id: String,
        exit_code: i32,
        output: &str,
    ) -> Result<DiagnosticResult, ProcessError> {
        let crash_type = self.detect_crash_type(output);
        let details = self.extract_details(output, exit_code);
        let recommendations = self.generate_recommendations(crash_type, &details);
        let recovery = self.suggest_recovery(crash_type, &details);

        let confidence = self.calculate_confidence(crash_type, output);

        let summary = match crash_type {
            CrashType::OutOfMemory => format!(
                "Server crashed due to OutOfMemoryError. Current memory usage: {}MB",
                details.memory_used_mb.unwrap_or(0)
            ),
            CrashType::StackOverflow => "Server crashed due to stack overflow".to_string(),
            CrashType::SigTerm => "Server was terminated by signal".to_string(),
            CrashType::SigKill => "Server was killed by system (likely OOM killer)".to_string(),
            CrashType::SegmentationFault => "Server crashed due to memory access violation".to_string(),
            CrashType::JvmError => format!(
                "JVM internal error: {}",
                details.error_message.as_deref().unwrap_or("Unknown")
            ),
            CrashType::PluginError => "Server crashed due to plugin error".to_string(),
            CrashType::WorldCorruption => "Server crashed due to world corruption".to_string(),
            CrashType::DiskFull => "Server crashed due to insufficient disk space".to_string(),
            CrashType::NetworkError => "Server crashed due to network error".to_string(),
            CrashType::Unknown => format!("Unknown crash type (exit code: {})", exit_code),
        };

        Ok(DiagnosticResult {
            crash_type,
            confidence,
            summary,
            details,
            recommendations,
            recovery_suggestion: recovery,
        })
    }

    fn detect_crash_type(&self, output: &str) -> CrashType {
        let mut highest_confidence = 0.0;
        let mut detected_type = CrashType::Unknown;

        for (crash_type, crash_patterns) in &self.patterns {
            for pattern in crash_patterns {
                if pattern.pattern.is_match(output) {
                    if pattern.confidence > highest_confidence {
                        highest_confidence = pattern.confidence;
                        detected_type = *crash_type;
                    }
                }
            }
        }

        if highest_confidence == 0.0 && output.contains("Killed") {
            detected_type = CrashType::SigKill;
        }

        detected_type
    }

    fn calculate_confidence(&self, crash_type: CrashType, output: &str) -> f32 {
        if let Some(patterns) = self.patterns.get(&crash_type) {
            let max_confidence = patterns
                .iter()
                .filter(|p| p.pattern.is_match(output))
                .map(|p| p.confidence)
                .fold(0.0, f32::max);

            max_confidence
        } else {
            0.5
        }
    }

    fn extract_details(&self, output: &str, exit_code: i32) -> CrashDetails {
        let mut details = CrashDetails::default();

        let java_version_re = Regex::new(r#"java version "([^"]+)""#).ok();
        if let Some(re) = java_version_re {
            if let Some(cap) = re.captures(output) {
                details.java_version = Some(cap[1].to_string());
            }
        }

        let memory_re = Regex::new(r"memory:\s*(\d+)M").ok();
        if let Some(re) = memory_re {
            if let Some(cap) = re.captures(output) {
                if let Ok(mb) = cap[1].parse() {
                    details.memory_used_mb = Some(mb);
                }
            }
        }

        if let Some(oom_match) = output.find("OutOfMemoryError") {
            let start = oom_match.saturating_sub(100);
            let end = (oom_match + 500).min(output.len());
            details.error_message = Some(output[start..end].to_string());
        }

        let stack_trace_re = Regex::new(r"Exception in thread[^\n]+\n([^\n]+\n){1,10}").ok();
        if let Some(re) = stack_trace_re {
            if let Some(mat) = re.find(output) {
                details.stack_trace = Some(mat.as_str().to_string());
            }
        }

        details.system_info.insert(
            "exit_code".to_string(),
            exit_code.to_string(),
        );

        if output.contains("Linux") {
            details.system_info.insert("os".to_string(), "Linux".to_string());
        } else if output.contains("Windows") {
            details.system_info.insert("os".to_string(), "Windows".to_string());
        } else if output.contains("Mac") {
            details.system_info.insert("os".to_string(), "macOS".to_string());
        }

        details
    }

    fn generate_recommendations(&self, crash_type: CrashType, details: &CrashDetails) -> Vec<String> {
        let mut recommendations = Vec::new();

        match crash_type {
            CrashType::OutOfMemory => {
                recommendations.push("Increase JVM maximum memory (-Xmx)".to_string());
                recommendations.push("Optimize server plugins to reduce memory usage".to_string());
                recommendations.push("Consider using a more efficient garbage collector (G1GC, ZGC)".to_string());
                if let Some(used) = details.memory_used_mb {
                    let recommended = (used as f64 * 1.5) as u64;
                    recommendations.push(format!("Recommended: -Xmx{}M", recommended));
                }
            }
            CrashType::StackOverflow => {
                recommendations.push("Increase JVM stack size (-Xss)".to_string());
                recommendations.push("Check for infinite recursion in plugins".to_string());
                recommendations.push("Update server software to latest version".to_string());
            }
            CrashType::SigTerm | CrashType::SigKill => {
                recommendations.push("Check system logs for OOM killer activity".to_string());
                recommendations.push("Ensure sufficient system memory".to_string());
                recommendations.push("Review system resource limits".to_string());
            }
            CrashType::SegmentationFault => {
                recommendations.push("Update native libraries and drivers".to_string());
                recommendations.push("Check for incompatible mods/plugins".to_string());
                recommendations.push("Consider using a different JVM implementation".to_string());
            }
            CrashType::JvmError => {
                recommendations.push("Update Java to the latest LTS version".to_string());
                recommendations.push("Check for known JVM bugs with current version".to_string());
                recommendations.push("Try using an alternative JVM".to_string());
            }
            CrashType::PluginError => {
                recommendations.push("Update problematic plugin to latest version".to_string());
                recommendations.push("Check plugin compatibility with server version".to_string());
                recommendations.push("Temporarily disable plugins to identify culprit".to_string());
            }
            CrashType::WorldCorruption => {
                recommendations.push("Restore from latest backup".to_string());
                recommendations.push("Run world repair tools".to_string());
                recommendations.push("Check disk health".to_string());
            }
            CrashType::DiskFull => {
                recommendations.push("Free up disk space".to_string());
                recommendations.push("Clean up old log files".to_string());
                recommendations.push("Consider expanding disk storage".to_string());
            }
            CrashType::NetworkError => {
                recommendations.push("Check network connectivity".to_string());
                recommendations.push("Verify firewall configuration".to_string());
                recommendations.push("Check for DDoS protection triggers".to_string());
            }
            CrashType::Unknown => {
                recommendations.push("Review full crash log for details".to_string());
                recommendations.push("Update server and plugins to latest versions".to_string());
                recommendations.push("Check system requirements".to_string());
            }
        }

        recommendations
    }

    fn suggest_recovery(&self, crash_type: CrashType, details: &CrashDetails) -> RecoverySuggestion {
        match crash_type {
            CrashType::OutOfMemory => RecoverySuggestion {
                action: RecoveryAction::IncreaseMemory,
                steps: vec![
                    "Stop the server".to_string(),
                    "Edit startup script to increase -Xmx value".to_string(),
                    "Restart the server".to_string(),
                ],
                estimated_recovery_time_secs: 60,
            },
            CrashType::WorldCorruption => RecoverySuggestion {
                action: RecoveryAction::RestoreBackup,
                steps: vec![
                    "Locate latest valid backup".to_string(),
                    "Stop the server".to_string(),
                    "Restore world folder from backup".to_string(),
                    "Restart the server".to_string(),
                ],
                estimated_recovery_time_secs: 300,
            },
            CrashType::PluginError => RecoverySuggestion {
                action: RecoveryAction::CheckPlugins,
                steps: vec![
                    "Review crash log for plugin name".to_string(),
                    "Update or disable the problematic plugin".to_string(),
                    "Restart the server".to_string(),
                ],
                estimated_recovery_time_secs: 120,
            },
            CrashType::DiskFull => RecoverySuggestion {
                action: RecoveryAction::CheckDisk,
                steps: vec![
                    "Identify large files and directories".to_string(),
                    "Remove unnecessary files".to_string(),
                    "Restart the server".to_string(),
                ],
                estimated_recovery_time_secs: 180,
            },
            _ => RecoverySuggestion {
                action: RecoveryAction::Restart,
                steps: vec![
                    "Review crash diagnostics".to_string(),
                    "Address root cause".to_string(),
                    "Restart the server".to_string(),
                ],
                estimated_recovery_time_secs: 60,
            },
        }
    }

    pub async fn save_crash_report(&self, report: CrashReport) {
        let mut reports = self.crash_reports.write().await;
        reports.push(report);
        info!("Crash report saved");
    }

    pub async fn get_crash_reports(&self) -> Vec<CrashReport> {
        let reports = self.crash_reports.read().await;
        reports.clone()
    }

    pub async fn get_latest_crash_report(&self) -> Option<CrashReport> {
        let reports = self.crash_reports.read().await;
        reports.last().cloned()
    }

    pub async fn clear_old_reports(&self, max_reports: usize) {
        let mut reports = self.crash_reports.write().await;
        while reports.len() > max_reports {
            reports.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crash_diagnosis_creation() {
        let diagnosis = CrashDiagnosis::new(PathBuf::from("/tmp/crashes"));
        assert!(diagnosis.get_crash_reports().await.is_empty());
    }

    #[tokio::test]
    async fn test_detect_out_of_memory() {
        let diagnosis = CrashDiagnosis::new(PathBuf::from("/tmp/crashes"));
        let output = "java.lang.OutOfMemoryError: Heap space";
        let crash_type = diagnosis.detect_crash_type(output);
        assert_eq!(crash_type, CrashType::OutOfMemory);
    }

    #[tokio::test]
    async fn test_detect_sigterm() {
        let diagnosis = CrashDiagnosis::new(PathBuf::from("/tmp/crashes"));
        let output = "SIGTERM received";
        let crash_type = diagnosis.detect_crash_type(output);
        assert_eq!(crash_type, CrashType::SigTerm);
    }

    #[tokio::test]
    async fn test_detect_disk_full() {
        let diagnosis = CrashDiagnosis::new(PathBuf::from("/tmp/crashes"));
        let output = "No space left on device";
        let crash_type = diagnosis.detect_crash_type(output);
        assert_eq!(crash_type, CrashType::DiskFull);
    }

    #[tokio::test]
    async fn test_diagnose_oom() {
        let diagnosis = CrashDiagnosis::new(PathBuf::from("/tmp/crashes"));
        let output = r#"java.lang.OutOfMemoryError: Java heap space
	at java.base/java.util.ArrayList.(ArrayList.java:83)
Server thread dump:"#;

        let result = diagnosis.diagnose("test-instance".to_string(), 1, output).await.unwrap();

        assert_eq!(result.crash_type, CrashType::OutOfMemory);
        assert!(result.confidence > 0.9);
        assert!(!result.recommendations.is_empty());
        assert_eq!(result.recovery_suggestion.action, RecoveryAction::IncreaseMemory);
    }

    #[tokio::test]
    async fn test_diagnose_unknown() {
        let diagnosis = CrashDiagnosis::new(PathBuf::from("/tmp/crashes"));
        let output = "Server stopped for unknown reason";
        let result = diagnosis.diagnose("test-instance".to_string(), 0, output).await.unwrap();
        assert_eq!(result.crash_type, CrashType::Unknown);
    }

    #[tokio::test]
    async fn test_save_and_retrieve_crash_report() {
        let diagnosis = CrashDiagnosis::new(PathBuf::from("/tmp/crashes"));

        let report = CrashReport {
            id: "test-1".to_string(),
            timestamp: Utc::now(),
            instance_id: "test-instance".to_string(),
            exit_code: 1,
            crash_type: CrashType::OutOfMemory,
            summary: "Test crash".to_string(),
            details: CrashDetails::default(),
            recommendations: vec![],
            raw_output: None,
        };

        diagnosis.save_crash_report(report.clone()).await;
        let reports = diagnosis.get_crash_reports().await;
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].id, "test-1");
    }

    #[tokio::test]
    async fn test_clear_old_reports() {
        let diagnosis = CrashDiagnosis::new(PathBuf::from("/tmp/crashes"));

        for i in 0..10 {
            let report = CrashReport {
                id: format!("test-{}", i),
                timestamp: Utc::now(),
                instance_id: "test-instance".to_string(),
                exit_code: 1,
                crash_type: CrashType::Unknown,
                summary: format!("Test crash {}", i),
                details: CrashDetails::default(),
                recommendations: vec![],
                raw_output: None,
            };
            diagnosis.save_crash_report(report).await;
        }

        diagnosis.clear_old_reports(5).await;

        let reports = diagnosis.get_crash_reports().await;
        assert_eq!(reports.len(), 5);
    }

    #[test]
    fn test_crash_type_serialization() {
        let crash_type = CrashType::OutOfMemory;
        let json = serde_json::to_string(&crash_type).unwrap();
        assert_eq!(json, "\"outofmemory\"");

        let deserialized: CrashType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, CrashType::OutOfMemory);
    }

    #[test]
    fn test_recovery_action_serialization() {
        let action = RecoveryAction::RestoreBackup;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"restorebackup\"");

        let deserialized: RecoveryAction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, RecoveryAction::RestoreBackup);
    }
}

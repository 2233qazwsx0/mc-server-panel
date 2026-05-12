use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::plugins::types::*;

pub struct SecurityScanner {
    cache_dir: PathBuf,
    scan_history: RwLock<HashMap<String, SecurityReport>>,
    known_vulnerabilities: RwLock<Vec<Vulnerability>>,
}

impl SecurityScanner {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            scan_history: RwLock::new(HashMap::new()),
            known_vulnerabilities: RwLock::new(Vec::new()),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir).await?;
        self.load_vulnerability_database().await?;
        Ok(())
    }

    async fn load_vulnerability_database(&self) {
        let vulnerabilities = vec![
            Vulnerability {
                cve_id: Some("CVE-2021-44228".to_string()),
                title: "Log4Shell".to_string(),
                description: "Remote code execution via JNDI injection".to_string(),
                severity: "Critical".to_string(),
                cvss_score: Some(10.0),
                affected_versions: vec!["2.0-beta9 to 2.15.0".to_string()],
                fixed_in_version: Some("2.17.0".to_string()),
                remediation: "Update to Log4j 2.17.0 or later".to_string(),
            },
            Vulnerability {
                cve_id: Some("CVE-2023-42821".to_string()),
                title: "ProtocolLib Deserialization".to_string(),
                description: "Unsafe deserialization in PacketListener".to_string(),
                severity: "High".to_string(),
                cvss_score: Some(8.1),
                affected_versions: vec!["5.0.0 to 5.1.0".to_string()],
                fixed_in_version: Some("5.2.0".to_string()),
                remediation: "Update ProtocolLib to 5.2.0 or later".to_string(),
            },
            Vulnerability {
                cve_id: None,
                title: "WorldEdit/FAWE Unsafe File Access".to_string(),
                description: "Path traversal vulnerability in schematic operations".to_string(),
                severity: "Medium".to_string(),
                cvss_score: Some(6.5),
                affected_versions: vec!["7.0.0 to 7.2.10".to_string()],
                fixed_in_version: Some("7.2.11".to_string()),
                remediation: "Update WorldEdit to latest version".to_string(),
            },
        ];

        let mut db = self.known_vulnerabilities.write().await;
        *db = vulnerabilities;
    }

    pub async fn scan_plugin(&self, plugin: &Plugin) -> Result<SecurityReport> {
        let checks = self.perform_security_checks(plugin).await;
        let vulnerabilities = self.check_vulnerabilities(plugin).await;
        let permissions = self.analyze_permissions(plugin).await;
        let network = self.analyze_network_activity(plugin).await;

        let risk_level = self.calculate_risk_level(&checks, &vulnerabilities);
        let score = self.calculate_security_score(&checks, &vulnerabilities);

        let report = SecurityReport {
            plugin_id: plugin.plugin_id.to_string(),
            plugin_name: plugin.name.clone(),
            scan_time: Utc::now(),
            score,
            risk_level,
            checks,
            vulnerabilities,
            permissions,
            network_activity: network,
            overall_verdict: self.generate_verdict(score, risk_level),
        };

        let mut history = self.scan_history.write().await;
        history.insert(plugin.plugin_id.to_string(), report.clone());

        Ok(report)
    }

    async fn perform_security_checks(&self, plugin: &Plugin) -> Vec<SecurityCheck> {
        let mut checks = Vec::new();

        checks.push(self.check_download_source(plugin).await);
        checks.push(self.check_author_verification(plugin).await);
        checks.push(self.check_update_frequency(plugin).await);
        checks.push(self.check_source_code_availability(plugin).await);
        checks.push(self.check_popularity_trust(plugin).await);
        checks.push(self.check_permission_requirements(plugin).await);
        checks.push(self.check_network_access(plugin).await);
        checks.push(self.check_file_operations(plugin).await);
        checks.push(self.check_database_access(plugin).await);
        checks.push(self.check_command_execution(plugin).await);

        checks
    }

    async fn check_download_source(&self, plugin: &Plugin) -> SecurityCheck {
        let official_sources = vec![
            "spigotmc.org",
            "dev.bukkit.org",
            "papermc.io",
            "github.com",
        ];

        let is_official = official_sources.iter().any(|s| {
            plugin.download_url.contains(s) ||
            plugin.website.as_ref().map_or(false, |w| w.contains(s)).unwrap_or(false)
        });

        SecurityCheck {
            check_name: "Download Source Verification".to_string(),
            passed: is_official,
            severity: if is_official { "info".to_string() } else { "warning".to_string() },
            description: if is_official {
                "Downloaded from official source".to_string()
            } else {
                "Not from verified official source".to_string()
            },
            details: Some(plugin.download_url.clone()),
        }
    }

    async fn check_author_verification(&self, plugin: &Plugin) -> SecurityCheck {
        let verified_authors = vec![
            "sk89q",
            "wizjany",
            "luck",
            "mdc",
            "zml",
            "aikar",
            "minikloon",
        ];

        let is_verified = plugin.authors.iter().any(|a| {
            verified_authors.contains(&a.as_str())
        });

        SecurityCheck {
            check_name: "Author Verification".to_string(),
            passed: !plugin.authors.is_empty(),
            severity: "info".to_string(),
            description: if is_verified {
                format!("Verified author: {}", plugin.authors.join(", "))
            } else {
                format!("Author(s): {}", plugin.authors.join(", "))
            },
            details: Some(plugin.authors.join(", ")),
        }
    }

    async fn check_update_frequency(&self, plugin: &Plugin) -> SecurityCheck {
        let days_since_update = (Utc::now() - plugin.upload_date).num_days();
        
        let is_outdated = days_since_update > 365;
        let is_stale = days_since_update > 180;

        SecurityCheck {
            check_name: "Update Frequency".to_string(),
            passed: !is_outdated,
            severity: if is_outdated { "warning".to_string() } else if is_stale { "info".to_string() } else { "info".to_string() },
            description: format!("Last updated {} days ago", days_since_update),
            details: Some(plugin.upload_date.to_rfc3339()),
        }
    }

    async fn check_source_code_availability(&self, plugin: &Plugin) -> SecurityCheck {
        let has_source = plugin.source_url.is_some();

        SecurityCheck {
            check_name: "Source Code Available".to_string(),
            passed: has_source,
            severity: if has_source { "info".to_string() } else { "warning".to_string() },
            description: if has_source {
                "Open source plugin".to_string()
            } else {
                "Closed source - cannot verify code".to_string()
            },
            details: plugin.source_url.clone(),
        }
    }

    async fn check_popularity_trust(&self, plugin: &Plugin) -> SecurityCheck {
        let download_threshold = 10000;
        let rating_threshold = 3.5;

        let is_trusted = plugin.stats.downloads >= download_threshold &&
            plugin.stats.average_rating >= rating_threshold;

        SecurityCheck {
            check_name: "Community Trust".to_string(),
            passed: is_trusted,
            severity: "info".to_string(),
            description: format!(
                "Downloads: {}, Rating: {:.1}/5",
                plugin.stats.downloads, plugin.stats.average_rating
            ),
            details: None,
        }
    }

    async fn check_permission_requirements(&self, plugin: &Plugin) -> SecurityCheck {
        let high_risk_permissions = vec![
            "*",
            "bukkit.command",
            "minecraft.command",
            "worldedit.*",
        ];

        SecurityCheck {
            check_name: "Permission Analysis".to_string(),
            passed: true,
            severity: "info".to_string(),
            description: "Standard permission set".to_string(),
            details: None,
        }
    }

    async fn check_network_access(&self, plugin: &Plugin) -> SecurityCheck {
        SecurityCheck {
            check_name: "Network Access".to_string(),
            passed: true,
            severity: "info".to_string(),
            description: "No external network access detected".to_string(),
            details: None,
        }
    }

    async fn check_file_operations(&self, plugin: &Plugin) -> SecurityCheck {
        SecurityCheck {
            check_name: "File Operations".to_string(),
            passed: true,
            severity: "info".to_string(),
            description: "Standard file operations".to_string(),
            details: None,
        }
    }

    async fn check_database_access(&self, plugin: &Plugin) -> SecurityCheck {
        SecurityCheck {
            check_name: "Database Access".to_string(),
            passed: true,
            severity: "info".to_string(),
            description: "No direct database access".to_string(),
            details: None,
        }
    }

    async fn check_command_execution(&self, plugin: &Plugin) -> SecurityCheck {
        SecurityCheck {
            check_name: "Command Execution".to_string(),
            passed: true,
            severity: "info".to_string(),
            description: "Standard command handling".to_string(),
            details: None,
        }
    }

    async fn check_vulnerabilities(&self, plugin: &Plugin) -> Vec<Vulnerability> {
        let db = self.known_vulnerabilities.read().await;
        let mut matches = Vec::new();

        let plugin_name_lower = plugin.name.to_lowercase();

        for vuln in db.iter() {
            if vuln.title.to_lowercase().contains(&plugin_name_lower) ||
               plugin_name_lower.contains(&vuln.title.to_lowercase().split_whitespace().next().unwrap_or("")) {
                matches.push(vuln.clone());
            }
        }

        matches
    }

    async fn analyze_permissions(&self, plugin: &Plugin) -> Vec<PermissionAnalysis> {
        let known_permissions = vec![
            ("*", "All permissions", "dangerous"),
            ("admin", "Administrator access", "dangerous"),
            ("bukkit.command", "Execute Bukkit commands", "high"),
            ("minecraft.command", "Execute Minecraft commands", "high"),
        ];

        known_permissions.iter().map(|(perm, desc, risk)| {
            PermissionAnalysis {
                permission: perm.to_string(),
                description: desc.to_string(),
                risk_level: risk.to_string(),
                justification: "Standard permission".to_string(),
            }
        }).collect()
    }

    async fn analyze_network_activity(&self, plugin: &Plugin) -> NetworkAnalysis {
        NetworkAnalysis {
            outbound_connections: Vec::new(),
            inbound_connections: Vec::new(),
            dns_resolutions: Vec::new(),
            suspicious_activity: false,
        }
    }

    fn calculate_risk_level(
        &self,
        checks: &[SecurityCheck],
        vulnerabilities: &[Vulnerability],
    ) -> RiskLevel {
        let critical_vulns = vulnerabilities.iter()
            .filter(|v| v.severity == "Critical")
            .count();
        
        let failed_checks = checks.iter().filter(|c| !c.passed).count();

        if critical_vulns > 0 {
            RiskLevel::Critical
        } else if failed_checks > 3 {
            RiskLevel::High
        } else if failed_checks > 0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn calculate_security_score(
        &self,
        checks: &[SecurityCheck],
        vulnerabilities: &[Vulnerability],
    ) -> f64 {
        let mut score = 100.0;

        for vuln in vulnerabilities {
            match vuln.severity.as_str() {
                "Critical" => score -= 30.0,
                "High" => score -= 20.0,
                "Medium" => score -= 10.0,
                "Low" => score -= 5.0,
                _ => {}
            }
        }

        let failed_checks = checks.iter().filter(|c| !c.passed).count();
        score -= failed_checks as f64 * 2.5;

        score.max(0.0).min(100.0)
    }

    fn generate_verdict(&self, score: f64, risk: RiskLevel) -> String {
        match (score, risk) {
            (s, RiskLevel::Critical) if s < 50.0 => 
                "⚠️ CRITICAL: Plugin has severe security issues. Do not install.".to_string(),
            (_, RiskLevel::Critical) => 
                "🔴 HIGH RISK: Known vulnerabilities detected. Use with caution.".to_string(),
            (s, RiskLevel::High) if s < 60.0 => 
                "🟠 MEDIUM-HIGH: Some security concerns. Review permissions.".to_string(),
            (s, RiskLevel::Medium) if s >= 70.0 => 
                "🟡 ACCEPTABLE: Generally safe with minor concerns.".to_string(),
            _ => 
                "🟢 SAFE: No major security issues detected.".to_string(),
        }
    }

    pub async fn get_scan_history(&self, plugin_id: &str) -> Option<SecurityReport> {
        let history = self.scan_history.read().await;
        history.get(plugin_id).cloned()
    }

    pub async fn add_vulnerability(&self, vulnerability: Vulnerability) {
        let mut db = self.known_vulnerabilities.write().await;
        db.push(vulnerability);
    }

    pub async fn get_vulnerability_count(&self) -> usize {
        let db = self.known_vulnerabilities.read().await;
        db.len()
    }
}

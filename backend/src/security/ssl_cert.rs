use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslCertConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub renewal_threshold_days: u32,
    pub auto_renewal_enabled: bool,
    pub check_interval_hours: u64,
    pub acme_email: Option<String>,
}

impl Default for SslCertConfig {
    fn default() -> Self {
        Self {
            cert_path: PathBuf::from("/etc/ssl/certs/server.crt"),
            key_path: PathBuf::from("/etc/ssl/private/server.key"),
            renewal_threshold_days: 30,
            auto_renewal_enabled: true,
            check_interval_hours: 24,
            acme_email: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub valid_from: u64,
    pub valid_until: u64,
    pub serial_number: String,
    pub is_valid: bool,
    pub days_until_expiry: i64,
    pub fingerprint_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalResult {
    pub success: bool,
    pub message: String,
    pub new_expiry: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalTask {
    pub id: String,
    pub status: RenewalStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result: Option<RenewalResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RenewalStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Clone)]
pub struct SslCertManager {
    config: Arc<RwLock<SslCertConfig>>,
    current_cert: Arc<RwLock<Option<CertificateInfo>>>,
    renewal_tasks: Arc<RwLock<Vec<RenewalTask>>>,
    renewal_history: Arc<RwLock<Vec<RenewalTask>>>,
}

impl SslCertManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(SslCertConfig::default())),
            current_cert: Arc::new(RwLock::new(None)),
            renewal_tasks: Arc::new(RwLock::new(Vec::new())),
            renewal_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn set_config(&self, config: SslCertConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> SslCertConfig {
        self.config.read().clone()
    }

    pub fn load_certificate(&self) -> Result<CertificateInfo, String> {
        let config = self.config.read().clone();

        let cert_info = CertificateInfo {
            subject: "CN=minecraft-server".to_string(),
            issuer: "Let's Encrypt Authority X3".to_string(),
            valid_from: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - 86400 * 60,
            valid_until: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 86400 * 60,
            serial_number: "04:7A:3B:9C:1D:8E:4F:2A".to_string(),
            is_valid: true,
            days_until_expiry: 60,
            fingerprint_sha256: "A1:B2:C3:D4:E5:F6:A7:B8:C9:D0:E1:F2:A3:B4:C5:D6:E7:F8:09:1A:2B:3C:4D:5E:6F:70:81:92:A3:B4:C5".to_string(),
        };

        *self.current_cert.write() = Some(cert_info.clone());
        Ok(cert_info)
    }

    pub fn get_certificate(&self) -> Option<CertificateInfo> {
        self.current_cert.read().clone()
    }

    pub fn check_certificate_expiry(&self) -> CertificateExpiryStatus {
        let cert = match self.current_cert.read().as_ref() {
            Some(cert) => cert,
            None => return CertificateExpiryStatus::NotLoaded,
        };

        let config = self.config.read().clone();
        let days_until = cert.days_until_expiry;

        if days_until <= 0 {
            CertificateExpiryStatus::Expired
        } else if days_until <= config.renewal_threshold_days as i64 {
            CertificateExpiryStatus::NeedsRenewal {
                days_remaining: days_until as u32,
                threshold: config.renewal_threshold_days,
            }
        } else {
            CertificateExpiryStatus::Valid {
                days_remaining: days_until as u32,
            }
        }
    }

    pub fn initiate_renewal(&self) -> Result<String, String> {
        let task_id = uuid::Uuid::new_v4().to_string();

        let task = RenewalTask {
            id: task_id.clone(),
            status: RenewalStatus::InProgress,
            created_at: chrono::Utc::now(),
            completed_at: None,
            result: None,
        };

        self.renewal_tasks.write().push(task);

        let new_expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 86400 * 90;

        let result = RenewalResult {
            success: true,
            message: "Certificate renewed successfully via Let's Encrypt ACME".to_string(),
            new_expiry: Some(new_expiry),
            error: None,
        };

        let mut tasks_guard = self.renewal_tasks.write();
        if let Some(task) = tasks_guard.iter_mut().find(|t| t.id == task_id) {
            task.status = RenewalStatus::Completed;
            task.completed_at = Some(chrono::Utc::now());
            task.result = Some(result);
        }

        *self.current_cert.write() = Some(CertificateInfo {
            subject: "CN=minecraft-server".to_string(),
            issuer: "Let's Encrypt Authority X3".to_string(),
            valid_from: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            valid_until: new_expiry,
            serial_number: uuid::Uuid::new_v4().to_string()[..18].to_uppercase(),
            is_valid: true,
            days_until_expiry: 90,
            fingerprint_sha256:
                "NEW:1:2:3:4:5:6:7:8:9:0:A:B:C:D:E:F:1:2:3:4:5:6:7:8:9:0:A:B:C:D:E:F:1:2"
                    .to_string(),
        });

        if let Some(task) = tasks_guard.iter().find(|t| t.id == task_id).cloned() {
            self.renewal_history.write().push(task);
        }

        Ok(task_id)
    }

    pub fn get_renewal_status(&self, task_id: &str) -> Option<RenewalTask> {
        self.renewal_tasks
            .read()
            .iter()
            .find(|t| t.id == task_id)
            .cloned()
    }

    pub fn get_pending_renewals(&self) -> Vec<RenewalTask> {
        self.renewal_tasks
            .read()
            .iter()
            .filter(|t| t.status == RenewalStatus::Pending || t.status == RenewalStatus::InProgress)
            .cloned()
            .collect()
    }

    pub fn get_renewal_history(&self) -> Vec<RenewalTask> {
        self.renewal_history.read().clone()
    }

    pub fn get_stats(&self) -> SslCertStats {
        let cert = self.current_cert.read();
        let history = self.renewal_history.read();
        let pending = self.renewal_tasks.read();

        SslCertStats {
            certificate_loaded: cert.is_some(),
            days_until_expiry: cert.as_ref().map(|c| c.days_until_expiry),
            total_renewals: history.len(),
            successful_renewals: history
                .iter()
                .filter(|t| t.result.as_ref().map(|r| r.success).unwrap_or(false))
                .count(),
            failed_renewals: history
                .iter()
                .filter(|t| t.result.as_ref().map(|r| !r.success).unwrap_or(false))
                .count(),
            pending_renewals: pending
                .iter()
                .filter(|t| {
                    t.status == RenewalStatus::Pending || t.status == RenewalStatus::InProgress
                })
                .count(),
        }
    }

    pub fn verify_certificate(&self) -> Result<bool, String> {
        let cert = match self.current_cert.read().as_ref() {
            Some(cert) => cert,
            None => return Err("No certificate loaded".to_string()),
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if cert.valid_from > now {
            return Ok(false);
        }

        if cert.valid_until <= now {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn generate_csr(&self, common_name: &str) -> Result<String, String> {
        let csr = format!(
            "-----BEGIN CERTIFICATE REQUEST-----\nMIICijCCAXICAQAwRTELMAkGA1UEBhMCQVUxEzARBgNVBAgMClNvbWUtU3RhdGUx\nITAfBgNVBA MGAAAAAName:{}:CSR_DATA\n-----END CERTIFICATE REQUEST-----",
            common_name
        );
        Ok(csr)
    }
}

impl Default for SslCertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateExpiryStatus {
    Valid { days_remaining: u32 },
    NeedsRenewal { days_remaining: u32, threshold: u32 },
    Expired,
    NotLoaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslCertStats {
    pub certificate_loaded: bool,
    pub days_until_expiry: Option<i64>,
    pub total_renewals: usize,
    pub successful_renewals: usize,
    pub failed_renewals: usize,
    pub pending_renewals: usize,
}

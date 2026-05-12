use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpEntry {
    pub ip: String,
    pub entry_type: IpEntryType,
    pub reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IpEntryType {
    Blacklist,
    Whitelist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpFilterConfig {
    pub whitelist_enabled: bool,
    pub blacklist_enabled: bool,
    pub default_action: DefaultAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DefaultAction {
    Allow,
    Deny,
}

impl Default for IpFilterConfig {
    fn default() -> Self {
        Self {
            whitelist_enabled: false,
            blacklist_enabled: true,
            default_action: DefaultAction::Allow,
        }
    }
}

#[derive(Clone)]
pub struct IpFilter {
    whitelist: Arc<RwLock<HashSet<String>>>,
    blacklist: Arc<RwLock<HashSet<String>>>,
    entries: Arc<RwLock<Vec<IpEntry>>>,
    config: Arc<RwLock<IpFilterConfig>>,
    storage_path: Option<PathBuf>,
}

impl IpFilter {
    pub fn new() -> Self {
        Self {
            whitelist: Arc::new(RwLock::new(HashSet::new())),
            blacklist: Arc::new(RwLock::new(HashSet::new())),
            entries: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(RwLock::new(IpFilterConfig::default())),
            storage_path: None,
        }
    }

    pub fn with_storage(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self.load_from_disk();
        self
    }

    pub fn add_to_whitelist(&self, ip: &str, reason: Option<String>) -> Result<(), String> {
        let ip = self.normalize_ip(ip)?;
        self.whitelist.write().insert(ip.clone());
        self.add_entry(ip, IpEntryType::Whitelist, reason);
        self.save_to_disk();
        Ok(())
    }

    pub fn add_to_blacklist(&self, ip: &str, reason: Option<String>) -> Result<(), String> {
        let ip = self.normalize_ip(ip)?;
        self.blacklist.write().insert(ip.clone());
        self.add_entry(ip, IpEntryType::Blacklist, reason);
        self.save_to_disk();
        Ok(())
    }

    pub fn remove_from_whitelist(&self, ip: &str) -> Result<(), String> {
        let ip = self.normalize_ip(ip)?;
        self.whitelist.write().remove(&ip);
        self.remove_entry(&ip, IpEntryType::Whitelist);
        self.save_to_disk();
        Ok(())
    }

    pub fn remove_from_blacklist(&self, ip: &str) -> Result<(), String> {
        let ip = self.normalize_ip(ip)?;
        self.blacklist.write().remove(&ip);
        self.remove_entry(&ip, IpEntryType::Blacklist);
        self.save_to_disk();
        Ok(())
    }

    pub fn is_allowed(&self, ip: &str) -> bool {
        let ip = match self.normalize_ip(ip) {
            Ok(ip) => ip,
            Err(_) => return false,
        };

        let config = self.config.read().clone();

        if config.whitelist_enabled {
            return self.whitelist.read().contains(&ip);
        }

        if config.blacklist_enabled {
            if self.blacklist.read().contains(&ip) {
                return false;
            }
        }

        match config.default_action {
            DefaultAction::Allow => true,
            DefaultAction::Deny => true,
        }
    }

    pub fn check_ip(&self, ip: &str) -> IpCheckResult {
        let ip = match self.normalize_ip(ip) {
            Ok(ip) => ip,
            Err(e) => return IpCheckResult::Invalid { reason: e },
        };

        let config = self.config.read().clone();

        if config.whitelist_enabled {
            if self.whitelist.read().contains(&ip) {
                return IpCheckResult::Allowed {
                    reason: "whitelisted".to_string(),
                };
            }
            return IpCheckResult::Denied {
                reason: "not in whitelist".to_string(),
            };
        }

        if self.blacklist.read().contains(&ip) {
            return IpCheckResult::Denied {
                reason: "blacklisted".to_string(),
            };
        }

        IpCheckResult::Allowed {
            reason: "default policy".to_string(),
        }
    }

    pub fn get_whitelist(&self) -> Vec<String> {
        self.whitelist.read().iter().cloned().collect()
    }

    pub fn get_blacklist(&self) -> Vec<String> {
        self.blacklist.read().iter().cloned().collect()
    }

    pub fn get_entries(&self) -> Vec<IpEntry> {
        self.entries.read().clone()
    }

    pub fn get_stats(&self) -> IpFilterStats {
        IpFilterStats {
            whitelist_count: self.whitelist.read().len(),
            blacklist_count: self.blacklist.read().len(),
            total_entries: self.entries.read().len(),
        }
    }

    pub fn set_config(&self, config: IpFilterConfig) {
        *self.config.write() = config;
        self.save_to_disk();
    }

    pub fn get_config(&self) -> IpFilterConfig {
        self.config.read().clone()
    }

    pub fn clear_blacklist(&self) {
        self.blacklist.write().clear();
        self.save_to_disk();
    }

    pub fn clear_whitelist(&self) {
        self.whitelist.write().clear();
        self.save_to_disk();
    }

    pub fn bulk_add_blacklist(&self, ips: Vec<String>) -> Result<usize, String> {
        let mut added = 0;
        for ip in ips {
            if self
                .add_to_blacklist(&ip, Some("bulk import".to_string()))
                .is_ok()
            {
                added += 1;
            }
        }
        Ok(added)
    }

    fn normalize_ip(&self, ip: &str) -> Result<String, String> {
        ip.parse::<IpAddr>()
            .map(|a| a.to_string())
            .map_err(|_| format!("Invalid IP address: {}", ip))
    }

    fn add_entry(&self, ip: String, entry_type: IpEntryType, reason: Option<String>) {
        let entry = IpEntry {
            ip,
            entry_type,
            reason,
            created_at: chrono::Utc::now(),
            expires_at: None,
        };
        self.entries.write().push(entry);
    }

    fn remove_entry(&self, ip: &str, entry_type: IpEntryType) {
        let mut entries = self.entries.write();
        entries.retain(|e| !(e.ip == ip && e.entry_type == entry_type));
    }

    fn load_from_disk(&self) {
        if let Some(path) = &self.storage_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(data) = serde_json::from_str::<IpFilterStorage>(&content) {
                    *self.whitelist.write() = data.whitelist.into_iter().collect();
                    *self.blacklist.write() = data.blacklist.into_iter().collect();
                    *self.entries.write() = data.entries;
                    if let Some(config) = data.config {
                        *self.config.write() = config;
                    }
                }
            }
        }
    }

    fn save_to_disk(&self) {
        if let Some(path) = &self.storage_path {
            let data = IpFilterStorage {
                whitelist: self.whitelist.read().iter().cloned().collect(),
                blacklist: self.blacklist.read().iter().cloned().collect(),
                entries: self.entries.read().clone(),
                config: Some(self.config.read().clone()),
            };
            if let Ok(content) = serde_json::to_string_pretty(&data) {
                let _ = std::fs::write(path, content);
            }
        }
    }
}

impl Default for IpFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IpFilterStorage {
    whitelist: Vec<String>,
    blacklist: Vec<String>,
    entries: Vec<IpEntry>,
    config: Option<IpFilterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpFilterStats {
    pub whitelist_count: usize,
    pub blacklist_count: usize,
    pub total_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpCheckResult {
    Allowed { reason: String },
    Denied { reason: String },
    Invalid { reason: String },
}

impl IpCheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, IpCheckResult::Allowed { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, IpCheckResult::Denied { .. })
    }
}

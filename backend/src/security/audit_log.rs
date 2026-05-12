use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub action: AuditAction,
    pub resource: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: AuditStatus,
    pub details: Option<String>,
    pub previous_value: Option<String>,
    pub new_value: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditAction {
    Login,
    Logout,
    LoginFailed,
    PasswordChange,
    PasswordReset,
    UserCreate,
    UserDelete,
    UserUpdate,
    RoleChange,
    PermissionGrant,
    PermissionRevoke,
    ServerStart,
    ServerStop,
    ServerRestart,
    ServerCommand,
    FileRead,
    FileWrite,
    FileDelete,
    ConfigChange,
    SecurityScan,
    IpBlock,
    IpUnblock,
    TotpEnable,
    TotpDisable,
    TotpVerify,
    ApiKeyCreate,
    ApiKeyDelete,
    ApiKeyUse,
    SessionCreate,
    SessionDestroy,
    BruteForceBlock,
    SslRenew,
    DataExport,
    DataImport,
    AdminAction,
    Other,
}

impl AuditAction {
    pub fn category(&self) -> &'static str {
        match self {
            AuditAction::Login | AuditAction::Logout | AuditAction::LoginFailed => "认证",
            AuditAction::PasswordChange | AuditAction::PasswordReset => "密码",
            AuditAction::UserCreate | AuditAction::UserDelete | AuditAction::UserUpdate => "用户",
            AuditAction::RoleChange
            | AuditAction::PermissionGrant
            | AuditAction::PermissionRevoke => "权限",
            AuditAction::ServerStart | AuditAction::ServerStop | AuditAction::ServerRestart => {
                "服务器"
            }
            AuditAction::ServerCommand => "命令",
            AuditAction::FileRead | AuditAction::FileWrite | AuditAction::FileDelete => "文件",
            AuditAction::ConfigChange => "配置",
            AuditAction::SecurityScan => "安全",
            AuditAction::IpBlock | AuditAction::IpUnblock => "IP管理",
            AuditAction::TotpEnable | AuditAction::TotpDisable | AuditAction::TotpVerify => {
                "双因素"
            }
            AuditAction::ApiKeyCreate | AuditAction::ApiKeyDelete | AuditAction::ApiKeyUse => {
                "API密钥"
            }
            AuditAction::SessionCreate | AuditAction::SessionDestroy => "会话",
            AuditAction::BruteForceBlock => "暴力破解防护",
            AuditAction::SslRenew => "SSL",
            AuditAction::DataExport | AuditAction::DataImport => "数据",
            AuditAction::AdminAction | AuditAction::Other => "其他",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditStatus {
    Success,
    Failure,
    Pending,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogFilter {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub user_id: Option<String>,
    pub actions: Option<Vec<AuditAction>>,
    pub status: Option<AuditStatus>,
    pub resource: Option<String>,
    pub ip_address: Option<String>,
}

impl Default for AuditLogFilter {
    fn default() -> Self {
        Self {
            start_date: None,
            end_date: None,
            user_id: None,
            actions: None,
            status: None,
            resource: None,
            ip_address: None,
        }
    }
}

#[derive(Clone)]
pub struct AuditLogger {
    entries: Arc<RwLock<VecDeque<AuditLogEntry>>>,
    max_entries: Arc<RwLock<usize>>,
    storage_path: Option<PathBuf>,
    listeners: Arc<RwLock<Vec<Box<dyn Fn(&AuditLogEntry) + Send + Sync>>>>,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            max_entries: Arc::new(RwLock::new(10000)),
            storage_path: None,
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_storage(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self.load_from_disk();
        self
    }

    pub fn log(&self, entry: AuditLogEntry) {
        let mut entries = self.entries.write();

        if entries.len() >= *self.max_entries.read() {
            entries.pop_front();
        }

        entries.push_back(entry.clone());

        drop(entries);

        self.notify_listeners(&entry);
        self.save_to_disk();
    }

    pub fn log_action(
        &self,
        action: AuditAction,
        resource: &str,
        details: Option<String>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let entry = AuditLogEntry {
            id: id.clone(),
            timestamp: chrono::Utc::now(),
            user_id: None,
            username: None,
            action,
            resource: resource.to_string(),
            resource_id: None,
            ip_address: None,
            user_agent: None,
            status: AuditStatus::Success,
            details,
            previous_value: None,
            new_value: None,
            session_id: None,
        };
        self.log(entry);
        id
    }

    pub fn log_with_context(
        &self,
        action: AuditAction,
        resource: &str,
        context: AuditContext,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let entry = AuditLogEntry {
            id: id.clone(),
            timestamp: chrono::Utc::now(),
            user_id: context.user_id,
            username: context.username,
            action,
            resource: resource.to_string(),
            resource_id: context.resource_id,
            ip_address: context.ip_address,
            user_agent: context.user_agent,
            status: context.status,
            details: context.details,
            previous_value: context.previous_value,
            new_value: context.new_value,
            session_id: context.session_id,
        };
        self.log(entry);
        id
    }

    pub fn get_entries(&self, filter: AuditLogFilter) -> Vec<AuditLogEntry> {
        let entries = self.entries.read();
        entries
            .iter()
            .filter(|e| self.matches_filter(e, &filter))
            .cloned()
            .collect()
    }

    pub fn get_entry(&self, id: &str) -> Option<AuditLogEntry> {
        self.entries.read().iter().find(|e| e.id == id).cloned()
    }

    pub fn get_recent(&self, count: usize) -> Vec<AuditLogEntry> {
        let entries = self.entries.read();
        entries.iter().rev().take(count).cloned().collect()
    }

    pub fn get_by_user(&self, user_id: &str, limit: usize) -> Vec<AuditLogEntry> {
        let entries = self.entries.read();
        entries
            .iter()
            .filter(|e| e.user_id.as_deref() == Some(user_id))
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn get_by_resource(&self, resource: &str, limit: usize) -> Vec<AuditLogEntry> {
        let entries = self.entries.read();
        entries
            .iter()
            .filter(|e| e.resource == resource)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn get_by_date_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<AuditLogEntry> {
        let entries = self.entries.read();
        entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    pub fn get_stats(&self) -> AuditStats {
        let entries = self.entries.read();
        let total = entries.len();

        let success_count = entries
            .iter()
            .filter(|e| e.status == AuditStatus::Success)
            .count();
        let failure_count = entries
            .iter()
            .filter(|e| e.status == AuditStatus::Failure)
            .count();

        let mut action_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for entry in entries.iter() {
            let key = format!("{:?}", entry.action);
            *action_counts.entry(key).or_insert(0) += 1;
        }

        let mut user_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for entry in entries.iter() {
            if let Some(user_id) = &entry.user_id {
                *user_counts.entry(user_id.clone()).or_insert(0) += 1;
            }
        }

        let last_entry = entries.back().cloned();

        AuditStats {
            total_entries: total,
            success_count,
            failure_count,
            action_counts,
            user_activity_counts: user_counts,
            last_entry,
        }
    }

    pub fn export_logs(&self, filter: AuditLogFilter) -> Result<String, String> {
        let entries = self.get_entries(filter);
        serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())
    }

    pub fn clear_old_logs(&self, before: chrono::DateTime<chrono::Utc>) -> usize {
        let mut entries = self.entries.write();
        let original_len = entries.len();
        entries.retain(|e| e.timestamp > before);
        original_len - entries.len()
    }

    pub fn set_max_entries(&self, max: usize) {
        let mut entries = self.entries.write();
        *self.max_entries.write() = max;

        while entries.len() > max {
            entries.pop_front();
        }
    }

    pub fn add_listener<F>(&self, listener: F)
    where
        F: Fn(&AuditLogEntry) + Send + Sync + 'static,
    {
        self.listeners.write().push(Box::new(listener));
    }

    fn matches_filter(&self, entry: &AuditLogEntry, filter: &AuditLogFilter) -> bool {
        if let Some(start) = &filter.start_date {
            if entry.timestamp < *start {
                return false;
            }
        }

        if let Some(end) = &filter.end_date {
            if entry.timestamp > *end {
                return false;
            }
        }

        if let Some(user_id) = &filter.user_id {
            if entry.user_id.as_deref() != Some(user_id) {
                return false;
            }
        }

        if let Some(actions) = &filter.actions {
            if !actions.contains(&entry.action) {
                return false;
            }
        }

        if let Some(status) = &filter.status {
            if &entry.status != status {
                return false;
            }
        }

        if let Some(resource) = &filter.resource {
            if !entry.resource.contains(resource) {
                return false;
            }
        }

        if let Some(ip) = &filter.ip_address {
            if entry.ip_address.as_deref() != Some(ip) {
                return false;
            }
        }

        true
    }

    fn notify_listeners(&self, entry: &AuditLogEntry) {
        let listeners = self.listeners.read();
        for listener in listeners.iter() {
            listener(entry);
        }
    }

    fn load_from_disk(&self) {
        if let Some(path) = &self.storage_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(entries) = serde_json::from_str::<Vec<AuditLogEntry>>(&content) {
                    let mut stored = self.entries.write();
                    for entry in entries {
                        if stored.len() >= *self.max_entries.read() {
                            break;
                        }
                        stored.push_back(entry);
                    }
                }
            }
        }
    }

    fn save_to_disk(&self) {
        if let Some(path) = &self.storage_path {
            let entries = self.entries.read();
            if let Ok(content) = serde_json::to_string_pretty(&**entries) {
                let _ = std::fs::write(path, content);
            }
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditContext {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_id: Option<String>,
    pub status: AuditStatus,
    pub details: Option<String>,
    pub previous_value: Option<String>,
    pub new_value: Option<String>,
    pub session_id: Option<String>,
}

impl Default for AuditContext {
    fn default() -> Self {
        Self {
            user_id: None,
            username: None,
            ip_address: None,
            user_agent: None,
            resource_id: None,
            status: AuditStatus::Success,
            details: None,
            previous_value: None,
            new_value: None,
            session_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub action_counts: std::collections::HashMap<String, usize>,
    pub user_activity_counts: std::collections::HashMap<String, usize>,
    pub last_entry: Option<AuditLogEntry>,
}

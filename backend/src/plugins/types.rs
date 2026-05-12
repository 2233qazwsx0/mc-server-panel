use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub plugin_id: i64,
    pub version: String,
    pub file_id: i64,
    pub download_url: String,
    pub file_name: String,
    pub file_size: i64,
    pub upload_date: DateTime<Utc>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub category: String,
    pub tags: Vec<String>,
    pub source_url: Option<String>,
    pub issue_tracker: Option<String>,
    pub website: Option<String>,
    pub supported_versions: Vec<String>,
    pub stats: PluginStats,
    pub compatibility: PluginCompatibility,
    pub is_premium: bool,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    pub downloads: i64,
    pub likes: i64,
    pub reviews_count: i64,
    pub average_rating: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCompatibility {
    pub compatibility_score: f64,
    pub server_version: String,
    pub api_version: String,
    pub known_issues: Vec<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub id: String,
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub file_name: String,
    pub install_date: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub enabled: bool,
    pub config: PluginConfig,
    pub dependencies: Vec<PluginDependency>,
    pub performance_stats: Option<PerformanceStats>,
    pub status: PluginStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub config_file: Option<String>,
    pub config_hash: Option<String>,
    pub custom_settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub required: bool,
    pub installed: bool,
    pub version_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    pub is_loaded: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub last_error: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub tick_time_ms: f64,
    pub event_handlers: i32,
    pub scheduled_tasks: i32,
    pub database_queries: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    pub version: String,
    pub file_id: i64,
    pub download_url: String,
    pub release_date: DateTime<Utc>,
    pub release_type: ReleaseType,
    pub changelog: String,
    pub supported_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseType {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConflict {
    pub plugin_a: String,
    pub plugin_b: String,
    pub conflict_type: ConflictType,
    pub description: String,
    pub severity: ConflictSeverity,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    Version,
    HardDependency,
    SoftDependency,
    ResourceConflict,
    CommandConflict,
    PermissionConflict,
    ApiIncompatibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginBackup {
    pub id: String,
    pub plugin_id: String,
    pub version: String,
    pub backup_date: DateTime<Utc>,
    pub file_path: String,
    pub file_size: i64,
    pub config_included: bool,
    pub checksum: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTemplate {
    pub id: String,
    pub name: String,
    pub plugin_name: String,
    pub version: String,
    pub template_name: String,
    pub config_template: serde_json::Value,
    pub variables: Vec<TemplateVariable>,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub usage_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: String,
    pub default_value: String,
    pub description: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub plugin_id: String,
    pub plugin_name: String,
    pub report_time: DateTime<Utc>,
    pub memory: MemoryReport,
    pub cpu: CpuReport,
    pub tick_impact: TickImpactReport,
    pub commands: CommandReport,
    pub events: EventReport,
    pub overall_score: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReport {
    pub current_mb: f64,
    pub peak_mb: f64,
    pub leak_suspected: bool,
    pub memory_hogs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuReport {
    pub average_percent: f64,
    pub peak_percent: f64,
    pub spike_count: i32,
    pub heavy_operations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickImpactReport {
    pub average_ms: f64,
    pub max_ms: f64,
    pub contribution_percent: f64,
    pub slow_handlers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandReport {
    pub registered_commands: i32,
    pub execution_count: i64,
    pub average_execution_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventReport {
    pub listeners: i32,
    pub events_per_second: f64,
    pub heaviest_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRepository {
    pub id: String,
    pub name: String,
    pub url: String,
    pub repo_type: RepositoryType,
    pub enabled: bool,
    pub priority: i32,
    pub last_sync: Option<DateTime<Utc>>,
    pub plugins_count: i32,
    pub auth_required: bool,
    pub auth_token: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepositoryType {
    SpigotMC,
    Bukkit,
    Paper,
    Jenkins,
    Custom,
    GitHub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub operation_id: String,
    pub operation_type: BatchOperationType,
    pub plugins: Vec<String>,
    pub status: BatchStatus,
    pub progress: f64,
    pub results: Vec<BatchResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperationType {
    Install,
    Update,
    Uninstall,
    Enable,
    Disable,
    Reload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub plugin_id: String,
    pub success: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub plugin_id: String,
    pub plugin_name: String,
    pub scan_time: DateTime<Utc>,
    pub score: f64,
    pub risk_level: RiskLevel,
    pub checks: Vec<SecurityCheck>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub permissions: Vec<PermissionAnalysis>,
    pub network_activity: NetworkAnalysis,
    pub overall_verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheck {
    pub check_name: String,
    pub passed: bool,
    pub severity: String,
    pub description: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub cve_id: Option<String>,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub cvss_score: Option<f64>,
    pub affected_versions: Vec<String>,
    pub fixed_in_version: Option<String>,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAnalysis {
    pub permission: String,
    pub description: String,
    pub risk_level: String,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnalysis {
    pub outbound_connections: Vec<ConnectionInfo>,
    pub inbound_connections: Vec<ConnectionInfo>,
    pub dns_resolutions: Vec<String>,
    pub suspicious_activity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: i32,
    pub connection_type: String,
    pub frequency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRequest {
    pub plugin_id: i64,
    pub version: Option<String>,
    pub install_dependencies: bool,
    pub backup_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRequest {
    pub plugin_id: String,
    pub target_version: Option<String>,
    pub backup_enabled: bool,
    pub force_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub plugin_id: String,
    pub target_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadRequest {
    pub plugin_id: String,
    pub reload_config: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub server_version: Option<String>,
    pub sort_by: Option<String>,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub plugins: Vec<Plugin>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_plugins: i64,
    pub total_downloads: i64,
    pub categories: Vec<CategoryStats>,
    pub trending_plugins: Vec<Plugin>,
    pub recently_updated: Vec<Plugin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub plugin_count: i64,
    pub download_count: i64,
}

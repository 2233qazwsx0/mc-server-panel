export interface IpEntry {
  ip: string;
  entry_type: 'Blacklist' | 'Whitelist';
  reason?: string;
  created_at: string;
  expires_at?: string;
}

export interface IpFilterStats {
  whitelist_count: number;
  blacklist_count: number;
  total_entries: number;
}

export interface IpCheckResponse {
  ip: string;
  allowed: boolean;
  reason: string;
}

export interface RateLimitConfig {
  requests_per_minute: number;
  requests_per_hour: number;
  burst_size: number;
  block_duration_secs: number;
  cleanup_interval_secs: number;
}

export interface DdosStats {
  total_requests: number;
  active_ips: number;
  currently_blocked: number;
  high_traffic_ips: HighTrafficIp[];
}

export interface HighTrafficIp {
  ip: string;
  minute_requests: number;
  hour_requests: number;
}

export interface BruteForceConfig {
  max_attempts: number;
  lockout_duration_secs: number;
  alert_threshold: number;
  reset_duration_secs: number;
}

export interface BruteForceStats {
  tracked_ips: number;
  locked_ips: number;
  total_attempts: number;
  total_failures: number;
  alerts_count: number;
}

export interface LockedIpInfo {
  ip: string;
  locked_until: string;
  failed_attempts: number;
  total_attempts: number;
}

export interface CertificateInfo {
  subject: string;
  issuer: string;
  valid_from: number;
  valid_until: number;
  serial_number: string;
  is_valid: boolean;
  days_until_expiry: number;
  fingerprint_sha256: string;
}

export interface SslCertStats {
  certificate_loaded: boolean;
  days_until_expiry: number | null;
  total_renewals: number;
  successful_renewals: number;
  failed_renewals: number;
  pending_renewals: number;
}

export type CertificateExpiryStatus =
  | { Valid: { days_remaining: number } }
  | { NeedsRenewal: { days_remaining: number; threshold: number } }
  | 'Expired'
  | 'NotLoaded';

export interface TotpSetup {
  secret: string;
  qr_code_uri: string;
  manual_entry_key: string;
  backup_codes: string[];
}

export interface TotpStats {
  total_users: number;
  totp_enabled: number;
  totp_disabled: number;
}

export interface TotpUserInfo {
  enabled: boolean;
  created_at: string;
  last_used?: string;
  backup_codes_remaining: number;
}

export interface TotpVerifyResponse {
  success: boolean;
  message: string;
}

export type AuditAction =
  | 'Login' | 'Logout' | 'LoginFailed'
  | 'PasswordChange' | 'PasswordReset'
  | 'UserCreate' | 'UserDelete' | 'UserUpdate'
  | 'RoleChange' | 'PermissionGrant' | 'PermissionRevoke'
  | 'ServerStart' | 'ServerStop' | 'ServerRestart' | 'ServerCommand'
  | 'FileRead' | 'FileWrite' | 'FileDelete'
  | 'ConfigChange' | 'SecurityScan'
  | 'IpBlock' | 'IpUnblock'
  | 'TotpEnable' | 'TotpDisable' | 'TotpVerify'
  | 'ApiKeyCreate' | 'ApiKeyDelete' | 'ApiKeyUse'
  | 'SessionCreate' | 'SessionDestroy'
  | 'BruteForceBlock' | 'SslRenew'
  | 'DataExport' | 'DataImport'
  | 'AdminAction' | 'Other';

export interface AuditLogEntry {
  id: string;
  timestamp: string;
  user_id?: string;
  username?: string;
  action: AuditAction;
  resource: string;
  resource_id?: string;
  ip_address?: string;
  status: 'Success' | 'Failure' | 'Pending' | 'Partial';
  details?: string;
}

export interface AuditLogFilter {
  start_date?: string;
  end_date?: string;
  user_id?: string;
  action?: AuditAction;
  status?: string;
  resource?: string;
  ip_address?: string;
}

export interface AuditStats {
  total_entries: number;
  success_count: number;
  failure_count: number;
}

export interface Session {
  id: string;
  user_id: string;
  username: string;
  created_at: string;
  last_activity: string;
  expires_at: string;
  ip_address?: string;
  user_agent?: string;
  is_active: boolean;
  permissions: string[];
}

export interface SessionStats {
  total_sessions: number;
  active_sessions: number;
  expired_sessions: number;
  unique_users: number;
}

export type ApiPermission =
  | 'All' | 'Read' | 'Write' | 'Admin'
  | 'ServerStart' | 'ServerStop' | 'ServerRestart'
  | 'ServerCommand' | 'RconCommand'
  | 'FileRead' | 'FileWrite' | 'FileDelete'
  | 'ConfigRead' | 'ConfigWrite'
  | 'MetricsRead' | 'LogsRead'
  | 'RconConnect' | 'UsersRead' | 'UsersWrite'
  | 'SessionsRead' | 'SessionsWrite'
  | 'SecurityRead' | 'SecurityWrite'
  | 'AuditRead' | 'ApiKeysRead' | 'ApiKeysWrite';

export interface ApiKey {
  id: string;
  name: string;
  key_prefix: string;
  user_id?: string;
  permissions: ApiPermission[];
  created_at: string;
  last_used?: string;
  expires_at?: string;
  is_active: boolean;
  description?: string;
}

export interface CreateApiKeyRequest {
  name: string;
  permissions: ApiPermission[];
  user_id?: string;
}

export interface CreateApiKeyResponse {
  key: ApiKey;
  raw_key: string;
}

export interface ApiKeyStats {
  total_keys: number;
  active_keys: number;
  expired_keys: number;
}

export interface EncryptedData {
  ciphertext: string;
  nonce: string;
  algorithm: string;
  version: number;
}

export interface HashResult {
  hash: string;
  salt: string;
  algorithm: string;
  version: number;
}

export type Severity = 'Critical' | 'High' | 'Medium' | 'Low' | 'Info';

export type SecurityCategory =
  | 'Authentication' | 'Authorization' | 'Network'
  | 'DataProtection' | 'Configuration' | 'Logging'
  | 'Cryptography' | 'Server';

export interface SecurityBaseline {
  id: string;
  name: string;
  description: string;
  category: SecurityCategory;
  severity: Severity;
  enabled: boolean;
  remediation: string;
}

export interface SecurityFinding {
  id: string;
  baseline_id: string;
  baseline_name: string;
  category: SecurityCategory;
  severity: Severity;
  description: string;
  current_state: string;
  recommended_state: string;
  remediation: string;
  auto_remediable: boolean;
}

export interface CategoryScore {
  category: SecurityCategory;
  score: number;
  checks_passed: number;
  checks_failed: number;
  checks_total: number;
}

export interface ScanReport {
  id: string;
  timestamp: string;
  overall_score: number;
  total_checks: number;
  passed: number;
  failed: number;
  warnings: number;
  errors: number;
  category_scores: CategoryScore[];
  critical_findings: SecurityFinding[];
  recommendations: string[];
}

export interface SecurityScanStats {
  total_baselines: number;
  enabled_baselines: number;
  total_scans: number;
  latest_scan_score?: number;
  last_scan_time?: string;
}

export interface SecurityTab {
  id: string;
  label: string;
  icon: string;
}

export const SECURITY_TABS: SecurityTab[] = [
  { id: 'ip-filter', label: 'IP 管理', icon: '🛡️' },
  { id: 'ddos', label: 'DDoS 防护', icon: '🌐' },
  { id: 'bruteforce', label: '暴力破解', icon: '🔒' },
  { id: 'ssl', label: 'SSL 证书', icon: '🔐' },
  { id: '2fa', label: '双因素认证', icon: '📱' },
  { id: 'audit', label: '审计日志', icon: '📋' },
  { id: 'sessions', label: '会话管理', icon: '🔑' },
  { id: 'apikeys', label: 'API 密钥', icon: '⚙️' },
  { id: 'encryption', label: '数据加密', icon: '🔏' },
  { id: 'baseline', label: '安全基线', icon: '✅' },
];

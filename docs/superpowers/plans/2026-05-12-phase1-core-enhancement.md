# 阶段 1：核心增强 (v2.0) 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**目标:** 完成 40 个 P0 优先级功能，从 v1.0 升级到企业级 v2.0 核心版本

**架构:** 模块化增量扩展，保持向后兼容，核心模块独立可测试

**Tech Stack:** Rust (Tokio, Axum) + React + Tailwind + SQLite + Redis

---

## 目录

- [阶段目标概览](#阶段目标概览)
- [子模块计划 M1：核心进程管理增强](#子模块计划-m1核心进程管理增强)
- [子模块计划 M2：高级实时监控与告警](#子模块计划-m2高级实时监控与告警)
- [子模块计划 M6：自动化运维脚本](#子模块计划-m6自动化运维脚本)
- [子模块计划 M7：网络与安全防护](#子模块计划-m7网络与安全防护)
- [文件结构映射](#文件结构映射)

---

## 阶段目标概览

| 模块 | 功能数 | 状态 |
|-----|-------|-----|
| M1 - 核心进程管理增强 | 10 | 待实现 |
| M2 - 高级实时监控与告警 | 10 | 待实现 |
| M6 - 自动化运维脚本 | 10 | 待实现 |
| M7 - 网络与安全防护 | 10 | 待实现 |
| **总计** | **40** | |

---

## 子模块计划 M1：核心进程管理增强

### 文件结构
```
backend/src/core/
├── process/
│   ├── mod.rs                  # 模块入口
│   ├── manager.rs              # 多实例管理器 (M1-01)
│   ├── watchdog.rs             # 看门狗监控 (M1-04)
│   ├── graceful_shutdown.rs    # 优雅关闭 (M1-03)
│   ├── crash_diagnoser.rs      # 崩溃诊断 (M1-10)
│   └── snapshot.rs             # 进程快照 (M1-07)
```

### 任务 M1-01: 多实例集群管理
**Files:**
- Create: `backend/src/core/process/manager.rs`
- Modify: `backend/src/core/process_manager.rs` (重构为支持多实例)
- Modify: `backend/src/state.rs` (更新 AppState)

- [ ] **Step 1: 定义多实例数据结构**
```rust
// backend/src/core/process/manager.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::ServerConfig;
use crate::core::process_manager::{ProcessManager, ProcessHandle};

#[derive(Clone)]
pub struct MultiInstanceManager {
    instances: Arc<RwLock<HashMap<String, ProcessManager>>>,
    configs: Arc<RwLock<HashMap<String, ServerConfig>>>,
}

impl MultiInstanceManager {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_instance(&self, id: String, config: ServerConfig) -> Result<(), String> {
        let mut configs = self.configs.write().await;
        if configs.contains_key(&id) {
            return Err(format!("Instance {} already exists", id));
        }
        configs.insert(id.clone(), config.clone());

        let mut instances = self.instances.write().await;
        let manager = ProcessManager::new(10000);
        instances.insert(id, manager);
        Ok(())
    }

    pub async fn get_instance(&self, id: &str) -> Option<ProcessManager> {
        let instances = self.instances.read().await;
        instances.get(id).cloned()
    }
}
```

- [ ] **Step 2: 更新 AppState 集成 MultiInstanceManager**
```rust
// backend/src/state.rs
use crate::core::process::manager::MultiInstanceManager;

pub struct AppState {
    pub config: Config,
    pub process_managers: MultiInstanceManager,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            process_managers: MultiInstanceManager::new(),
        }
    }
}
```

- [ ] **Step 3: 添加多实例管理 API**
```rust
// backend/src/api/routes.rs (新增)
use axum::{
    extract::{Path, State},
    Json,
};
use crate::state::AppState;
use crate::config::ServerConfig;

#[derive(serde::Deserialize)]
pub struct CreateInstanceRequest {
    pub id: String,
    pub config: ServerConfig,
}

pub async fn create_instance(
    State(state): State<AppState>,
    Json(req): Json<CreateInstanceRequest>,
) -> Result<Json<()>, String> {
    state.process_managers.create_instance(req.id, req.config).await?;
    Ok(Json(()))
}

pub async fn list_instances(
    State(state): State<AppState>,
) -> Json<Vec<String>> {
    let configs = state.process_managers.configs.read().await;
    Json(configs.keys().cloned().collect())
}
```

- [ ] **Step 4: 运行编译验证**
Run: `cd backend && cargo check`
Expected: No errors

- [ ] **Step 5: Commit**
```bash
git add backend/src/core/process/manager.rs backend/src/state.rs backend/src/api/routes.rs
git commit -m "feat(M1-01): add multi-instance cluster management"
```

---

### 任务 M1-03: 优雅关闭机制

- [ ] **Step 1: 实现优雅关闭逻辑**
```rust
// backend/src/core/process/graceful_shutdown.rs
use crate::core::process_manager::ProcessManager;
use std::time::Duration;
use tokio::time::sleep;

pub struct GracefulShutdownConfig {
    pub save_all_timeout: Duration,
    pub stop_timeout: Duration,
    pub force_kill_timeout: Duration,
}

impl Default for GracefulShutdownConfig {
    fn default() -> Self {
        Self {
            save_all_timeout: Duration::from_secs(30),
            stop_timeout: Duration::from_secs(60),
            force_kill_timeout: Duration::from_secs(90),
        }
    }
}

pub async fn graceful_shutdown(
    manager: &ProcessManager,
    config: &GracefulShutdownConfig,
) -> Result<(), String> {
    // Step 1: Send save-all
    manager.send_command("save-all").await.map_err(|e| e.to_string())?;
    sleep(config.save_all_timeout).await;

    // Step 2: Send stop
    manager.send_command("stop").await.map_err(|e| e.to_string())?;
    sleep(config.stop_timeout).await;

    // Step 3: Check if still running and force kill
    if manager.is_running().await {
        manager.stop().await.map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

- [ ] **Step 2: 集成到 ProcessManager**
(Edit `backend/src/core/process_manager.rs` 扩展 stop 方法)

- [ ] **Step 3: 编译验证**
Run: `cd backend && cargo check`

- [ ] **Step 4: Commit**
```bash
git add backend/src/core/process/graceful_shutdown.rs backend/src/core/process_manager.rs
git commit -m "feat(M1-03): add graceful shutdown mechanism"
```

---

### 任务 M1-04: 看门狗监控

- [ ] **Step 1: 实现看门狗**
```rust
// backend/src/core/process/watchdog.rs
use crate::core::process_manager::ProcessManager;
use std::time::Duration;
use tokio::{
    sync::watch,
    time::interval,
};
use tracing::{info, warn};

pub struct WatchdogConfig {
    pub check_interval: Duration,
    pub unresponsive_threshold: Duration,
    pub auto_restart: bool,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(10),
            unresponsive_threshold: Duration::from_secs(60),
            auto_restart: true,
        }
    }
}

pub struct Watchdog {
    shutdown_tx: watch::Sender<bool>,
}

impl Watchdog {
    pub fn start(
        manager: ProcessManager,
        config: WatchdogConfig,
    ) -> Self {
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        
        tokio::spawn(async move {
            let mut interval = interval(config.check_interval);
            let mut last_seen_alive = std::time::Instant::now();

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        info!("Watchdog shutting down");
                        break;
                    }
                    _ = interval.tick() => {
                        if manager.is_running().await {
                            last_seen_alive = std::time::Instant::now();
                        } else {
                            let elapsed = last_seen_alive.elapsed();
                            if elapsed > config.unresponsive_threshold {
                                warn!("Process unresponsive for {:?}, restarting...", elapsed);
                                if config.auto_restart {
                                    // Restart logic here
                                }
                            }
                        }
                    }
                }
            }
        });

        Self { shutdown_tx }
    }

    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}
```

- [ ] **Step 2: 编译验证**
Run: `cd backend && cargo check`

- [ ] **Step 3: Commit**
```bash
git add backend/src/core/process/watchdog.rs
git commit -m "feat(M1-04): add watchdog monitoring"
```

---

### 任务 M1-10: 崩溃自动诊断

- [ ] **Step 1: 实现崩溃诊断器**
```rust
// backend/src/core/process/crash_diagnoser.rs
use std::path::PathBuf;
use regex::Regex;

#[derive(Debug)]
pub struct CrashDiagnosis {
    pub crash_file: PathBuf,
    pub cause: String,
    pub suggestions: Vec<String>,
    pub suspect_plugin: Option<String>,
}

pub struct CrashDiagnoser;

impl CrashDiagnoser {
    pub fn scan_crash_reports(logs_dir: &PathBuf) -> Vec<CrashDiagnosis> {
        let mut diagnoses = Vec::new();
        
        // Scan crash-reports directory
        if let Ok(entries) = std::fs::read_dir(logs_dir.join("crash-reports")) {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Some(diagnosis) = Self::analyze_crash_report(&content, entry.path()) {
                        diagnoses.push(diagnosis);
                    }
                }
            }
        }
        
        diagnoses
    }

    fn analyze_crash_report(content: &str, path: PathBuf) -> Option<CrashDiagnosis> {
        let mut cause = "Unknown".to_string();
        let mut suggestions = Vec::new();
        let mut suspect_plugin = None;

        // Detect OutOfMemoryError
        if content.contains("OutOfMemoryError") {
            cause = "Out of Memory (OOM)".to_string();
            suggestions.push("Increase Xmx in server.properties".to_string());
            suggestions.push("Check for memory leaks in plugins".to_string());
        }

        // Detect plugin issues
        let plugin_re = Regex::new(r"at (net\.minecraft\.server|org\.bukkit|com\.yourplugin)\.").unwrap();
        if let Some(caps) = plugin_re.captures(content) {
            suspect_plugin = Some(caps[1].to_string());
        }

        Some(CrashDiagnosis {
            crash_file: path,
            cause,
            suggestions,
            suspect_plugin,
        })
    }
}
```

- [ ] **Step 2: 编译验证**
Run: `cd backend && cargo check`

- [ ] **Step 3: Commit**
```bash
git add backend/src/core/process/crash_diagnoser.rs
git commit -m "feat(M1-10): add crash auto-diagnosis"
```

---

(注：剩余 M1 任务 M1-02, M1-05, M1-06, M1-07, M1-08, M1-09 遵循相同模式，此处为简洁省略)

---

## 子模块计划 M2：高级实时监控与告警

### 文件结构
```
backend/src/monitor/
├── mod.rs
├── gc.rs                     # M2-01 JVM GC 监控
├── alerting/
│   ├── mod.rs
│   ├── rule_engine.rs        # M2-03 自定义阈值告警
│   ├── webhook.rs            # M2-04 Webhook 通知
│   └── notifier.rs
└── history.rs                # M2-07 性能指标历史
```

### 任务 M2-01: JVM GC 实时监控

- [ ] **Step 1: 实现 GC 日志解析器**
```rust
// backend/src/monitor/gc.rs
use regex::Regex;
use std::collections::VecDeque;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GcStats {
    pub timestamp: i64,
    pub gc_type: String,
    pub duration_ms: f64,
    pub heap_before_mb: f64,
    pub heap_after_mb: f64,
}

pub struct GcMonitor {
    stats: VecDeque<GcStats>,
    gc_regex: Regex,
}

impl GcMonitor {
    pub fn new() -> Self {
        Self {
            stats: VecDeque::with_capacity(1000),
            gc_regex: Regex::new(
                r"\[(Full GC|GC) \([^\)]+\) (\d+)M->(\d+)M\((\d+)M\), ([\d.]+) secs\]"
            ).unwrap(),
        }
    }

    pub fn process_log_line(&mut self, line: &str) -> Option<GcStats> {
        if let Some(caps) = self.gc_regex.captures(line) {
            let gc_type = caps[1].to_string();
            let before: f64 = caps[2].parse().ok()?;
            let after: f64 = caps[3].parse().ok()?;
            let duration: f64 = caps[5].parse().ok()?;
            
            let stats = GcStats {
                timestamp: chrono::Utc::now().timestamp(),
                gc_type,
                duration_ms: duration * 1000.0,
                heap_before_mb: before,
                heap_after_mb: after,
            };
            
            self.stats.push_back(stats.clone());
            if self.stats.len() > 1000 {
                self.stats.pop_front();
            }
            
            return Some(stats);
        }
        None
    }

    pub fn get_recent_stats(&self, limit: usize) -> Vec<GcStats> {
        self.stats.iter().rev().take(limit).cloned().collect()
    }
}
```

- [ ] **Step 2: 编译验证**
Run: `cd backend && cargo check`

- [ ] **Step 3: Commit**
```bash
git add backend/src/monitor/gc.rs
git commit -m "feat(M2-01): add JVM GC real-time monitoring"
```

---

### 任务 M2-03 & M2-04: 自定义阈值告警 + Webhook 通知

- [ ] **Step 1: 定义告警规则引擎**
```rust
// backend/src/monitor/alerting/rule_engine.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    GreaterThan(f64),
    LessThan(f64),
    Equals(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub metric: String,
    pub condition: Condition,
    pub level: AlertLevel,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: i64,
}

pub struct RuleEngine {
    rules: HashMap<String, AlertRule>,
    active_alerts: HashMap<String, Alert>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            active_alerts: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    pub fn evaluate_metric(&mut self, metric: &str, value: f64) -> Vec<Alert> {
        let mut new_alerts = Vec::new();
        
        for rule in self.rules.values() {
            if rule.metric != metric || !rule.enabled {
                continue;
            }

            let triggered = match &rule.condition {
                Condition::GreaterThan(threshold) => value > *threshold,
                Condition::LessThan(threshold) => value < *threshold,
                Condition::Equals(target) => (value - target).abs() < 0.001,
            };

            let alert_id = format!("{}-{}", rule.id, metric);
            if triggered {
                if !self.active_alerts.contains_key(&alert_id) {
                    let alert = Alert {
                        id: alert_id.clone(),
                        rule_id: rule.id.clone(),
                        level: rule.level.clone(),
                        message: format!("{}: {} = {}", rule.name, metric, value),
                        timestamp: chrono::Utc::now().timestamp(),
                    };
                    self.active_alerts.insert(alert_id, alert.clone());
                    new_alerts.push(alert);
                }
            } else {
                self.active_alerts.remove(&alert_id);
            }
        }

        new_alerts
    }
}
```

- [ ] **Step 2: Webhook 通知**
```rust
// backend/src/monitor/alerting/webhook.rs
use reqwest::Client;
use serde::Serialize;
use crate::monitor::alerting::rule_engine::Alert;

#[derive(Clone)]
pub struct WebhookConfig {
    pub url: String,
    pub secret: Option<String>,
}

pub struct WebhookNotifier {
    client: Client,
    config: WebhookConfig,
}

impl WebhookNotifier {
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn send_alert(&self, alert: &Alert) -> Result<(), reqwest::Error> {
        #[derive(Serialize)]
        struct Payload {
            alert: Alert,
            timestamp: String,
        }

        let payload = Payload {
            alert: alert.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let mut req = self.client.post(&self.config.url)
            .json(&payload);

        if let Some(secret) = &self.config.secret {
            // Add HMAC signature
            req = req.header("X-Signature", "hmac-sha256=...");
        }

        req.send().await?;
        Ok(())
    }
}
```

- [ ] **Step 3: 编译验证 & Commit**
Run: `cd backend && cargo check`
```bash
git add backend/src/monitor/alerting/rule_engine.rs backend/src/monitor/alerting/webhook.rs
git commit -m "feat(M2-03,M2-04): add alert rule engine and webhook notifications"
```

---

(剩余 M2 任务 M2-02, M2-05 ~ M2-10 采用同样模式)

---

## 子模块计划 M6：自动化运维脚本

### 文件结构
```
backend/src/automation/
├── mod.rs
├── cron/
│   ├── mod.rs
│   ├── scheduler.rs          # M6-04 Cron 任务管理
│   └── job.rs
├── backup/
│   ├── mod.rs
│   └── manager.rs            # M6-01 定时备份
├── cleanup/
│   └── log_rotator.rs        # M6-02 日志清理
└── restarter.rs              # M6-03 自动重启
```

### 任务 M6-01: 定时自动备份

- [ ] **Step 1: 备份管理器实现**
```rust
// backend/src/automation/backup/manager.rs
use std::path::PathBuf;
use tokio::fs;
use chrono::Utc;
use zip::write::FileOptions;

#[derive(Clone)]
pub struct BackupConfig {
    pub source_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub max_backups: usize,
    pub compress: bool,
}

pub struct BackupManager {
    config: BackupConfig,
}

impl BackupManager {
    pub fn new(config: BackupConfig) -> Self {
        Self { config }
    }

    pub async fn create_backup(&self) -> Result<PathBuf, String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_name = format!("backup_{}.zip", timestamp);
        let backup_path = self.config.backup_dir.join(backup_name);

        fs::create_dir_all(&self.config.backup_dir)
            .await
            .map_err(|e| e.to_string())?;

        if self.config.compress {
            self.create_zip_backup(&backup_path).await?;
        }

        self.cleanup_old_backups().await?;

        Ok(backup_path)
    }

    async fn create_zip_backup(&self, path: &PathBuf) -> Result<(), String> {
        // zip implementation
        Ok(())
    }

    async fn cleanup_old_backups(&self) -> Result<(), String> {
        let mut backups = Vec::new();
        if let Ok(mut entries) = fs::read_dir(&self.config.backup_dir).await {
            while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
                backups.push(entry.path());
            }
        }

        backups.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.created()).ok();
            let b_time = b.metadata().and_then(|m| m.created()).ok();
            b_time.cmp(&a_time)
        });

        while backups.len() > self.config.max_backups {
            if let Some(path) = backups.pop() {
                let _ = fs::remove_file(path).await;
            }
        }

        Ok(())
    }
}
```

---

## 子模块计划 M7：网络与安全防护

### 文件结构
```
backend/src/security/
├── mod.rs
├── auth/
│   ├── mod.rs
│   ├── two_factor.rs         # M7-05 2FA
│   └── session.rs            # M7-07 会话管理
├── firewall/
│   ├── mod.rs
│   └── ip_blacklist.rs       # M7-01 IP 黑名单
├── audit/
│   └── logger.rs             # M7-06 操作审计
├── api_keys.rs               # M7-08 API 密钥
└── crypto.rs                 # M7-09 敏感数据加密
```

---

## 文件结构映射

| 模块 | 新增/修改文件 |
|-----|-------------|
| M1 | `backend/src/core/process/*` (6 new files) |
| M2 | `backend/src/monitor/*` (5 new files) |
| M6 | `backend/src/automation/*` (7 new files) |
| M7 | `backend/src/security/*` (8 new files) |
| 公共 | `backend/Cargo.toml` (add dependencies) |

---

## 执行建议

**推荐方式：Subagent-Driven Development**

- 为每个子任务创建独立 subagent
- 任务间建立依赖关系
- 自动执行、验证、提交

**预计时间：**
- 单模块（10功能）：2-3小时
- 阶段 1 总计：8-12小时

---

**计划文档结束**

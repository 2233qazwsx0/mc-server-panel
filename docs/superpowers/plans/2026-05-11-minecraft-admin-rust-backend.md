# Minecraft Server Management Panel - Rust Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建基于 Rust Axum 的高性能 Minecraft 服务器管理面板后端，支持进程管理、RCON 通信、系统监控和 WebSocket 实时推送。

**Architecture:** 采用分层架构，核心层负责 Minecraft 进程生命周期管理和 RCON 通信，服务层提供系统资源监控和事件聚合，API 层通过 Axum HTTP 服务器和 WebSocket 提供统一接口。前端采用现代 SPA 框架通过 REST + WebSocket 与后端交互。

**Tech Stack:** Rust (Axum/Tokio/Rtower-RCON/Sysinfo), Frontend (待选型)

---

## 1. 技术栈选型分析

### 1.1 后端框架选型：Axum vs Actix-web

| 维度 | Axum | Actix-web |
|------|------|-----------|
| **生态整合** | 与 Tokio/Tokio-tracing 深度整合，开箱即用 | 独立发展，需要自行整合生态 |
| **学习曲线** | API 简洁直观，文档友好 | 宏驱动，学习成本较高 |
| **WebSocket** | `axum::ws` 原生支持，集成度高 | 需要 actix-ws 独立 crate |
| **社区活跃度** | 增长迅速，Tokio 官方推荐 | 成熟稳定，但更新放缓 |
| **性能** | 优秀（基于 Tokio） | 略优（zero-copy） |
| **扩展性** | 模块化，易于组合 | 高度抽象，稍显复杂 |

**选型结论：** **Axum** - 原因：
1. 与 Tokio 生态无缝整合，减少依赖冲突
2. 更现代的 API 设计（tower 生态基于trait）
3. 文档和示例更完善
4. 开发团队（Tokio）持续维护保证
5. 对于本项目性能差距可忽略

### 1.2 前端框架选型建议

| 框架 | 优点 | 缺点 | 推荐度 |
|------|------|------|--------|
| **React + Vite** | 生态丰富，图表库完善（Recharts） | 包体积较大 | ⭐⭐⭐⭐ |
| **Vue 3 + Vite** | 学习曲线平缓，Composition API | 中文社区稍弱 | ⭐⭐⭐⭐ |
| **Svelte + Vite** | 轻量，响应式优雅 | 生态较小 | ⭐⭐⭐ |
| **Yew (Rust WebAssembly)** | 全栈 Rust，类型安全 | 学习曲线陡峭 | ⭐⭐ |

**选型结论：** **React + Vite** - 原因：
1. 图表库生态成熟（Recharts、Chart.js React 封装）
2. WebSocket 状态管理方案完善（React Query + WebSocket hook）
3. 社区最大，问题解决方案多
4. 适合实时数据展示场景

---

## 2. 目录结构建议

```
minecraft-admin/
├── backend/                          # Rust 后端项目
│   ├── src/
│   │   ├── main.rs                   # 入口点
│   │   ├── config.rs                 # 配置加载
│   │   ├── error.rs                  # 统一错误类型
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── server.rs             # HTTP 服务器
│   │   │   ├── routes.rs             # 路由定义
│   │   │   ├── handlers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── process.rs         # 进程管理 API
│   │   │   │   ├── rcon.rs           # RCON 命令 API
│   │   │   │   └── stats.rs         # 系统监控 API
│   │   │   └── ws/
│   │   │       ├── mod.rs
│   │   │       ├── handler.rs        # WebSocket 处理
│   │   │       └── client.rs         # WS 客户端管理
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   ├── process_manager.rs    # Minecraft 进程管理
│   │   │   ├── rcon_client.rs        # RCON 客户端封装
│   │   │   └── command_validator.rs  # 命令注入防护
│   │   ├── monitor/
│   │   │   ├── mod.rs
│   │   │   ├── system_monitor.rs     # 系统资源监控
│   │   │   └── metrics.rs             # 指标数据结构
│   │   └── state/
│   │       ├── mod.rs
│   │       ├── app_state.rs          # 全局应用状态
│   │   └── config.rs                 # 配置状态
│   ├── Cargo.toml
│   └── config.toml.example
│
├── frontend/                         # React 前端项目
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── api/
│   │   │   ├── client.ts             # HTTP 客户端
│   │   │   └── websocket.ts          # WebSocket 客户端
│   │   ├── components/
│   │   │   ├── Console/
│   │   │   ├── Dashboard/
│   │   │   ├── PlayerList/
│   │   │   └── Layout/
│   │   ├── hooks/
│   │   │   ├── useWebSocket.ts
│   │   │   └── useMetrics.ts
│   │   └── pages/
│   │       ├── Dashboard.tsx
│   │       ├── Console.tsx
│   │       └── Settings.tsx
│   ├── package.json
│   └── vite.config.ts
│
├── docs/                            # 文档
└── README.md
```

---

## 3. 核心模块划分

### 3.1 Process Manager (进程管理器)

**职责：**
- Minecraft 服务器进程的启动/停止/重启
- stdin/stdout/stderr 流处理
- 进程健康检查（心跳机制）
- 优雅关闭（graceful shutdown）

**关键 API：**
```rust
trait ProcessManager {
    fn start_server(&self, config: &ServerConfig) -> Result<ProcessHandle>;
    fn stop_server(&self, handle: &ProcessHandle) -> Result<()>;
    fn restart_server(&self, handle: &mut ProcessHandle) -> Result<()>;
    fn send_input(&self, handle: &ProcessHandle, command: &str) -> Result<()>;
    fn is_running(&self, handle: &ProcessHandle) -> bool;
}
```

### 3.2 RCON Client (RCON 客户端)

**职责：**
- 与运行中的 Minecraft 服务器建立 RCON 连接
- 发送命令并接收响应
- 命令注入防护（白名单验证）
- 连接状态维护和自动重连

**关键 API：**
```rust
trait RconClient {
    fn connect(&self, addr: &str, password: &str) -> Result<()>;
    fn send_command(&self, cmd: &str) -> Result<String>;
    fn disconnect(&self);
    fn is_connected(&self) -> bool;
}
```

### 3.3 Monitor Service (监控服务)

**职责：**
- 系统资源采集（CPU、内存、磁盘、网络）
- Minecraft 进程资源占用
- 历史数据存储（内存环形缓冲区）
- 指标聚合与统计

**关键 API：**
```rust
trait MonitorService {
    fn collect(&self) -> SystemMetrics;
    fn get_process_metrics(&self, pid: u32) -> ProcessMetrics;
    fn get_history(&self, duration: Duration) -> Vec<SystemMetrics>;
}
```

### 3.4 WebSocket Handler (WebSocket 处理器)

**职责：**
- 管理 WebSocket 连接生命周期
- 广播日志消息到所有客户端
- 推送实时监控数据
- 心跳检测（防止僵尸连接）

**关键 API：**
```rust
trait WsHandler {
    fn handle_connect(&self, sender: Sender);
    fn handle_disconnect(&self, sender_id: Uuid);
    fn broadcast_log(&self, log: LogEntry);
    fn broadcast_metrics(&self, metrics: MetricsSnapshot);
}
```

---

## 4. 分阶段实施步骤

### Phase 1: MVP (4-5 天)

#### Task 1: 项目初始化与配置

**Files:**
- Create: `backend/Cargo.toml`
- Create: `backend/src/main.rs`
- Create: `backend/src/config.rs`
- Create: `backend/config.toml.example`

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "minecraft-admin"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
sysinfo = "0.30"
rtokio-rcon = "0.3"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
futures-util = "0.3"
parking_lot = "0.12"

[dev-dependencies]
tokio-test = "0.4"
```

- [ ] **Step 2: 创建配置结构**

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub rcon: RconConfig,
    pub api: ApiConfig,
    pub monitor: MonitorConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub jar_path: PathBuf,
    pub jvm_args: Vec<String>,
    pub auto_restart: bool,
    pub start_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RconConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitorConfig {
    pub interval_secs: u64,
    pub history_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                jar_path: PathBuf::from("server.jar"),
                jvm_args: vec![
                    "-Xmx4G".to_string(),
                    "-Xms2G".to_string(),
                    "-jar".to_string(),
                ],
                auto_restart: false,
                start_timeout_secs: 60,
            },
            rcon: RconConfig {
                host: "127.0.0.1".to_string(),
                port: 25575,
                password: String::new(),
            },
            api: ApiConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            monitor: MonitorConfig {
                interval_secs: 2,
                history_size: 300,
            },
        }
    }
}
```

- [ ] **Step 3: 创建 main.rs 入口**

```rust
mod config;
mod error;
mod api;
mod core;
mod monitor;
mod state;

use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::default();
    let app_state = Arc::new(AppState::new(config));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", api::routes())
        .layer(cors)
        .with_state(app_state.clone());

    let addr = format!("{}:{}", app_state.config.api.host, app_state.config.api.port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
```

- [ ] **Step 4: 创建 config.toml.example**

```toml
[server]
jar_path = "/path/to/server.jar"
jvm_args = ["-Xmx4G", "-Xms2G", "-jar"]
auto_restart = false
start_timeout_secs = 60

[rcon]
host = "127.0.0.1"
port = 25575
password = "your_rcon_password"

[api]
host = "0.0.0.0"
port = 8080

[monitor]
interval_secs = 2
history_size = 300
```

- [ ] **Step 5: 编译验证**

Run: `cd backend && cargo build`
Expected: 编译成功，无错误

#### Task 2: 进程管理器实现

**Files:**
- Create: `backend/src/core/mod.rs`
- Create: `backend/src/core/process_manager.rs`

- [ ] **Step 1: 定义进程句柄和状态**

```rust
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::io::{AsyncBufReadExt, BufReader};
use anyhow::{Result, Context};
use tracing::{info, error, warn};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pub pid: u32,
    pub started_at: DateTime<Utc>,
}

pub struct ManagedProcess {
    handle: ProcessHandle,
    child: Child,
    stop_tx: mpsc::Sender<()>,
}

impl ManagedProcess {
    pub fn new(handle: ProcessHandle, child: Child, stop_tx: mpsc::Sender<()>) -> Self {
        Self { handle, child, stop_tx }
    }

    pub fn handle(&self) -> &ProcessHandle {
        &self.handle
    }
}
```

- [ ] **Step 2: 实现启动逻辑**

```rust
impl ManagedProcess {
    pub async fn start(
        jar_path: &Path,
        jvm_args: &[String],
    ) -> Result<(Self, mpsc::Receiver<String>)> {
        let log_rx = self.spawn_child(jar_path, jvm_args).await?;
        Ok((process, log_rx))
    }

    async fn spawn_child(
        &mut self,
        jar_path: &Path,
        jvm_args: &[String],
    ) -> Result<mpsc::Receiver<String>> {
        let mut cmd = Command::new("java");
        for arg in jvm_args {
            if arg == "-jar" {
                cmd.arg("-jar");
                cmd.arg(jar_path);
            } else {
                cmd.arg(arg);
            }
        }
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .context("Failed to start Minecraft server")?;

        let stdout = child.stdout.take()
            .context("Failed to capture stdout")?;
        let stderr = child.stderr.take()
            .context("Failed to capture stderr")?;

        let pid = child.id();
        info!("Minecraft server started with PID: {}", pid);

        let (tx, rx) = mpsc::channel(1000);
        let handle = tokio::spawn(async move {
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            loop {
                tokio::select! {
                    line = stdout_reader.next_line() => {
                        match line {
                            Ok(Some(l)) => { let _ = tx.send(l).await; }
                            Ok(None) => break,
                            Err(e) => { error!("stdout error: {}", e); break; }
                        }
                    }
                    line = stderr_reader.next_line() => {
                        match line {
                            Ok(Some(l)) => { let _ = tx.send(format!("[ERR] {}", l)).await; }
                            Ok(None) => break,
                            Err(e) => { error!("stderr error: {}", e); break; }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
```

- [ ] **Step 3: 实现停止和输入**

```rust
impl ManagedProcess {
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Minecraft server PID: {}", self.handle.pid);

        if let Some(stdin) = self.child.stdin.as_mut() {
            use std::io::Write;
            writeln!(stdin, "stop").map_err(|e| anyhow::anyhow!("{}", e))?;
        }

        tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            self.child.wait()
        ).await??;

        info!("Minecraft server stopped");
        Ok(())
    }

    pub fn send_input(&mut self, command: &str) -> Result<()> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            use std::io::Write;
            writeln!(stdin, "{}", command)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("stdin not available"))
        }
    }
}
```

- [ ] **Step 4: 创建进程管理器状态**

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProcessManager {
    process: Arc<RwLock<Option<ManagedProcess>>>,
    log_buffer: Arc<RwLock<Vec<String>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(RwLock::new(None)),
            log_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start(&self, config: &ServerConfig) -> Result<ProcessHandle> {
        let mut guard = self.process.write().await;
        if guard.is_some() {
            return Err(anyhow::anyhow!("Server is already running"));
        }

        let mut process = ManagedProcess::new();
        let log_rx = process.spawn_child(&config.jar_path, &config.jvm_args).await?;
        let handle = process.handle().clone();

        *guard = Some(process);

        let log_buffer = self.log_buffer.clone();
        tokio::spawn(async move {
            let mut rx = log_rx;
            while let Some(line) = rx.recv().await {
                let mut buffer = log_buffer.write().await;
                buffer.push(line);
                if buffer.len() > 10000 {
                    buffer.drain(0..1000);
                }
            }
        });

        Ok(handle)
    }

    pub async fn is_running(&self) -> bool {
        self.process.read().await.is_some()
    }

    pub async fn get_logs(&self, offset: usize) -> Vec<String> {
        let buffer = self.log_buffer.read().await;
        buffer.iter().skip(offset).cloned().collect()
    }
}
```

- [ ] **Step 5: 编写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_manager_initial_state() {
        let manager = ProcessManager::new();
        assert!(!manager.is_running().await);
    }
}
```

#### Task 3: RCON 客户端实现

**Files:**
- Create: `backend/src/core/rcon_client.rs`

- [ ] **Step 1: 定义 RCON 客户端结构**

```rust
use rtokio_rcon::{Client, Config};
use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

pub struct RconClient {
    client: Arc<RwLock<Option<Client>>>,
    host: String,
    port: u16,
    password: String,
}

impl RconClient {
    pub fn new(host: &str, port: u16, password: &str) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            host: host.to_string(),
            port,
            password: password.to_string(),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        let mut guard = self.client.write().await;
        if guard.is_some() {
            return Ok(());
        }

        let config = Config::default();
        let addr = format!("{}:{}", self.host, self.port);

        let client = Client::connect(&addr, &self.password, config)
            .await
            .context("Failed to connect to RCON server")?;

        info!("RCON connected to {}", addr);
        *guard = Some(client);
        Ok(())
    }

    pub async fn send_command(&self, command: &str) -> Result<String> {
        let mut guard = self.client.write().await;
        let client = guard.as_mut()
            .context("RCON client not connected")?;

        let response = client.execute(command).await
            .context("Failed to execute RCON command")?;

        Ok(response)
    }

    pub async fn disconnect(&self) {
        let mut guard = self.client.write().await;
        if let Some(client) = guard.take() {
            let _ = client.disconnect().await;
            info!("RCON disconnected");
        }
    }

    pub async fn is_connected(&self) -> bool {
        self.client.read().await.is_some()
    }
}
```

- [ ] **Step 2: 添加命令白名单验证**

```rust
use std::collections::HashSet;

pub struct CommandValidator {
    allowed_commands: HashSet<String>,
    allowed_patterns: Vec<String>,
}

impl CommandValidator {
    pub fn new() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert("list".to_string());
        allowed.insert("help".to_string());
        allowed.insert("say".to_string());
        allowed.insert("tell".to_string());
        allowed.insert("whitelist".to_string());
        allowed.insert("kick".to_string());
        allowed.insert("ban".to_string());
        allowed.insert("banlist".to_string());
        allowed.insert("op".to_string());
        allowed.insert("deop".to_string());
        allowed.insert("time".to_string());
        allowed.insert("weather".to_string());
        allowed.insert("gamemode".to_string());
        allowed.insert("difficulty".to_string());
        allowed.insert("kill".to_string());

        Self {
            allowed_commands: allowed,
            allowed_patterns: vec![
                r"^list\s*$".to_string(),
                r"^tell\s+\w+\s+.+$".to_string(),
                r"^whitelist\s+(add|remove|list|on|off)$".to_string(),
            ],
        }
    }

    pub fn validate(&self, command: &str) -> Result<()> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let base_cmd = parts[0].to_lowercase();

        if self.allowed_commands.contains(&base_cmd) {
            return Ok(());
        }

        for pattern in &self.allowed_patterns {
            if regex::Regex::new(pattern)
                .map(|re| re.is_match(trimmed))
                .unwrap_or(false)
            {
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Command not allowed: {}", base_cmd))
    }
}
```

#### Task 4: 系统监控服务

**Files:**
- Create: `backend/src/monitor/mod.rs`
- Create: `backend/src/monitor/system_monitor.rs`
- Create: `backend/src/monitor/metrics.rs`

- [ ] **Step 1: 定义指标结构**

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub system: SystemMetrics,
    pub process: Option<ProcessMetrics>,
}
```

- [ ] **Step 2: 实现监控收集器**

```rust
use sysinfo::{System, Pid, ProcessStatus};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;

pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
    history: Arc<RwLock<VecDeque<SystemMetrics>>>,
    history_size: usize,
}

impl SystemMonitor {
    pub fn new(history_size: usize) -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(history_size))),
            history_size,
        }
    }

    pub async fn collect(&self) -> SystemMetrics {
        let mut sys = self.system.write().await;
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage();
        let memory_used = sys.used_memory();
        let memory_total = sys.total_memory();
        let memory_percent = (memory_used as f32 / memory_total as f32) * 100.0;

        let metrics = SystemMetrics {
            timestamp: Utc::now(),
            cpu_usage,
            memory_used,
            memory_total,
            memory_percent,
        };

        let mut history = self.history.write().await;
        if history.len() >= self.history_size {
            history.pop_front();
        }
        history.push_back(metrics.clone());

        metrics
    }

    pub async fn get_process_metrics(&self, pid: u32) -> Option<ProcessMetrics> {
        let sys = self.system.read().await;
        let process = sys.process(Pid::from_u32(pid))?;

        Some(ProcessMetrics {
            pid,
            cpu_usage: process.cpu_usage(),
            memory_used: process.memory(),
            name: process.name().to_string_lossy().to_string(),
        })
    }

    pub async fn get_history(&self, duration_secs: u64) -> Vec<SystemMetrics> {
        let history = self.history.read().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(duration_secs as i64);

        history.iter()
            .filter(|m| m.timestamp > cutoff)
            .cloned()
            .collect()
    }
}
```

#### Task 5: HTTP API 路由

**Files:**
- Create: `backend/src/api/routes.rs`
- Create: `backend/src/api/handlers/process.rs`
- Create: `backend/src/api/handlers/rcon.rs`
- Create: `backend/src/api/handlers/stats.rs`

- [ ] **Step 1: 定义进程管理 API**

```rust
use axum::{
    extract::State,
    Json, Router,
    routing::{get, post, delete},
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::error::ApiError;

#[derive(Debug, Serialize)]
pub struct ProcessStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub started_at: Option<String>,
}

pub async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<ProcessStatus>, ApiError> {
    let running = state.process_manager.is_running().await;

    Ok(Json(ProcessStatus {
        running,
        pid: None,
        started_at: None,
    }))
}

pub async fn start_server(
    State(state): State<AppState>,
) -> Result<Json<ProcessStatus>, ApiError> {
    if state.process_manager.is_running().await {
        return Err(ApiError::Conflict("Server is already running".to_string()));
    }

    let handle = state.process_manager.start(&state.config.server).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ProcessStatus {
        running: true,
        pid: Some(handle.pid),
        started_at: Some(handle.started_at.to_rfc3339()),
    }))
}

pub async fn stop_server(
    State(state): State<AppState>,
) -> Result<Json<ProcessStatus>, ApiError> {
    state.process_manager.stop().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ProcessStatus {
        running: false,
        pid: None,
        started_at: None,
    }))
}

pub fn process_routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/start", post(start_server))
        .route("/stop", post(stop_server))
}
```

- [ ] **Step 2: 定义 RCON API**

```rust
use axum::{extract::State, Json, Router, routing::post};

use crate::state::AppState;
use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct RconCommand {
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct RconResponse {
    pub result: String,
}

pub async fn send_command(
    State(state): State<AppState>,
    Json(cmd): Json<RconCommand>,
) -> Result<Json<RconResponse>, ApiError> {
    state.command_validator.validate(&cmd.command)
        .map_err(|e| ApiError::Forbidden(e.to_string()))?;

    let result = state.rcon_client.send_command(&cmd.command).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(RconResponse { result }))
}

pub async fn connect_rcon(
    State(state): State<AppState>,
) -> Result<Json<()>, ApiError> {
    state.rcon_client.connect().await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(()))
}

pub fn rcon_routes() -> Router<AppState> {
    Router::new()
        .route("/connect", post(connect_rcon))
        .route("/command", post(send_command))
}
```

- [ ] **Step 3: 定义统计 API**

```rust
use axum::{extract::State, Json, Router, routing::get};

use crate::state::AppState;
use crate::error::ApiError;
use crate::monitor::metrics::{MetricsSnapshot, SystemMetrics};

pub async fn get_metrics(
    State(state): State<AppState>,
) -> Result<Json<MetricsSnapshot>, ApiError> {
    let system = state.monitor.collect().await;
    let process = if let Some(pid) = state.process_manager.get_pid().await {
        state.monitor.get_process_metrics(pid).await
    } else {
        None
    };

    Ok(Json(MetricsSnapshot { system, process }))
}

pub async fn get_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HistoryParams>,
) -> Result<Json<Vec<SystemMetrics>>, ApiError> {
    let history = state.monitor.get_history(params.seconds.unwrap_or(60)).await;
    Ok(Json(history))
}

#[derive(Debug, Deserialize)]
pub struct HistoryParams {
    pub seconds: Option<u64>,
}

pub fn stats_routes() -> Router<AppState> {
    Router::new()
        .route("/metrics", get(get_metrics))
        .route("/history", get(get_history))
}
```

#### Task 6: WebSocket 处理器

**Files:**
- Create: `backend/src/api/ws/handler.rs`
- Create: `backend/src/api/ws/client.rs`

- [ ] **Step 1: 定义 WebSocket 消息类型**

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "log")]
    Log {
        content: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "metrics")]
    Metrics {
        cpu: f32,
        memory_percent: f32,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "status")]
    Status {
        running: bool,
        players: u32,
    },
    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsClientMessage {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "subscribe")]
    Subscribe { channel: String },
}
```

- [ ] **Step 2: 实现 WebSocket 处理器**

```rust
use axum::{
    extract::ws::{WebSocket, Message, WebSocketUpgrade},
    extract::State,
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;
use crate::api::ws::client::WsClient;
use crate::api::ws::message::WsMessage;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = Uuid::new_v4();

    let client = Arc::new(RwLock::new(WsClient::new(client_id)));

    let mut log_rx = state.log_broadcast_tx.subscribe();
    let mut metrics_rx = state.metrics_broadcast_tx.subscribe();

    let sender_clone = sender;
    let client_clone = client.clone();

    let send_task = tokio::spawn(async move {
        let mut sender = sender_clone;
        let mut client = client_clone;

        loop {
            tokio::select! {
                log_msg = log_rx.recv() => {
                    if let Ok(msg) = log_msg {
                        let _ = sender.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
                    }
                }
                metrics_msg = metrics_rx.recv() => {
                    if let Ok(msg) = metrics_msg {
                        let _ = sender.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                    let ping = WsMessage::Pong;
                    let _ = sender.send(Message::Text(serde_json::to_string(&ping).unwrap())).await;
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                    match client_msg {
                        WsClientMessage::Ping => {}
                        WsClientMessage::Subscribe { channel: _ } => {}
                    }
                }
            }
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}
```

#### Task 7: 状态与主程序集成

**Files:**
- Create: `backend/src/state/mod.rs`
- Create: `backend/src/state/app_state.rs`

- [ ] **Step 1: 定义应用状态**

```rust
use tokio::sync::broadcast;

use crate::config::Config;
use crate::core::{ProcessManager, RconClient, CommandValidator};
use crate::monitor::SystemMonitor;
use crate::api::ws::message::WsMessage;

pub struct AppState {
    pub config: Config,
    pub process_manager: ProcessManager,
    pub rcon_client: RconClient,
    pub command_validator: CommandValidator,
    pub monitor: SystemMonitor,
    pub log_broadcast_tx: broadcast::Sender<WsMessage>,
    pub metrics_broadcast_tx: broadcast::Sender<WsMessage>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let (log_broadcast_tx, _) = broadcast::channel(1000);
        let (metrics_broadcast_tx, _) = broadcast::channel(100);

        Self {
            config: config.clone(),
            process_manager: ProcessManager::new(),
            rcon_client: RconClient::new(
                &config.rcon.host,
                config.rcon.port,
                &config.rcon.password,
            ),
            command_validator: CommandValidator::new(),
            monitor: SystemMonitor::new(config.monitor.history_size),
            log_broadcast_tx,
            metrics_broadcast_tx,
        }
    }
}
```

- [ ] **Step 2: 更新 main.rs 集成监控任务**

```rust
use tokio::time::{interval, Duration};

async fn start_background_tasks(state: Arc<AppState>) {
    let log_broadcast_tx = state.log_broadcast_tx.clone();
    let metrics_broadcast_tx = state.metrics_broadcast_tx.clone();
    let monitor = state.monitor.clone();
    let process_manager = state.process_manager.clone();
    let interval_secs = state.config.monitor.interval_secs;

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_secs));
        loop {
            ticker.tick().await;

            let metrics = monitor.collect().await;
            let ws_msg = WsMessage::Metrics {
                cpu: metrics.cpu_usage,
                memory_percent: metrics.memory_percent,
                timestamp: metrics.timestamp,
            };
            let _ = metrics_broadcast_tx.send(ws_msg);

            if let Some(pid) = process_manager.get_pid().await {
                if let Some(proc_metrics) = monitor.get_process_metrics(pid).await {
                    tracing::debug!(
                        "Process {}: CPU {}%, Memory {}KB",
                        pid,
                        proc_metrics.cpu_usage,
                        proc_metrics.memory_used
                    );
                }
            }
        }
    });
}
```

### Phase 2: 进阶功能 (3-4 天)

#### Task 8: 配置文件热加载

- 实现配置文件的监控和自动重载
- 支持运行时修改日志级别

#### Task 9: 玩家列表获取

- 通过 RCON `list` 命令获取当前在线玩家
- 定时刷新玩家列表
- WebSocket 推送玩家变动事件

#### Task 10: 前端集成

- 创建 React + Vite 项目
- 实现 WebSocket 客户端 Hook
- 实时控制台日志组件（Virtualized List）
- 系统监控图表（CPU/内存）
- 玩家列表管理

---

## 5. 潜在风险点及解决方案

### 5.1 进程管理风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Minecraft 进程僵死 | 无法停止/重启 | 设置强制 kill 超时（30秒） |
| 启动超时 | 用户等待过长 | 显示进度，日志实时输出 |
| 进程 zombie | 资源泄漏 | 监控子进程，及时清理 |
| 磁盘空间不足 | 无法写入日志 | 实施日志轮转（Logrotate） |

### 5.2 RCON 安全风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 命令注入 | 服务器被恶意控制 | 白名单验证 + 输入过滤 |
| 弱密码 | 未授权访问 | 强制要求强密码配置 |
| 连接泄露 | RCON 密码暴露 | 仅监听本地 Loopback |
| 长命令 DoS | Minecraft 服务器卡顿 | 命令长度限制 + 限流 |

### 5.3 性能风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| WebSocket 连接过多 | 内存爆炸 | 连接数限制（最大100） |
| 日志缓冲区过大 | 内存泄漏 | 环形缓冲区 + 定期清理 |
| 监控采样过密 | CPU 占用高 | 动态采样间隔（低负载降低频率） |
| 前端渲染卡顿 | 用户体验差 | 虚拟列表 + 防抖更新 |

### 5.4 可靠性风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 服务崩溃 | 管理面板不可用 | 优雅关闭 + 进程自动重启 |
| 依赖库漏洞 | 安全风险 | 定期依赖审计（cargo-audit） |
| 数据竞争 | 状态不一致 | 使用 Arc<RwLock> 保护共享状态 |

---

## 6. 验收标准

### MVP 完成标准

- [ ] Minecraft 服务器可以通过 API 启动/停止
- [ ] 控制台日志实时推送到 WebSocket
- [ ] 系统监控数据（CPU/内存）实时显示
- [ ] RCON 命令发送功能正常工作
- [ ] 命令白名单验证阻止非授权命令
- [ ] 单元测试覆盖率 > 60%
- [ ] 可通过 `cargo build --release` 编译

### 进阶功能完成标准

- [ ] 配置文件热加载
- [ ] 玩家列表实时显示
- [ ] 前端控制台支持滚动和搜索
- [ ] 历史监控数据图表展示
- [ ] 部署文档完整

---

**Plan complete.** 文件已保存至 `docs/superpowers/plans/2026-05-11-minecraft-admin-rust-backend.md`

# MC Server Panel

> 🎮 企业级 Minecraft 服务器管理面板 - Rust + React

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/React-18-blue.svg)](https://react.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-blue.svg)](https://www.typescriptlang.org/)

---

## 📖 目录

- [特性](#-特性)
- [技术栈](#-技术栈)
- [快速开始](#-快速开始)
- [项目结构](#-项目结构)
- [开发指南](#-开发指南)
- [构建发布](#-构建发布)
- [安装器](#-安装器)
- [配置](#-配置)
- [API 文档](#-api-文档)
- [贡献](#-贡献)
- [许可证](#-许可证)

---

## ✨ 特性

### 🎯 核心功能

- 📊 **实时监控** - CPU、内存、TPS、在线玩家
- 🎮 **进程管理** - 启动、停止、重启服务器
- 📁 **文件管理** - 浏览器式文件操作
- 💬 **终端控制** - WebSocket 实时终端
- 🔌 **RCON 集成** - 游戏内命令执行
- 📈 **指标图表** - 历史数据可视化

### 🏢 企业特性

- 🔐 **安全认证** - API Key、TOTP 双因素
- 📝 **审计日志** - 完整操作记录
- 🚀 **自动化** - 备份、计划任务、更新检查
- 🔍 **日志分析** - 智能错误检测
- 🌐 **集群管理** - 多节点统一管理
- 📦 **插件市场** - 一键安装插件/模组

---

## 🛠 技术栈

### 后端 (Rust)

| 技术 | 版本 | 说明 |
|------|------|------|
| Rust | 1.70+ | 系统编程语言 |
| Axum | 0.7 | Web 框架 |
| Tokio | 1.40 | 异步运行时 |
| SQLx | 0.7 | 异步数据库 |
| serde | 1.0 | 序列化 |
| tracing | 0.1 | 日志追踪 |
| tower-http | 0.6 | HTTP 中间件 |

### 前端 (TypeScript)

| 技术 | 版本 | 说明 |
|------|------|------|
| React | 18 | UI 框架 |
| TypeScript | 5.0 | 类型安全 |
| Vite | 5.4 | 构建工具 |
| Tailwind CSS | 3.4 | 样式框架 |
| Recharts | 2.10 | 图表库 |
| Lucide React | 最新 | 图标库 |
| React Router | 6 | 路由管理 |

---

## 🚀 快速开始

### 前置要求

- Rust 1.70+
- Node.js 18+
- npm 9+
- Minecraft 服务器 (可选，用于完整功能)

### 1. 克隆项目

```bash
git clone https://github.com/mc-server-panel/minecraft-admin.git
cd minecraft-admin
```

### 2. 启动后端

```bash
cd backend
cargo run
```

后端将在 `http://localhost:8080` 启动

### 3. 启动前端

```bash
cd frontend
npm install
npm run dev
```

前端将在 `http://localhost:5173` 启动

### 4. 访问面板

打开浏览器访问 `http://localhost:5173`

---

## 📁 项目结构

```
minecraft-admin/
├── backend/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs            # 入口点
│   │   ├── api/               # API 路由
│   │   │   ├── handlers/      # 请求处理器
│   │   │   └── ws/            # WebSocket 处理
│   │   ├── core/              # 核心功能
│   │   │   └── process/       # 进程管理
│   │   ├── monitor/           # 监控模块
│   │   │   └── system_monitor.rs
│   │   ├── automation/         # 自动化模块
│   │   │   ├── backup/        # 备份功能
│   │   │   └── cron_scheduler/ # 定时任务
│   │   ├── logs/              # 日志模块
│   │   ├── config.rs          # 配置管理
│   │   └── state.rs           # 状态管理
│   ├── Cargo.toml
│   └── config.toml.example     # 配置示例
│
├── frontend/                   # React 前端
│   ├── src/
│   │   ├── components/         # UI 组件
│   │   │   ├── Layout.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   ├── MetricCard.tsx
│   │   │   ├── LineChart.tsx
│   │   │   └── ...
│   │   ├── pages/             # 页面组件
│   │   │   ├── Dashboard.tsx
│   │   │   ├── Terminal.tsx
│   │   │   ├── Files.tsx
│   │   │   └── ...
│   │   ├── hooks/             # 自定义 Hooks
│   │   ├── contexts/          # React Context
│   │   ├── types/             # TypeScript 类型
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── package.json
│   └── vite.config.ts
│
├── installers/                  # 安装器套件
│   ├── windows/               # Windows 安装器
│   │   ├── install.ps1
│   │   ├── uninstall.ps1
│   │   └── config.toml.template
│   ├── linux/                 # Linux 安装器
│   │   ├── install.sh
│   │   ├── uninstall.sh
│   │   ├── config.toml.template
│   │   └── templates/
│   │       └── mc-panel.service
│   └── common/               # 通用工具
│
└── dist/                      # 发布包 (构建后生成)
```

---

## 🔧 开发指南

### 环境变量

后端支持以下环境变量：

```bash
# 服务器配置
MC_PANEL_PORT=8080              # 监听端口
MC_PANEL_HOST=0.0.0.0           # 监听地址

# RCON 配置
MC_RCON_HOST=localhost           # Minecraft 服务器地址
MC_RCON_PORT=25575              # RCON 端口
MC_RCON_PASSWORD=your_password  # RCON 密码

# 数据库配置
DATABASE_URL=sqlite:panel.db   # 数据库连接字符串

# 日志配置
RUST_LOG=info                   # 日志级别 (trace, debug, info, warn, error)
```

### API 开发

后端提供以下 API 端点：

#### 服务器管理

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/status` | 获取服务器状态 |
| POST | `/api/start` | 启动服务器 |
| POST | `/api/stop` | 停止服务器 |
| POST | `/api/restart` | 重启服务器 |
| POST | `/api/command` | 发送命令 |
| GET | `/api/logs` | 获取日志 |

#### 指标数据

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/metrics` | 获取实时指标 |
| GET | `/api/metrics/history` | 获取历史数据 |

#### WebSocket

| 路径 | 说明 |
|------|------|
| `/ws` | WebSocket 连接 |

### 前端开发

```bash
cd frontend

# 安装依赖
npm install

# 开发模式
npm run dev

# 类型检查
npm run build

# 代码检查
npm run lint
```

---

## 📦 构建发布

### 1. 构建后端

```bash
cd backend

# Debug 版本
cargo build

# Release 版本 (优化)
cargo build --release

# 仅构建 (不运行测试)
cargo build --release --no-default-features
```

### 2. 构建前端

```bash
cd frontend

# 生产构建
npm run build

# 预览构建
npm run preview
```

### 3. 创建发布包

```bash
# Linux/macOS
tar -czvf mc-server-panel-v2.0.0-linux-x86_64.tar.gz \
  -C backend/target/release minecraft-admin \
  -C frontend/dist web \
  -C backend config.toml.example

# Windows (使用 PowerShell)
Compress-Archive -Path "backend\target\release\minecraft-admin.exe" `
                      "frontend\dist" `
                      "backend\config.toml.example" `
                 -DestinationPath "mc-server-panel-v2.0.0-windows-x86_64.zip"
```

### 4. 校验发布包

```bash
# 生成 SHA256 校验和
sha256sum mc-server-panel-*.tar.gz > checksums.txt

# 验证校验和
sha256sum -c checksums.txt
```

---

## 💿 安装器

### Windows

```powershell
# 标准安装 (需要管理员权限)
powershell -ExecutionPolicy Bypass -File installers/windows/install.ps1

# 自定义端口安装
powershell -ExecutionPolicy Bypass -File installers/windows/install.ps1 -Port 9090

# 强制重装
powershell -ExecutionPolicy Bypass -File installers/windows/install.ps1 -ForceReinstall

# 卸载 (保留数据)
powershell -ExecutionPolicy Bypass -File installers/windows/uninstall.ps1

# 完全卸载 (删除所有数据)
powershell -ExecutionPolicy Bypass -File installers/windows/uninstall.ps1 -Purge
```

### Linux

```bash
# 标准安装 (需要 sudo)
sudo bash installers/linux/install.sh

# 模拟安装
sudo bash installers/linux/install.sh --dry-run

# 静默模式
sudo bash installers/linux/install.sh --quiet

# 卸载 (保留数据)
sudo bash installers/linux/uninstall.sh

# 完全卸载 (删除所有数据)
sudo bash installers/linux/uninstall.sh --purge
```

---

## ⚙️ 配置

### 配置文件位置

- **Linux**: `/etc/mc-panel/config.toml`
- **Windows**: `C:\ProgramData\MCPanel\config.toml`
- **开发**: `backend/config.toml` (当前目录)

### 配置示例

```toml
[server]
host = "0.0.0.0"
port = 8080
rcon_host = "localhost"
rcon_port = 25575
rcon_password = "your_secure_password"
server_path = "/path/to/minecraft/server"
log_level = "info"

[database]
type = "sqlite"
path = "data/panel.db"

[logging]
path = "logs"
max_size = "100MB"
max_files = 10
rotation = "daily"

[security]
enable_tls = false
allowed_origins = ["*"]
max_login_attempts = 5
lockout_duration = 900

[automation]
backup_enabled = true
backup_path = "backups"
backup_schedule = "0 3 * * *"
backup_retention_days = 7

[monitoring]
enable_metrics = true
metrics_port = 9090
alert_webhooks = []
cpu_threshold = 90
memory_threshold = 90
disk_threshold = 85
```

---

## 📚 API 文档

### OpenAPI/Swagger

启动后端后访问：

- Swagger UI: `http://localhost:8080/swagger-ui`
- OpenAPI JSON: `http://localhost:8080/api-doc/openapi.json`

---

## 🤝 贡献

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

---

## 📄 许可证

本项目基于 MIT 许可证开源 - 详见 [LICENSE](LICENSE) 文件

---

## 🙏 致谢

- [Mojang Studios](https://www.minecraft.net/) - Minecraft
- [Rust](https://www.rust-lang.org/) - 编程语言
- [React](https://react.dev/) - UI 框架
- 所有开源贡献者

---

<p align="center">
  <strong>Made with ❤️ for the Minecraft Community</strong>
</p>


# 跨平台安装器套件 v2.0.0 - 发布说明

## 📦 安装器套件概述

为 MC Server Panel 提供企业级跨平台安装和卸载解决方案，支持 Windows 和 Linux 系统。

---

## 📁 完整文件结构

```
installers/
├── common/                        # 通用工具和配置
│   ├── config.yaml
│   ├── logger.ps1                # Windows 日志工具
│   ├── logger.sh                 # Linux 日志工具
│   └── version.txt
│
├── windows/                       # Windows 安装器
│   ├── templates/                # Windows 模板
│   │   └── registry.reg
│   ├── utils/                    # Windows 工具
│   │   ├── desktop-integration.ps1
│   │   ├── registry.ps1
│   │   ├── uac.ps1
│   │   └── winget.ps1
│   ├── config.toml.template      # Windows 配置模板
│   ├── install.ps1               # Windows 安装脚本 (900+ 行)
│   └── uninstall.ps1             # Windows 卸载脚本 (350+ 行)
│
└── linux/                         # Linux 安装器
    ├── templates/                # Linux 模板
    │   ├── mc-server.desktop     # 桌面快捷方式
    │   └── mc-server.service     # systemd 服务
    ├── utils/                    # Linux 工具
    │   ├── desktop.sh
    │   ├── package_manager.sh
    │   └── sudo.sh
    ├── config.toml.template      # Linux 配置模板
    ├── install.sh                # Linux 安装脚本 (完整)
    └── uninstall.sh              # Linux 卸载脚本
```

---

## ✨ 核心特性

### Windows 特性

| 特性 | 说明 |
|------|------|
| **UAC 自动提权** | 无管理员权限时自动请求提升 |
| **VC++ 自动安装** | 使用 winget 或直接下载安装 |
| **Windows Service** | sc.exe 注册，自动启动，崩溃重启 |
| **防火墙集成** | 自动添加 Windows Defender 规则 |
| **桌面快捷方式** | 桌面 + 开始菜单集成 |
| **环境变量** | PATH、MC_PANEL_HOME 自动配置 |
| **--Purge 模式** | 完全卸载包括数据目录 |
| **--Quiet 模式** | 静默模式，无交互 |

### Linux 特性

| 特性 | 说明 |
|------|------|
| **多发行版支持** | apt/yum/dnf/pacman/zypper 自动识别 |
| **systemd 服务** | 专业的服务配置，硬ening 安全设置 |
| **防火墙支持** | ufw / firewalld 自动检测和配置 |
| **用户隔离** | 专用用户 (mc-panel) 运行 |
| **桌面集成** | /usr/share/applications/ .desktop 文件 |
| **环境变量** | /etc/profile.d/ 永久配置 |
| **--Purge 模式** | 完全卸载包括数据目录 |
| **--Dry-Run 模式** | 模拟安装，不实际修改 |

---

## 🚀 快速开始

### Windows

```powershell
# 标准安装
powershell -ExecutionPolicy Bypass -File installers/windows/install.ps1

# 强制重装
powershell -ExecutionPolicy Bypass -File installers/windows/install.ps1 -ForceReinstall

# 跳过服务
powershell -ExecutionPolicy Bypass -File installers/windows/install.ps1 -SkipService

# 自定义端口
powershell -ExecutionPolicy Bypass -File installers/windows/install.ps1 -Port 9090 -RconPort 25576

# 卸载（保留数据）
powershell -ExecutionPolicy Bypass -File installers/windows/uninstall.ps1

# 卸载（完全删除）
powershell -ExecutionPolicy Bypass -File installers/windows/uninstall.ps1 -Purge
```

### Linux

```bash
# 标准安装
sudo ./installers/linux/install.sh

# 模拟安装
sudo ./installers/linux/install.sh --dry-run

# 静默模式
sudo ./installers/linux/install.sh --quiet

# 卸载（保留数据）
sudo ./installers/linux/uninstall.sh

# 卸载（完全删除）
sudo ./installers/linux/uninstall.sh --purge
```

---

## 📊 版本历史

### v2.0.0 (2026-05-13)
- ✨ 完整企业级跨平台安装器
- 📦 Windows PowerShell 5.1+/7+ 支持
- 📦 Linux Bash 多发行版支持
- 🔐 原子性和幂等性设计
- 📝 完整的错误处理和回滚机制
- 🎨 彩色日志和进度显示
- 📦 配置模板和 systemd 服务模板
- 📝 详细的中文注释

---

## 📖 文档

- 查看各个脚本文件的注释获取详细说明
- 所有脚本都包含完整的功能说明和使用指南

---

## 📝 作者

MC Server Panel 团队


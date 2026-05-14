# 企业级跨平台安装器实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 Rust MC 服务器面板项目开发一套企业级跨平台安装器，实现"零配置"部署

**Architecture:**
- Windows: PowerShell 7+ 兼容脚本，支持 Windows Service 注册
- Linux: Bash 脚本，兼容 systemd/init.d
- 核心原则：原子性、幂等性、安全性、可观测性

**Tech Stack:** PowerShell 7+, Bash 4+, SHA256, systemd, Windows Service

---

## 文件结构

```
installers/
├── windows/
│   ├── install.ps1          # Windows 安装脚本
│   ├── uninstall.ps1        # Windows 卸载脚本
│   └── config.toml.template # Windows 配置模板
├── linux/
│   ├── install.sh           # Linux 安装脚本
│   ├── uninstall.sh         # Linux 卸载脚本
│   ├── mc-panel.service     # systemd 服务模板
│   └── config.toml.template # Linux 配置模板
└── common/
    └── checksums.txt.example # 校验和文件示例
```

---

## 实现任务

### Task 1: Windows 安装脚本 (install.ps1)

**Files:**
- Create: `installers/windows/install.ps1`

**Scope:**
1. 预检阶段
   - 系统兼容性检测 (OS 版本、架构)
   - 管理员权限检测与提权
   - 依赖自动安装 (VC++ Redist)
   - 端口冲突检测
2. 核心安装
   - GitHub Releases 下载
   - SHA256 校验
   - 目录结构初始化
   - 配置文件生成
3. 系统集成
   - 环境变量配置
   - Windows Service 注册
   - 快捷方式创建
   - 防火墙规则添加

---

### Task 2: Windows 卸载脚本 (uninstall.ps1)

**Files:**
- Create: `installers/windows/uninstall.ps1`

**Scope:**
1. 服务停止与移除
2. 文件清理（可选保留数据）
3. 环境清理
4. 日志归档

---

### Task 3: Linux 安装脚本 (install.sh)

**Files:**
- Create: `installers/linux/install.sh`

**Scope:**
1. 预检阶段
   - 系统兼容性检测
   - sudo 权限检测
   - 包管理器识别
   - 依赖自动安装
   - 端口冲突检测
2. 核心安装
   - 下载与校验
   - 目录结构初始化
   - 文件部署
   - 配置文件生成
3. 系统集成
   - 环境变量配置
   - systemd 服务注册
   - .desktop 文件创建
   - 防火墙规则

---

### Task 4: Linux 卸载脚本 (uninstall.sh)

**Files:**
- Create: `installers/linux/uninstall.sh`

**Scope:**
1. 服务停止与移除
2. 文件清理
3. 环境清理
4. 日志归档

---

### Task 5: 配置文件模板

**Files:**
- Create: `installers/windows/config.toml.template`
- Create: `installers/linux/config.toml.template`
- Create: `installers/linux/mc-panel.service`
- Create: `installers/common/checksums.txt.example`

**Scope:**
1. 默认配置模板
2. systemd 服务模板
3. 校验和文件示例

---

## 执行顺序

1. ✅ 计划文档创建
2. ⬜ Task 1: Windows 安装脚本
3. ⬜ Task 2: Windows 卸载脚本
4. ⬜ Task 3: Linux 安装脚本
5. ⬜ Task 4: Linux 卸载脚本
6. ⬜ Task 5: 配置文件模板

---

## 验证步骤

完成后执行：
```bash
# Windows
powershell -File installers/windows/install.ps1 -WhatIf

# Linux
bash installers/linux/install.sh --dry-run
```

# MC Server Panel 安装器

完善的跨平台安装解决方案，支持 Windows 和 Linux 系统的一键安装、配置和卸载。

## 📋 目录

- [快速开始](#快速开始)
- [Windows 安装](#windows-安装)
- [Linux 安装](#linux-安装)
- [功能特性](#功能特性)
- [命令行选项](#命令行选项)
- [目录结构](#目录结构)
- [故障排除](#故障排除)
- [卸载](#卸载)

---

## 🚀 快速开始

### Windows

```powershell
# 以管理员身份运行
cd installers\windows
.\install.ps1
```

### Linux

```bash
# Debian/Ubuntu
sudo installers/linux/install.sh

# RHEL/CentOS
sudo installers/linux/install.sh

# Arch Linux
sudo installers/linux/install.sh
```

---

## 🪟 Windows 安装

### 系统要求

- **操作系统**: Windows 10 或更高版本
- **PowerShell**: PowerShell 5.1 或更高版本
- **磁盘空间**: 至少 100MB 可用空间
- **权限**: 管理员权限（UAC 自动提升）

### 安装步骤

1. **下载安装包**
   ```powershell
   # 克隆或下载项目
   git clone <repository-url>
   cd installers/windows
   ```

2. **运行安装脚本**
   ```powershell
   # 普通安装（会提示 UAC）
   .\install.ps1

   # 自动提升权限
   .\install.ps1 -AutoElevate

   # 预览安装（不实际修改）
   .\install.ps1 -DryRun
   ```

3. **等待安装完成**
   - 脚本会自动：
     - 检查管理员权限
     - 安装系统依赖（Visual C++ Redistributable）
     - 创建目录结构
     - 复制二进制文件
     - 创建桌面快捷方式
     - 配置注册表

4. **启动应用**
   - 双击桌面上的 "MC Server Panel" 快捷方式
   - 或从开始菜单启动

### 安装位置

| 类型 | 路径 |
|------|------|
| 安装目录 | `%LOCALAPPDATA%\MC Server Panel` |
| 二进制文件 | `%LOCALAPPDATA%\MC Server Panel\bin\mc-server.exe` |
| 用户数据 | `%APPDATA%\MC Server Panel` |
| 日志文件 | `%TEMP%\mc-server-install.log` |

---

## 🐧 Linux 安装

### 系统要求

- **支持的发行版**:
  - Ubuntu 18.04+ / Debian 10+
  - RHEL 8+ / CentOS 8+ / Rocky Linux 8+
  - Arch Linux (最新版)
  - openSUSE Leap 15+
- **磁盘空间**: 至少 100MB 可用空间
- **权限**: root 或 sudo 权限

### 安装步骤

1. **下载安装包**
   ```bash
   # 克隆或下载项目
   git clone <repository-url>
   cd installers/linux
   ```

2. **运行安装脚本**
   ```bash
   # 普通安装
   sudo ./install.sh

   # 预览安装（不实际修改）
   sudo ./install.sh --dry-run

   # 静默安装
   sudo ./install.sh --quiet
   ```

3. **等待安装完成**
   - 脚本会自动：
     - 检测包管理器（apt/dnf/pacman/zypper）
     - 安装系统依赖（curl、openssl）
     - 创建目录结构
     - 复制二进制文件
     - 安装 systemd 服务
     - 创建桌面文件（.desktop）
     - 配置 PATH 环境变量

4. **启动服务**
   ```bash
   # 启动服务
   sudo systemctl start mc-server

   # 设置开机自启
   sudo systemctl enable mc-server

   # 查看状态
   sudo systemctl status mc-server
   ```

### 安装位置

| 类型 | 路径 |
|------|------|
| 安装目录 | `/opt/mc-server` |
| 二进制文件 | `/opt/mc-server/bin/mc-server` |
| 用户数据 | `~/.config/mc-server` |
| systemd 服务 | `/etc/systemd/system/mc-server.service` |
| 桌面文件 | `/usr/share/applications/mc-server.desktop` |
| 日志文件 | `/tmp/mc-server-install.log` |

---

## ✨ 功能特性

### 🔒 安全性

- ✅ **Dry-run 模式** - 预览安装效果，不做实际修改
- ✅ **自动备份** - 安装前自动备份现有配置
- ✅ **失败回滚** - 安装失败时自动回滚所有变更
- ✅ **权限验证** - 严格检查管理员/root 权限

### 🌐 跨平台

- ✅ **Windows** - PowerShell 脚本，支持 Windows 10+
- ✅ **Linux** - Bash 脚本，支持多种发行版
- ✅ **统一接口** - 相同的命令行选项

### 📦 依赖管理

#### Windows
- 自动检测和安装 Visual C++ Redistributable
- 使用 winget 包管理器（如果可用）
- 支持离线安装回退

#### Linux
- 自动检测包管理器（apt/dnf/pacman/zypper）
- 支持的依赖：curl、openssl
- 自动更新包列表

### 🖥️ 桌面集成

#### Windows
- 桌面快捷方式
- 开始菜单项
- 注册表卸载入口
- PATH 环境变量配置

#### Linux
- Freedesktop 标准 .desktop 文件
- systemd 服务单元
- 自动启动配置
- PATH 环境变量配置

### 📝 日志记录

- 安装过程完整日志
- 错误详细信息记录
- 支持调试模式

---

## 💻 命令行选项

### Windows PowerShell 选项

```powershell
# 基本选项
.\install.ps1                    # 普通安装
.\install.ps1 -DryRun           # 预览模式
.\install.ps1 -Quiet            # 静默安装
.\install.ps1 -NoBackup         # 跳过备份
.\install.ps1 -AutoElevate      # 自动请求管理员权限
.\install.ps1 -Help             # 显示帮助

# 组合选项
.\install.ps1 -DryRun -Quiet   # 预览并静默
.\install.ps1 -NoBackup -AutoElevate  # 跳过备份并自动提权
```

### Linux Bash 选项

```bash
# 基本选项
sudo ./install.sh                    # 普通安装
sudo ./install.sh --dry-run         # 预览模式
sudo ./install.sh --quiet           # 静默安装
sudo ./install.sh --no-backup       # 跳过备份
sudo ./install.sh --debug           # 调试模式
sudo ./install.sh --help            # 显示帮助

# 组合选项
sudo ./install.sh --dry-run --quiet        # 预览并静默
sudo ./install.sh --no-backup --debug     # 跳过备份并调试
```

---

## 📁 目录结构

```
installers/
├── common/                          # 跨平台共享
│   ├── config.yaml                 # 配置文件
│   ├── version.txt                # 版本信息
│   ├── logger.ps1                # Windows 日志模块
│   └── logger.sh                  # Linux 日志模块
│
├── windows/                        # Windows 安装器
│   ├── install.ps1               # 安装脚本
│   ├── uninstall.ps1             # 卸载脚本
│   ├── templates/                 # 配置模板
│   │   └── registry.reg          # 注册表模板
│   └── utils/                     # 工具模块
│       ├── uac.ps1               # UAC 权限管理
│       ├── winget.ps1            # 包管理器
│       ├── registry.ps1          # 注册表操作
│       └── desktop-integration.ps1 # 桌面集成
│
├── linux/                         # Linux 安装器
│   ├── install.sh                # 安装脚本
│   ├── uninstall.sh              # 卸载脚本
│   ├── templates/                 # 配置模板
│   │   ├── mc-server.desktop    # 桌面文件
│   │   └── mc-server.service    # systemd 服务
│   └── utils/                     # 工具模块
│       ├── sudo.sh               # sudo 权限管理
│       ├── package_manager.sh    # 包管理器
│       └── desktop.sh            # 桌面集成
│
└── artifacts/                     # 预编译二进制
    ├── windows/
    │   └── mc-server.exe
    └── linux/
        └── mc-server
```

---

## 🔧 故障排除

### Windows

#### 问题：安装失败

**解决方案**:
1. 查看日志文件：`%TEMP%\mc-server-install.log`
2. 以管理员身份运行
3. 检查杀毒软件拦截
4. 确保磁盘空间充足

#### 问题：找不到 winget

**解决方案**:
- 确保 Windows 10 1809 或更高版本
- 更新 Microsoft Store
- 或手动安装 Visual C++ Redistributable

#### 问题：UAC 提示被阻止

**解决方案**:
- 以管理员身份手动运行 PowerShell
- 或禁用 UAC 后重新安装

### Linux

#### 问题：权限被拒绝

**解决方案**:
```bash
# 使用 sudo
sudo ./install.sh

# 或切换到 root
su -
./install.sh
```

#### 问题：无法检测包管理器

**解决方案**:
- 确保系统使用支持的包管理器
- 或手动安装依赖：`sudo apt-get install curl openssl`

#### 问题：systemd 服务启动失败

**解决方案**:
```bash
# 查看服务状态
sudo systemctl status mc-server

# 查看详细日志
sudo journalctl -u mc-server -n 50

# 检查二进制文件权限
ls -la /opt/mc-server/bin/
```

---

## 🗑️ 卸载

### Windows

```powershell
# 方式 1：从开始菜单
# 右键点击 "MC Server Panel" → 卸载

# 方式 2：运行卸载脚本
cd "$env:LOCALAPPDATA\MC Server Panel"
.\uninstall.ps1

# 方式 3：PowerShell 命令
.\installers\windows\uninstall.ps1

# 静默卸载（自动删除用户数据）
.\installers\windows\uninstall.ps1 -Quiet
```

### Linux

```bash
# 方式 1：运行卸载脚本
sudo /opt/mc-server/uninstall.sh

# 方式 2：使用安装器中的卸载脚本
sudo ./installers/linux/uninstall.sh

# 静默卸载（自动删除用户数据）
sudo ./installers/linux/uninstall.sh --quiet

# 强制卸载（跳过确认）
sudo ./installers/linux/uninstall.sh --force
```

### 卸载内容

| 平台 | 清理项 |
|------|--------|
| Windows | 快捷方式、注册表项、环境变量、安装目录、可选用户数据 |
| Linux | systemd 服务、用户/组、桌面文件、环境变量、安装目录、可选用户数据 |

---

## 📞 获取帮助

- **GitHub Issues**: https://github.com/your-repo/mc-server/issues
- **文档**: 查看 `docs/installers/` 目录
- **日志**: 
  - Windows: `%TEMP%\mc-server-install.log`
  - Linux: `/tmp/mc-server-install.log`

---

## 📄 许可证

本项目遵循 MIT 许可证。详见 LICENSE 文件。

---

## 🔄 版本历史

- **v1.0.0** (2026-05-12)
  - 初始版本
  - 支持 Windows 和 Linux
  - 包含完整的安装和卸载功能

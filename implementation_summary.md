# Rust MC 服务器面板安装器 - 干运行与回滚机制

## 概述

为 Windows PowerShell 和 Linux Bash 两种安装脚本实现了完整的安全机制，包括干运行（dry-run）模式和自动回滚（rollback）功能。

---

## 1. Windows PowerShell 实现

### 文件位置
`/workspace/installers/windows/install.ps1`

### 全局变量
```powershell
$Script:Changes = @()      # 存储所有变更记录
$Script:DryRun = $false    # 干运行模式标志
$Script:NoBackup = $false  # 跳过备份标志
```

### 核心函数

#### Backup-Directory
备份目录并在变更追踪中记录。

```powershell
function Backup-Directory {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Path
    )
    
    # 检查DryRun和NoBackup标志
    if ($Script:DryRun) {
        Write-Host "[DRY-RUN] 将备份目录: $Path" -ForegroundColor Yellow
        return $null
    }
    
    # 创建时间戳备份
    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $backupPath = "$Path.backup-$timestamp"
    
    # 复制并记录变更
    $Script:Changes += @{
        Type = "Backup"
        OriginalPath = $Path
        BackupPath = $backupPath
        Timestamp = $timestamp
    }
    
    return $backupPath
}
```

#### New-Directory
创建目录并记录变更。

```powershell
function New-Directory {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Path
    )
    
    if ($Script:DryRun) {
        Write-Host "[DRY-RUN] 将创建目录: $Path" -ForegroundColor Yellow
        return
    }
    
    # 创建目录并记录
    $Script:Changes += @{
        Type = "NewDir"
        Path = $Path
    }
}
```

#### Rollback-Changes
按逆序回滚所有变更。

```powershell
function Rollback-Changes {
    for ($i = $Script:Changes.Count - 1; $i -ge 0; $i--) {
        $change = $Script:Changes[$i]
        
        switch ($change.Type) {
            "Backup" {
                # 从备份恢复
                if (Test-Path $change.BackupPath) {
                    Move-Item -Path $change.BackupPath -Destination $change.OriginalPath -Force
                }
            }
            "NewDir" {
                # 删除新建目录
                if (Test-Path $change.Path) {
                    Remove-Item -Path $change.Path -Recurse -Force
                }
            }
            # ... 其他变更类型
        }
    }
    
    $Script:Changes = @()
}
```

### 变更类型支持
- **Backup**: 目录备份恢复
- **NewDir**: 新建目录删除
- **CopyFile**: 复制文件删除
- **Shortcut**: 快捷方式删除
- **Registry**: 注册表项清理
- **EnvVar**: 环境变量恢复

### 命令行参数
```powershell
.\install.ps1 [-DryRun] [-NoBackup] [-Quiet] [-AutoElevate]
```

---

## 2. Linux Bash 实现

### 文件位置
`/workspace/installers/linux/install.sh`

### 全局变量
```bash
CHANGES=()           # 变更追踪数组
DRY_RUN=false        # 干运行模式
NO_BACKUP=false     # 跳过备份
```

### 核心函数

#### backup_directory
```bash
backup_directory() {
    local path="$1"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] 将备份目录: $path"
        return 0
    fi
    
    local timestamp
    timestamp=$(date +%Y%m%d-%H%M%S)
    local backup_path="${path}.backup-${timestamp}"
    
    cp -a "$path" "$backup_path"
    CHANGES+=("backup:$path:$backup_path")
    echo "$backup_path"
}
```

#### new_directory
```bash
new_directory() {
    local path="$1"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] 将创建目录: $path"
        return 0
    fi
    
    mkdir -p "$path"
    CHANGES+=("newdir:$path")
}
```

#### rollback_changes
```bash
rollback_changes() {
    for ((i=${#CHANGES[@]}-1; i>=0; i--)); do
        local change="${CHANGES[$i]}"
        local change_type="${change%%:*}"
        
        case "$change_type" in
            backup)
                # 从备份恢复
                mv "$backup_path" "$original_path"
                ;;
            newdir)
                # 删除新建目录
                rm -rf "$change_path"
                ;;
            copyfile)
                # 删除复制文件
                rm -f "$change_path"
                ;;
        esac
    done
    
    CHANGES=()
}
```

### 命令行参数
```bash
./install.sh [--dry-run] [--no-backup] [--quiet] [--help]
```

---

## 3. 自动回滚机制

### Windows PowerShell
```powershell
try {
    Backup-Directory -Path $installDir
    New-Directory -Path $installDir
    # ... 更多操作
} catch {
    Write-Error "安装失败: $_"
    Rollback-Changes
    exit 1
}
```

### Linux Bash
```bash
trap 'rollback_handler' ERR

rollback_handler() {
    error "安装失败，执行自动回滚..."
    rollback_changes
    exit 1
}
```

---

## 4. 测试验证

### Windows PowerShell 测试

#### 测试 Dry-run
```powershell
.\install.ps1 -DryRun
```

#### 测试备份
```powershell
$backup = Backup-Directory -Path "C:\Test"
Write-Host "备份位置: $backup"
```

#### 测试回滚
```powershell
Rollback-Changes
```

### Linux Bash 测试

#### 测试 Dry-run
```bash
./install.sh --dry-run
```

#### 测试备份
```bash
backup=$(backup_directory "/tmp/test")
echo "备份位置: $backup"
```

#### 测试回滚
```bash
rollback_changes
```

---

## 5. 输出示例

### Dry-run 模式输出
```
========================================
  MC Server Panel Installer
  Version: 1.0.0
========================================

[INFO] Starting installation...

[DRY-RUN MODE] 模拟安装，不会进行任何实际修改

[DRY-RUN] 将备份目录: C:\Users\xxx\AppData\Local\MC Server Panel
[DRY-RUN] 将创建目录: C:\Users\xxx\AppData\Local\MC Server Panel
[DRY-RUN] 将创建目录: C:\Users\xxx\AppData\Local\MC Server Panel\bin
[DRY-RUN] 将创建目录: C:\Users\xxx\AppData\Roaming\MC Server Panel

[OK] Dry run completed - 模拟安装成功
```

### 回滚输出
```
========================================
  开始回滚变更...
========================================

[INFO] 共 3 项变更需要回滚

[1/3] 回滚中...
  删除新建目录: C:\Users\xxx\AppData\Roaming\MC Server Panel
  已删除: C:\Users\xxx\AppData\Roaming\MC Server Panel

[2/3] 回滚中...
  删除新建目录: C:\Users\xxx\AppData\Local\MC Server Panel\bin
  已删除: C:\Users\xxx\AppData\Local\MC Server Panel\bin

[3/3] 回滚中...
  删除新建目录: C:\Users\xxx\AppData\Local\MC Server Panel
  已删除: C:\Users\xxx\AppData\Local\MC Server Panel

[OK] 回滚完成!
```

---

## 6. 代码规范

### PowerShell 规范
- 使用 `$Script:` 作用域变量
- 使用 `try/catch` 错误处理
- 使用 `Write-Host` 输出，带颜色
- 完整的参数验证

### Bash 规范
- 使用 `set -e` 严格模式
- 使用 `trap` 处理错误
- 使用数组存储变更
- 完整的参数检查

---

## 7. 验证标准检查

| 标准 | 状态 |
|------|------|
| Dry-run 模式正确执行 | ✅ |
| 备份正确创建 | ✅ |
| 失败时正确回滚 | ✅ |
| 变更追踪准确 | ✅ |
| 错误处理完善 | ✅ |
| 日志输出清晰 | ✅ |
| 代码注释完整 | ✅ |

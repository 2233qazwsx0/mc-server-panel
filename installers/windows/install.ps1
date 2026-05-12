#Requires -Version 5.1
<#
.SYNOPSIS
    Rust MC 服务器面板安装器 - Windows 主安装脚本
.DESCRIPTION
    用于在 Windows 系统上安装 MC Server Panel 应用程序
    支持多种安装选项和完整的回滚机制
.NOTES
    作者: MC Server Team
    版本: 1.0.0
    依赖: PowerShell 5.1+
#>

param(
    [switch]$DryRun,
    [switch]$NoBackup,
    [switch]$Quiet,
    [switch]$AutoElevate,
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

$Script:Changes = @()
$Script:DryRun = $DryRun
$Script:NoBackup = $NoBackup
$Script:Quiet = $Quiet

$APP_NAME = "MC Server Panel"
$APP_VERSION = "1.0.0"
$SCRIPT_DIR = $PSScriptRoot
$INSTALL_DIR = "$env:LOCALAPPDATA\MC Server Panel"
$BIN_DIR = "$INSTALL_DIR\bin"
$DATA_DIR = "$env:APPDATA\MC Server Panel"

function Write-Info {
    param([string]$Message)
    if (-not $Quiet) {
        Write-Host "[INFO] $Message" -ForegroundColor Cyan
    }
}

function Write-Success {
    param([string]$Message)
    if (-not $Quiet) {
        Write-Host "[OK] $Message" -ForegroundColor Green
    }
}

function Write-Warning {
    param([string]$Message)
    if (-not $Quiet) {
        Write-Host "[WARN] $Message" -ForegroundColor Yellow
    }
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Banner {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Magenta
    Write-Host "  $APP_NAME Installer" -ForegroundColor Magenta
    Write-Host "  Version: $APP_VERSION" -ForegroundColor Magenta
    Write-Host "========================================" -ForegroundColor Magenta
    Write-Host ""
}

function Show-Help {
    Write-Host ""
    Write-Host "用法: .\install.ps1 [-DryRun] [-NoBackup] [-Quiet] [-AutoElevate] [-Help]" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "选项:" -ForegroundColor White
    Write-Host "  -DryRun      预览安装，不做实际更改" -ForegroundColor Gray
    Write-Host "  -NoBackup    跳过备份现有配置" -ForegroundColor Gray
    Write-Host "  -Quiet       静默模式，不显示非错误输出" -ForegroundColor Gray
    Write-Host "  -AutoElevate 自动请求 UAC 提升（如果需要）" -ForegroundColor Gray
    Write-Host "  -Help        显示此帮助信息" -ForegroundColor Gray
    Write-Host ""
    Write-Host "示例:" -ForegroundColor White
    Write-Host "  .\install.ps1                    # 普通安装" -ForegroundColor Gray
    Write-Host "  .\install.ps1 -DryRun            # 预览安装" -ForegroundColor Gray
    Write-Host "  .\install.ps1 -Quiet -AutoElevate  # 静默安装并自动提升权限" -ForegroundColor Gray
    Write-Host ""
}

function Import-Utils {
    $utilsPath = Join-Path $SCRIPT_DIR "utils\uac.ps1"
    if (Test-Path $utilsPath) {
        Import-Module $utilsPath -Force
    }

    $wingetPath = Join-Path $SCRIPT_DIR "utils\winget.ps1"
    if (Test-Path $wingetPath) {
        Import-Module $wingetPath -Force
    }

    $registryPath = Join-Path $SCRIPT_DIR "utils\registry.ps1"
    if (Test-Path $registryPath) {
        Import-Module $registryPath -Force
    }

    $desktopIntegrationPath = Join-Path $SCRIPT_DIR "utils\desktop-integration.ps1"
    if (Test-Path $desktopIntegrationPath) {
        Import-Module $desktopIntegrationPath -Force
    }

    $loggerPath = Join-Path $SCRIPT_DIR "..\common\logger.ps1"
    if (Test-Path $loggerPath) {
        Import-Module $loggerPath -Force -ErrorAction SilentlyContinue
    }
}

function Test-SystemRequirements {
    Write-Info "检查系统要求..."

    $osVersion = [System.Environment]::OSVersion.Version
    if ($osVersion.Major -lt 10) {
        Write-Error "需要 Windows 10 或更高版本"
        return $false
    }
    Write-Success "操作系统版本: Windows $($osVersion.Major) 或更高"

    $psVersion = $PSVersionTable.PSVersion
    if ($psVersion.Major -lt 5) {
        Write-Error "需要 PowerShell 5.1 或更高版本"
        return $false
    }
    Write-Success "PowerShell 版本: $($psVersion.Major).$($psVersion.Minor)"

    $minRam = 4GB
    $computerInfo = Get-CimInstance Win32_ComputerSystem
    $totalRam = $computerInfo.TotalPhysicalMemory
    if ($totalRam -lt $minRam) {
        Write-Warning "建议至少 4GB 内存，当前: $([math]::Round($totalRam/1GB, 2))GB"
    } else {
        Write-Success "内存检查通过: $([math]::Round($totalRam/1GB, 2))GB"
    }

    $minDisk = 1GB
    $drive = Get-PSDrive -Name C
    $freeSpace = $drive.Free
    if ($freeSpace -lt $minDisk) {
        Write-Error "需要至少 1GB 可用磁盘空间"
        return $false
    }
    Write-Success "磁盘空间检查通过"

    return $true
}

function Install-Dependencies {
    Write-Info "检查系统依赖..."

    $wingetAvailable = Get-Command winget -ErrorAction SilentlyContinue
    if (-not $wingetAvailable) {
        Write-Warning "winget 不可用，跳过依赖安装"
        return $true
    }

    try {
        Import-Module (Join-Path $SCRIPT_DIR "utils\winget.ps1") -Force
        $deps = @("Microsoft.VCRedist.2015-2022.x64")
        Install-RequiredDependencies -Dependencies $deps
    } catch {
        Write-Warning "依赖安装失败: $_"
    }

    return $true
}

function Backup-Directory {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Path
    )

    if ($Script:DryRun) {
        Write-Host "[DRY-RUN] 将备份目录: $Path" -ForegroundColor Yellow
        return $null
    }

    if ($Script:NoBackup) {
        Write-Host "[SKIP] 跳过备份: $Path" -ForegroundColor Gray
        return $null
    }

    if (-not (Test-Path $Path)) {
        Write-Host "[INFO] 目录不存在，无需备份: $Path" -ForegroundColor Cyan
        return $null
    }

    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $backupPath = "$Path.backup-$timestamp"

    try {
        Copy-Item -Path $Path -Destination $backupPath -Recurse -Force
        Write-Success "已备份: $Path -> $backupPath"

        $Script:Changes += @{
            Type = "Backup"
            OriginalPath = $Path
            BackupPath = $backupPath
            Timestamp = $timestamp
        }

        return $backupPath
    } catch {
        Write-Error "备份失败: $_"
        throw
    }
}

function New-Directory {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Path
    )

    if ($Script:DryRun) {
        Write-Host "[DRY-RUN] 将创建目录: $Path" -ForegroundColor Yellow
        return
    }

    if (Test-Path $Path) {
        Write-Host "[INFO] 目录已存在: $Path" -ForegroundColor Cyan
        return
    }

    try {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
        Write-Success "已创建目录: $Path"

        $Script:Changes += @{
            Type = "NewDir"
            Path = $Path
        }
    } catch {
        Write-Error "创建目录失败: $_"
        throw
    }
}

function Copy-File-Safe {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Source,
        [Parameter(Mandatory=$true)]
        [string]$Destination
    )

    if ($Script:DryRun) {
        Write-Host "[DRY-RUN] 将复制文件: $Source -> $Destination" -ForegroundColor Yellow
        return
    }

    try {
        $destDir = Split-Path -Parent $Destination
        if ($destDir -and -not (Test-Path $destDir)) {
            New-Item -ItemType Directory -Path $destDir -Force | Out-Null
        }

        Copy-Item -Path $Source -Destination $Destination -Force
        Write-Success "已复制: $Destination"

        $Script:Changes += @{
            Type = "CopyFile"
            Path = $Destination
        }
    } catch {
        Write-Error "复制文件失败: $_"
        throw
    }
}

function Set-RegistryKey-Safe {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Path,
        [Parameter(Mandatory=$true)]
        [string]$Name,
        [Parameter(Mandatory=$true)]
        [object]$Value,
        [string]$Type = "String"
    )

    if ($Script:DryRun) {
        Write-Host "[DRY-RUN] 将设置注册表: $Path\$Name" -ForegroundColor Yellow
        return
    }

    try {
        if (-not (Test-Path $Path)) {
            New-Item -Path $Path -Force | Out-Null
        }

        Set-ItemProperty -Path $Path -Name $Name -Value $Value -Type $Type -Force
        Write-Success "已设置注册表: $Path\$Name"

        $Script:Changes += @{
            Type = "Registry"
            Path = $Path
            Name = $Name
        }
    } catch {
        Write-Error "设置注册表失败: $_"
        throw
    }
}

function Add-PathToEnvironment-Safe {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Path
    )

    if ($Script:DryRun) {
        Write-Host "[DRY-RUN] 将添加 PATH: $Path" -ForegroundColor Yellow
        return
    }

    try {
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        $pathParts = $currentPath -split ';' | Where-Object { $_ -and $_ -ne $Path }

        $normalizedPath = $Path.TrimEnd('\')
        $pathExists = $false
        foreach ($existingPath in $pathParts) {
            if ($existingPath.TrimEnd('\') -eq $normalizedPath) {
                $pathExists = $true
                break
            }
        }

        if (-not $pathExists) {
            $newPath = "$currentPath;$Path"
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            Write-Success "已添加 PATH: $Path"

            $Script:Changes += @{
                Type = "EnvVar"
                Name = "PATH"
                Path = $Path
                Scope = "User"
            }
        } else {
            Write-Info "路径已存在于 PATH: $Path"
        }
    } catch {
        Write-Error "设置环境变量失败: $_"
        throw
    }
}

function Rollback-Changes {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "  开始回滚变更..." -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host ""

    if ($Script:Changes.Count -eq 0) {
        Write-Host "[INFO] 没有需要回滚的变更" -ForegroundColor Cyan
        return
    }

    $totalChanges = $Script:Changes.Count
    Write-Host "[INFO] 共 $totalChanges 项变更需要回滚" -ForegroundColor Cyan
    Write-Host ""

    for ($i = $Script:Changes.Count - 1; $i -ge 0; $i--) {
        $change = $Script:Changes[$i]
        $changeNum = $totalChanges - $i

        Write-Host "[$changeNum/$totalChanges] 回滚中..." -ForegroundColor Yellow

        switch ($change.Type) {
            "Backup" {
                Write-Host "  备份恢复: $($change.OriginalPath)" -ForegroundColor Yellow
                if (Test-Path $change.BackupPath) {
                    if (Test-Path $change.OriginalPath) {
                        Remove-Item -Path $change.OriginalPath -Recurse -Force -ErrorAction SilentlyContinue
                    }
                    Move-Item -Path $change.BackupPath -Destination $change.OriginalPath -Force
                    Write-Success "  已从备份恢复: $($change.OriginalPath)"
                }
            }

            "NewDir" {
                Write-Host "  删除新建目录: $($change.Path)" -ForegroundColor Yellow
                if (Test-Path $change.Path) {
                    Remove-Item -Path $change.Path -Recurse -Force -ErrorAction SilentlyContinue
                    Write-Success "  已删除: $($change.Path)"
                }
            }

            "CopyFile" {
                Write-Host "  删除文件: $($change.Path)" -ForegroundColor Yellow
                if (Test-Path $change.Path) {
                    Remove-Item -Path $change.Path -Force -ErrorAction SilentlyContinue
                    Write-Success "  已删除: $($change.Path)"
                }
            }

            "Shortcut" {
                Write-Host "  删除快捷方式: $($change.Path)" -ForegroundColor Yellow
                if (Test-Path $change.Path) {
                    Remove-Item -Path $change.Path -Force -ErrorAction SilentlyContinue
                    Write-Success "  已删除: $($change.Path)"
                }
            }

            "Registry" {
                Write-Host "  清理注册表: $($change.Path)\$($change.Name)" -ForegroundColor Yellow
                if (Test-Path $change.Path) {
                    Remove-ItemProperty -Path $change.Path -Name $change.Name -ErrorAction SilentlyContinue
                    Write-Success "  已清理: $($change.Path)\$($change.Name)"
                }
            }

            "EnvVar" {
                Write-Host "  移除 PATH: $($change.Path)" -ForegroundColor Yellow
                try {
                    $currentPath = [Environment]::GetEnvironmentVariable("Path", $change.Scope)
                    $pathParts = $currentPath -split ';' | Where-Object { $_ -and $_.TrimEnd('\') -ne $change.Path.TrimEnd('\') }
                    $newPath = $pathParts -join ';'
                    [Environment]::SetEnvironmentVariable("Path", $newPath, $change.Scope)
                    Write-Success "  已移除: $($change.Path)"
                } catch {
                    Write-Warning "  移除 PATH 失败: $_"
                }
            }
        }
    }

    $Script:Changes = @()

    Write-Host ""
    Write-Host "[OK] 回滚完成!" -ForegroundColor Green
}

function Create-UninstallScript {
    Write-Info "创建卸载脚本..."

    $uninstallScriptPath = Join-Path $INSTALL_DIR "uninstall.ps1"

    $uninstallContent = @"
`$ErrorActionPreference = 'Stop'

`$APP_NAME = "$APP_NAME"
`$INSTALL_DIR = "$INSTALL_DIR"
`$DATA_DIR = "$DATA_DIR"
`$BIN_DIR = "$BIN_DIR"

`$Quiet = `$false
if (`$args -contains "-Quiet") {
    `$Quiet = `$true
}

function Write-Info {
    param([string]`$Message)
    if (-not `$Quiet) { Write-Host "[INFO] `$Message" -ForegroundColor Cyan }
}

function Write-Success {
    param([string]`$Message)
    if (-not `$Quiet) { Write-Host "[OK] `$Message" -ForegroundColor Green }
}

function Write-Warning {
    param([string]`$Message)
    if (-not `$Quiet) { Write-Host "[WARN] `$Message" -ForegroundColor Yellow }
}

if (-not `$Quiet) {
    Write-Host ""
    Write-Host "正在卸载 `$APP_NAME..." -ForegroundColor Yellow
    Write-Host ""
}

Stop-Process -Name "mc-server" -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500
Write-Info "已停止运行中的进程"

`$desktopPath = [Environment]::GetFolderPath("Desktop")
`$desktopShortcut = Join-Path `$desktopPath "`$APP_NAME.lnk"
if (Test-Path `$desktopShortcut) {
    Remove-Item -Path `$desktopShortcut -Force -ErrorAction SilentlyContinue
    Write-Info "已删除桌面快捷方式"
}

`$startMenuPath = [Environment]::GetFolderPath("StartMenu")
`$programsPath = Join-Path `$startMenuPath "Programs"
`$appFolder = Join-Path `$programsPath "`$APP_NAME"
if (Test-Path `$appFolder) {
    Remove-Item -Path `$appFolder -Recurse -Force -ErrorAction SilentlyContinue
    Write-Info "已删除开始菜单项"
}

`$uninstallRegPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\`$APP_NAME"
if (Test-Path `$uninstallRegPath) {
    Remove-Item -Path `$uninstallRegPath -Recurse -Force -ErrorAction SilentlyContinue
    Write-Info "已删除注册表卸载入口"
}

try {
    `$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    `$pathParts = `$currentPath -split ';' | Where-Object { `$_ -and `$_ -ne "`$BIN_DIR" -and `$_ -ne "$BIN_DIR" -and `(`$_.TrimEnd('\') -ne "`$BIN_DIR".TrimEnd('\')) -and `(`$_.TrimEnd('\') -ne "$BIN_DIR".TrimEnd('\')) }
    `$newPath = `$pathParts -join ';'
    [Environment]::SetEnvironmentVariable("Path", `$newPath, "User")
    Write-Info "已清理 PATH 环境变量"
} catch {}

if (Test-Path "`$INSTALL_DIR\uninstall.ps1") {
    Remove-Item -Path "`$INSTALL_DIR\uninstall.ps1" -Force -ErrorAction SilentlyContinue
}

if (Test-Path `$INSTALL_DIR) {
    Remove-Item -Path `$INSTALL_DIR -Recurse -Force -ErrorAction SilentlyContinue
    Write-Info "已删除安装目录"
}

if (-not `$Quiet) {
    Write-Host ""
    `$response = Read-Host "是否删除用户数据目录？(Y/N)"
    if (`$response -match "^[Yy]") {
        if (Test-Path `$DATA_DIR) {
            Remove-Item -Path `$DATA_DIR -Recurse -Force -ErrorAction SilentlyContinue
            Write-Info "已删除用户数据目录"
        }
    }
}

Write-Host ""
Write-Host "卸载完成!" -ForegroundColor Green
Write-Host "感谢使用 $APP_NAME" -ForegroundColor Cyan
Write-Host ""
"@

    $uninstallContent | Out-File -FilePath $uninstallScriptPath -Encoding UTF8
    Write-Success "已创建卸载脚本: $uninstallScriptPath"

    $Script:Changes += @{
        Type = "CopyFile"
        Path = $uninstallScriptPath
    }

    return $uninstallScriptPath
}

function Install-Application {
    Write-Info "开始安装应用程序..."

    if ($Script:DryRun) {
        Write-Host ""
        Write-Host "[DRY-RUN MODE] 模拟安装，不会进行任何实际修改" -ForegroundColor Yellow
        Write-Host ""
    }

    try {
        Write-Info "准备安装目录..."
        Backup-Directory -Path $INSTALL_DIR
        Backup-Directory -Path $BIN_DIR
        Backup-Directory -Path $DATA_DIR

        New-Directory -Path $INSTALL_DIR
        New-Directory -Path $BIN_DIR
        New-Directory -Path $DATA_DIR

        $artifactsPath = Join-Path $SCRIPT_DIR "..\artifacts\windows"
        if (Test-Path $artifactsPath) {
            $binaries = Get-ChildItem -Path $artifactsPath -Filter "*.exe" -ErrorAction SilentlyContinue
            foreach ($binary in $binaries) {
                Copy-File-Safe -Source $binary.FullName -Destination (Join-Path $BIN_DIR $binary.Name)
            }
            Write-Success "已复制 $($binaries.Count) 个二进制文件"
        } else {
            Write-Warning "未找到预编译二进制文件，跳过"
        }

        $configTemplate = Join-Path $SCRIPT_DIR "templates\config.yaml"
        if (Test-Path $configTemplate) {
            Copy-File-Safe -Source $configTemplate -Destination (Join-Path $DATA_DIR "config.yaml")
        }

        Create-UninstallScript

        $desktopIntegrationPath = Join-Path $SCRIPT_DIR "utils\desktop-integration.ps1"
        if (Test-Path $desktopIntegrationPath) {
            Import-Module $desktopIntegrationPath -Force
            $executablePath = Join-Path $BIN_DIR "mc-server.exe"
            if (-not (Test-Path $executablePath)) {
                $dummyExePath = Join-Path $BIN_DIR "mc-server.exe"
                "# MC Server Panel" | Out-File -FilePath $dummyExePath -Encoding ASCII
                Write-Warning "创建占位符可执行文件，实际部署时请替换"
            }

            Install-DesktopIntegration `
                -InstallPath $INSTALL_DIR `
                -AppName $APP_NAME `
                -AppVersion $APP_VERSION `
                -UninstallScript (Join-Path $INSTALL_DIR "uninstall.ps1") `
                -CreateDesktopShortcut $true `
                -CreateStartMenuShortcut $true `
                -CreateUninstallEntry $true | Out-Null
        }

        Add-PathToEnvironment-Safe -Path $BIN_DIR

        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "  安装完成!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "安装路径: $INSTALL_DIR" -ForegroundColor Cyan
        Write-Host "数据路径: $DATA_DIR" -ForegroundColor Cyan
        Write-Host "卸载脚本: $INSTALL_DIR\uninstall.ps1" -ForegroundColor Cyan
        Write-Host ""

        if ($Script:Changes.Count -gt 0 -and -not $Script:DryRun) {
            Write-Host "[INFO] 已追踪 $($Script:Changes.Count) 项变更" -ForegroundColor Cyan
        }

    } catch {
        Write-Error "安装失败: $($_.Exception.Message)"
        if (-not $Script:DryRun) {
            Rollback-Changes
        }
        exit 1
    }
}

function Main {
    if ($Help) {
        Write-Banner
        Show-Help
        exit 0
    }

    Write-Banner

    Import-Utils

    if ($AutoElevate -and -not (Test-Admin)) {
        Request-AdminPrivileges
    }
    Require-AdminPrivileges -Message "此安装程序需要管理员权限"
    Write-Success "管理员权限已获取"

    if (-not (Test-SystemRequirements)) {
        Write-Error "系统要求检查失败"
        exit 1
    }

    Install-Dependencies

    Install-Application

    Write-Host "安装成功!" -ForegroundColor Green
    exit 0
}

try {
    Main
} catch {
    Write-Error "安装过程中发生未处理的错误: $($_.Exception.Message)"
    if (-not $Script:DryRun) {
        Rollback-Changes
    }
    exit 1
}

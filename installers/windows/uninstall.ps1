#Requires -Version 5.1
<#
.SYNOPSIS
    Rust MC 服务器面板卸载器 - Windows 卸载脚本
.DESCRIPTION
    用于从 Windows 系统完全卸载 MC Server Panel 应用程序
    支持静默卸载和选择性删除用户数据
.NOTES
    作者: MC Server Team
    版本: 1.0.0
    依赖: PowerShell 5.1+
#>

param(
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'

$APP_NAME = "MC Server Panel"
$APP_VERSION = "1.0.0"
$INSTALL_DIR = "$env:LOCALAPPDATA\MC Server Panel"
$BIN_DIR = "$InstallDir\bin"
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
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "  $APP_NAME 卸载器" -ForegroundColor Yellow
    Write-Host "  Version: $APP_VERSION" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host ""
}

function Show-Help {
    Write-Host ""
    Write-Host "用法: .\uninstall.ps1 [-Quiet]" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "选项:" -ForegroundColor White
    Write-Host "  -Quiet    静默模式，不询问确认（自动删除用户数据）" -ForegroundColor Gray
    Write-Host ""
    Write-Host "示例:" -ForegroundColor White
    Write-Host "  .\uninstall.ps1              # 交互式卸载" -ForegroundColor Gray
    Write-Host "  .\uninstall.ps1 -Quiet       # 静默卸载" -ForegroundColor Gray
    Write-Host ""
}

function Stop-RunningProcess {
    Write-Info "正在停止运行中的进程..."

    $processes = @("mc-server", "mc-server-panel")
    foreach ($processName in $processes) {
        $process = Get-Process -Name $processName -ErrorAction SilentlyContinue
        if ($process) {
            try {
                $process | Stop-Process -Force -ErrorAction Stop
                Write-Success "已停止进程: $processName"
            } catch {
                Write-Warning "无法停止进程: $processName"
            }
        }
    }

    Start-Sleep -Milliseconds 500
}

function Remove-DesktopShortcut {
    Write-Info "正在删除桌面快捷方式..."

    $desktopPath = [Environment]::GetFolderPath("Desktop")
    $shortcutPath = Join-Path $desktopPath "$APP_NAME.lnk"

    if (Test-Path $shortcutPath) {
        try {
            Remove-Item -Path $shortcutPath -Force -ErrorAction Stop
            Write-Success "已删除桌面快捷方式"
        } catch {
            Write-Warning "删除桌面快捷方式失败: $_"
        }
    } else {
        Write-Info "桌面快捷方式不存在，跳过"
    }
}

function Remove-StartMenuShortcuts {
    Write-Info "正在删除开始菜单快捷方式..."

    $startMenuPath = [Environment]::GetFolderPath("StartMenu")
    $programsPath = Join-Path $startMenuPath "Programs"
    $appFolderPath = Join-Path $programsPath $APP_NAME

    if (Test-Path $appFolderPath) {
        try {
            Remove-Item -Path $appFolderPath -Recurse -Force -ErrorAction Stop
            Write-Success "已删除开始菜单文件夹"
        } catch {
            Write-Warning "删除开始菜单文件夹失败: $_"
        }
    } else {
        Write-Info "开始菜单文件夹不存在，跳过"
    }
}

function Remove-RegistryEntries {
    Write-Info "正在清理注册表..."

    $uninstallRegPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$APP_NAME"
    if (Test-Path $uninstallRegPath) {
        try {
            Remove-Item -Path $uninstallRegPath -Recurse -Force -ErrorAction Stop
            Write-Success "已删除注册表卸载入口"
        } catch {
            Write-Warning "删除注册表卸载入口失败: $_"
        }
    } else {
        Write-Info "注册表卸载入口不存在，跳过"
    }

    $appRegPath = "HKCU:\Software\$APP_NAME"
    if (Test-Path $appRegPath) {
        try {
            Remove-Item -Path $appRegPath -Recurse -Force -ErrorAction Stop
            Write-Info "已删除应用程序注册表项"
        } catch {
            Write-Warning "删除应用程序注册表项失败: $_"
        }
    }
}

function Remove-EnvironmentVariables {
    Write-Info "正在清理环境变量..."

    try {
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($currentPath) {
            $pathParts = $currentPath -split ';' | Where-Object {
                $_ -and $_.TrimEnd('\') -ne $BIN_DIR.TrimEnd('\')
            }

            $normalizedBinDir = $BIN_DIR.TrimEnd('\').ToLower()
            $filteredParts = @()
            $removed = $false

            foreach ($part in $pathParts) {
                $normalizedPart = $part.TrimEnd('\').ToLower()
                if ($normalizedPart -ne $normalizedBinDir) {
                    $filteredParts += $part
                } else {
                    $removed = $true
                }
            }

            if ($removed) {
                $newPath = $filteredParts -join ';'
                [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
                Write-Success "已从 PATH 移除安装目录"
            } else {
                Write-Info "PATH 中未找到安装目录"
            }
        }
    } catch {
        Write-Warning "清理 PATH 环境变量失败: $_"
    }
}

function Remove-InstallationDirectory {
    Write-Info "正在删除安装目录..."

    if (Test-Path $INSTALL_DIR) {
        try {
            $uninstallScriptInDir = Join-Path $INSTALL_DIR "uninstall.ps1"
            if (Test-Path $uninstallScriptInDir) {
                try {
                    Remove-Item -Path $uninstallScriptInDir -Force -ErrorAction SilentlyContinue
                } catch {}
            }

            Remove-Item -Path $INSTALL_DIR -Recurse -Force -ErrorAction Stop
            Write-Success "已删除安装目录: $INSTALL_DIR"
        } catch {
            Write-Warning "删除安装目录失败: $_"
            Write-Warning "可能需要管理员权限才能删除某些文件"
        }
    } else {
        Write-Info "安装目录不存在，跳过"
    }
}

function Remove-UserData {
    param([bool]$Force)

    if ($Force) {
        Write-Info "静默模式：自动删除用户数据"
    } else {
        Write-Host ""
        $response = Read-Host "是否删除用户数据目录？(Y/N)"
        if ($response -notmatch "^[Yy]") {
            Write-Info "保留用户数据目录: $DATA_DIR"
            return
        }
    }

    if (Test-Path $DATA_DIR) {
        try {
            Remove-Item -Path $DATA_DIR -Recurse -Force -ErrorAction Stop
            Write-Success "已删除用户数据目录: $DATA_DIR"
        } catch {
            Write-Warning "删除用户数据目录失败: $_"
            Write-Warning "可能需要手动删除: $DATA_DIR"
        }
    } else {
        Write-Info "用户数据目录不存在，跳过"
    }
}

function Confirm-Uninstall {
    if ($Quiet) {
        return $true
    }

    Write-Host ""
    Write-Host "确认要卸载 $APP_NAME 吗？" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "此操作将删除:" -ForegroundColor White
    Write-Host "  - 桌面快捷方式" -ForegroundColor Gray
    Write-Host "  - 开始菜单快捷方式" -ForegroundColor Gray
    Write-Host "  - 注册表项" -ForegroundColor Gray
    Write-Host "  - 环境变量" -ForegroundColor Gray
    Write-Host "  - 安装目录 ($INSTALL_DIR)" -ForegroundColor Gray
    Write-Host ""
    Write-Host "用户数据目录 ($DATA_DIR) 将根据后续选择处理" -ForegroundColor Gray
    Write-Host ""

    $response = Read-Host "继续卸载？(Y/N)"
    return ($response -match "^[Yy]")
}

function Main {
    if ($args -contains "-Help" -or $args -contains "-?") {
        Write-Banner
        Show-Help
        exit 0
    }

    Write-Banner

    if (-not (Confirm-Uninstall)) {
        Write-Info "取消卸载操作"
        exit 0
    }

    Write-Host ""
    Write-Host "开始卸载..." -ForegroundColor Yellow
    Write-Host ""

    try {
        Stop-RunningProcess

        Remove-DesktopShortcut

        Remove-StartMenuShortcuts

        Remove-RegistryEntries

        Remove-EnvironmentVariables

        Remove-InstallationDirectory

        Remove-UserData -Force $Quiet

        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "  卸载完成!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "感谢使用 $APP_NAME" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "如果重新安装，请运行 install.ps1" -ForegroundColor Gray
        Write-Host ""

        exit 0

    } catch {
        Write-Error "卸载过程中发生错误: $($_.Exception.Message)"
        Write-Host ""
        Write-Host "请尝试以管理员身份运行此脚本" -ForegroundColor Yellow
        exit 1
    }
}

Main

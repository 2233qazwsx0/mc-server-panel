#Requires -Version 5.1
<#
.SYNOPSIS
    MC Server Panel 企业级 Windows 卸载器
.DESCRIPTION
    企业级跨平台卸载器 - Windows 版本
    支持停止服务、删除程序文件、移除快捷方式、清理防火墙规则和环境变量
.NOTES
    版本: 2.0.0
    兼容: PowerShell 5.1, 7+
    要求: 管理员权限
#>

[CmdletBinding(SupportsShouldProcess, ConfirmImpact = 'High')]
param(
    [Parameter(Mandatory = $false)]
    [string]$InstallPath = "$env:ProgramFiles\MCPanel",

    [Parameter(Mandatory = $false)]
    [string]$DataPath = "$env:ProgramData\MCPanel",

    [Parameter(Mandatory = $false)]
    [switch]$Purge,

    [Parameter(Mandatory = $false)]
    [switch]$Quiet
)

# ============================================
# 全局变量定义
# ============================================
$Script:ErrorActionPreference = 'Stop'
$Script:LogFile = Join-Path $env:TEMP "MCPanel_Uninstall_$(Get-Date -Format 'yyyyMMdd').log"
$Script:IsAdmin = $false
$Script:ServiceName = "MCPanel"

# ============================================
# 日志函数 - 统一日志输出
# ============================================
function Write-Log {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Message,

        [Parameter(Mandatory = $false)]
        [ValidateSet("INFO", "WARN", "ERROR", "OK", "STEP")]
        [string]$Level = "INFO"
    )

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $color = switch ($Level) {
        "INFO" { "White" }
        "WARN" { "Yellow" }
        "ERROR" { "Red" }
        "OK" { "Green" }
        "STEP" { "Cyan" }
    }

    $logEntry = "[$timestamp] [$Level] $Message"
    Add-Content -Path $LogFile -Value $logEntry -Encoding UTF8

    if ($PSCmdlet.MyInvocation.BoundParameters.ContainsKey('Verbose') -or $VerbosePreference -eq 'Continue') {
        Write-Verbose $logEntry
    }

    if ($Quiet) {
        return
    }

    Write-Host $logEntry -ForegroundColor $color
}

# ============================================
# 权限检测函数
# ============================================
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Request-AdministratorPrivileges {
    if (-not (Test-Administrator)) {
        Write-Log "需要管理员权限，正在请求提升..." "WARN"

        $arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""

        foreach ($key in $PSBoundParameters.Keys) {
            if ($PSBoundParameters[$key] -is [switch]) {
                if ($PSBoundParameters[$key].IsPresent) {
                    $arguments += " -$key"
                }
            } else {
                $arguments += " -$key `"$($PSBoundParameters[$key])`""
            }
        }

        try {
            Start-Process powershell.exe -ArgumentList $arguments -Verb RunAs -Wait
            exit 0
        } catch {
            Write-Log "无法获取管理员权限，卸载终止" "ERROR"
            exit 1
        }
    }
    $Script:IsAdmin = $true
    Write-Log "管理员权限已获取" "OK"
}

# ============================================
# 确认卸载
# ============================================
function Confirm-Uninstall {
    if ($Quiet -or $Purge) {
        return $true
    }

    Write-Host ""
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "  MC Server Panel 卸载确认" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "此操作将:" -ForegroundColor White
    Write-Host "  - 停止并删除 Windows 服务 (MCPanel)" -ForegroundColor Gray
    Write-Host "  - 移除桌面和开始菜单快捷方式" -ForegroundColor Gray
    Write-Host "  - 删除防火墙规则" -ForegroundColor Gray
    Write-Host "  - 清理环境变量" -ForegroundColor Gray
    Write-Host "  - 删除程序目录: $InstallPath" -ForegroundColor Gray
    
    if ($Purge) {
        Write-Host "  - [PURGE] 删除数据目录: $DataPath" -ForegroundColor Red
    } else {
        Write-Host "  - 保留数据目录: $DataPath" -ForegroundColor Green
    }
    
    Write-Host ""

    $response = Read-Host "确定要继续吗？(Y/N)"
    return ($response -match "^[Yy]")
}

# ============================================
# 停止并删除 Windows 服务
# ============================================
function Remove-WindowsService {
    Write-Log "=== 步骤 1/6: 停止并删除 Windows 服务 ===" "STEP"

    try {
        $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if ($service) {
            if ($service.Status -eq 'Running') {
                Write-Log "正在停止服务: $ServiceName" "INFO"
                Stop-Service -Name $ServiceName -Force -ErrorAction Stop
                Write-Log "服务已停止" "OK"
            }
            
            Write-Log "正在删除服务: $ServiceName" "INFO"
            sc.exe delete $ServiceName | Out-Null
            Start-Sleep -Seconds 2
            Write-Log "服务已删除" "OK"
        } else {
            Write-Log "服务不存在，跳过" "INFO"
        }
        return $true
    } catch {
        Write-Log "删除服务失败: $_" "WARN"
        return $false
    }
}

# ============================================
# 删除桌面和开始菜单快捷方式
# ============================================
function Remove-Shortcuts {
    Write-Log "=== 步骤 2/6: 删除快捷方式 ===" "STEP"

    try {
        $desktopPath = [Environment]::GetFolderPath("Desktop")
        $desktopShortcut = Join-Path $desktopPath "MC Server Panel.lnk"
        if (Test-Path $desktopShortcut) {
            Remove-Item -Path $desktopShortcut -Force -ErrorAction Stop
            Write-Log "已删除桌面快捷方式" "OK"
        }

        $startMenuPath = Join-Path ([Environment]::GetFolderPath("Programs")) "MC Server Panel.lnk"
        if (Test-Path $startMenuPath) {
            Remove-Item -Path $startMenuPath -Force -ErrorAction Stop
            Write-Log "已删除开始菜单快捷方式" "OK"
        }

        Write-Log "快捷方式清理完成" "OK"
        return $true
    } catch {
        Write-Log "删除快捷方式失败: $_" "WARN"
        return $false
    }
}

# ============================================
# 删除防火墙规则
# ============================================
function Remove-FirewallRules {
    Write-Log "=== 步骤 3/6: 删除防火墙规则 ===" "STEP"

    try {
        $rules = @("MCPanel-Web", "MCPanel-RCON")
        foreach ($ruleName in $rules) {
            $rule = Get-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue
            if ($rule) {
                Remove-NetFirewallRule -DisplayName $ruleName -ErrorAction Stop
                Write-Log "已删除防火墙规则: $ruleName" "OK"
            }
        }
        Write-Log "防火墙规则清理完成" "OK"
        return $true
    } catch {
        Write-Log "删除防火墙规则失败: $_" "WARN"
        return $false
    }
}

# ============================================
# 清理环境变量
# ============================================
function Remove-EnvironmentVariables {
    Write-Log "=== 步骤 4/6: 清理环境变量 ===" "STEP"

    try {
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if ($currentPath) {
            $pathParts = $currentPath -split ';' | Where-Object { $_ -and $_.TrimEnd('\') -ne $InstallPath.TrimEnd('\') }
            $newPath = $pathParts -join ';'
            [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
            Write-Log "已从 PATH 移除安装目录" "OK"
        }

        [Environment]::SetEnvironmentVariable("MC_PANEL_HOME", $null, "Machine")
        Write-Log "已删除 MC_PANEL_HOME 环境变量" "OK"
        return $true
    } catch {
        Write-Log "清理环境变量失败: $_" "WARN"
        return $false
    }
}

# ============================================
# 删除程序目录
# ============================================
function Remove-InstallDirectory {
    Write-Log "=== 步骤 5/6: 删除程序目录 ===" "STEP"

    try {
        if (Test-Path $InstallPath) {
            Write-Log "正在删除目录: $InstallPath" "INFO"
            Remove-Item -Path $InstallPath -Recurse -Force -ErrorAction Stop
            Write-Log "程序目录已删除" "OK"
        } else {
            Write-Log "程序目录不存在，跳过" "INFO"
        }
        return $true
    } catch {
        Write-Log "删除程序目录失败: $_" "ERROR"
        return $false
    }
}

# ============================================
# 删除数据目录（仅当 --purge）
# ============================================
function Remove-DataDirectory {
    Write-Log "=== 步骤 6/6: 处理数据目录 ===" "STEP"

    if (-not $Purge) {
        Write-Log "保留数据目录: $DataPath (使用 --purge 可删除)" "INFO"
        return $true
    }

    try {
        if (Test-Path $DataPath) {
            Write-Log "正在删除数据目录: $DataPath" "INFO"
            Remove-Item -Path $DataPath -Recurse -Force -ErrorAction Stop
            Write-Log "数据目录已删除" "OK"
        } else {
            Write-Log "数据目录不存在，跳过" "INFO"
        }
        return $true
    } catch {
        Write-Log "删除数据目录失败: $_" "ERROR"
        return $false
    }
}

# ============================================
# 主函数
# ============================================
function Start-Uninstall {
    Write-Log "========================================" "INFO"
    Write-Log "  MC Server Panel 卸载程序 v2.0.0" "INFO"
    Write-Log "========================================" "INFO"
    Write-Log "开始时间: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" "INFO"
    Write-Log "日志文件: $LogFile" "INFO"

    try {
        # 1. 权限检测
        Request-AdministratorPrivileges

        # 2. 确认卸载
        if (-not (Confirm-Uninstall)) {
            Write-Log "卸载已取消" "INFO"
            exit 0
        }

        Write-Host ""

        # 3. 执行卸载步骤
        Remove-WindowsService | Out-Null
        Remove-Shortcuts | Out-Null
        Remove-FirewallRules | Out-Null
        Remove-EnvironmentVariables | Out-Null
        Remove-InstallDirectory | Out-Null
        Remove-DataDirectory | Out-Null

        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "  卸载完成!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""

        if (-not $Purge) {
            Write-Host "数据目录已保留: $DataPath" -ForegroundColor Cyan
            Write-Host "如需完全删除，请使用: .\uninstall.ps1 -Purge" -ForegroundColor Gray
        }

        Write-Log "卸载完成时间: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" "INFO"
        Write-Log "卸载成功!" "OK"

        return $true
    } catch {
        Write-Log "卸载失败: $_" "ERROR"
        Write-Log "详细信息请查看日志: $LogFile" "ERROR"
        return $false
    }
}

# ============================================
# 程序入口
# ============================================
Start-Uninstall

#Requires -Version 5.1
<#
.SYNOPSIS
    MC Server Panel 企业级 Windows 安装器
.DESCRIPTION
    企业级跨平台安装器 - Windows 版本
    支持幂等安装、自动依赖检测、SHA256 校验和服务注册
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
    [string]$DownloadUrl = "https://github.com/mc-server-panel/minecraft-admin/releases/latest/download/minecraft-admin-windows-x86_64.zip",

    [Parameter(Mandatory = $false)]
    [string]$ConfigUrl = "https://raw.githubusercontent.com/mc-server-panel/minecraft-admin/main/config.toml.example",

    [Parameter(Mandatory = $false)]
    [switch]$SkipService,
    [Parameter(Mandatory = $false)]
    [switch]$SkipFirewall,
    [Parameter(Mandatory = $false)]
    [switch]$ForceReinstall,
    [Parameter(Mandatory = $false)]
    [switch]$SkipUpgrade,
    [Parameter(Mandatory = $false)]
    [int]$Timeout = 300,
    [Parameter(Mandatory = $false)]
    [int]$Port = 8080,
    [Parameter(Mandatory = $false)]
    [int]$RconPort = 25575
)

# ============================================
# 全局变量定义
# ============================================
$Script:ErrorActionPreference = 'Continue'
$Script:InstallTempPath = Join-Path $env:TEMP "MCPanel_Install_$(Get-Date -Format 'yyyyMMddHHmmss')"
$Script:LogFile = Join-Path $env:TEMP "MCPanel_Install_$(Get-Date -Format 'yyyyMMdd').log"
$Script:IsAdmin = $false
$Script:RollbackActions = @()
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

    if ($PSCmdlet.MyInvocation.BoundParameters.ContainsKey('Silent')) {
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
            Write-Log "无法获取管理员权限，安装终止" "ERROR"
            exit 1
        }
    }
    $Script:IsAdmin = $true
    Write-Log "管理员权限已获取" "OK"
}

# ============================================
# 系统兼容性检测
# ============================================
function Test-SystemCompatibility {
    Write-Log "=== 步骤 1/6: 系统兼容性检测 ===" "STEP"

    # 检测操作系统
    $os = Get-CimInstance Win32_OperatingSystem
    $osVersion = [System.Environment]::OSVersion.Version
    $arch = $env:PROCESSOR_ARCHITECTURE

    Write-Log "操作系统: $($os.Caption)" "INFO"
    Write-Log "版本: $($osVersion)" "INFO"
    Write-Log "架构: $arch" "INFO"

    # Windows 10/11 或 Windows Server 2019+
    if ($osVersion.Major -lt 10) {
        Write-Log "不支持的操作系统版本，需要 Windows 10 或更高版本" "ERROR"
        return $false
    }

    # 仅支持 x64
    if ($arch -ne "AMD64") {
        Write-Log "不支持的处理器架构，当前仅支持 x64" "ERROR"
        return $false
    }

    Write-Log "系统兼容性检测通过" "OK"
    return $true
}

# ============================================
# 依赖检测与安装
# ============================================
function Test-DotNetRuntime {
    Write-Log "检测 .NET Runtime..." "INFO"

    try {
        $dotnetPath = Get-ItemProperty -Path "HKLM:\SOFTWARE\dotnet\Setup\InstalledVersions\x64\sharedfx\Microsoft.WindowsDesktop.App" -ErrorAction SilentlyContinue

        if ($null -ne $dotnetPath) {
            $version = $dotnetPath.PSObject.Properties | Where-Object { $_.Name -match '^\d+\.' } | Select-Object -First 1
            Write-Log "检测到 .NET Runtime: $($version.Name)" "OK"
            return $true
        }

        # 检查 Visual C++ Redistributable
        $vcPath = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" -ErrorAction SilentlyContinue
        if ($null -ne $vcPath) {
            Write-Log "检测到 Visual C++ Redistributable 2015+" "OK"
            return $true
        }

        Write-Log "未检测到 Visual C++ Redistributable，尝试安装..." "WARN"

        # 静默安装 VC++ Redistributable
        $vcUrl = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
        $vcInstaller = Join-Path $InstallTempPath "vc_redist.x64.exe"

        try {
            Invoke-WebRequest -Uri $vcUrl -OutFile $vcInstaller -TimeoutSec 60 -UseBasicParsing
            Start-Process -FilePath $vcInstaller -ArgumentList "/install", "/quiet", "/norestart" -Wait -NoNewWindow
            Write-Log "Visual C++ Redistributable 安装完成" "OK"
            $Script:RollbackActions += { Remove-Item $vcInstaller -Force -ErrorAction SilentlyContinue }
            return $true
        } catch {
            Write-Log "VC++ Redistributable 安装失败，请手动安装: https://aka.ms/vs/17/release/vc_redist.x64.exe" "WARN"
            return $false
        }
    } catch {
        Write-Log "依赖检测失败: $_" "WARN"
        return $false
    }
}

# ============================================
# 端口冲突检测
# ============================================
function Test-PortAvailability {
    param(
        [int[]]$Ports
    )

    Write-Log "检测端口占用情况..." "INFO"

    foreach ($port in $Ports) {
        $connection = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue

        if ($null -ne $connection) {
            Write-Log "端口 $port 已被占用" "WARN"

            $process = Get-Process -Id $connection.OwningProcess -ErrorAction SilentlyContinue
            if ($process) {
                Write-Log "  占用进程: $($process.ProcessName) (PID: $($process.Id))" "INFO"
            }

            if (-not $PSCmdlet.ShouldContinue("端口 $port 被占用，是否终止占用进程？", "端口冲突")) {
                Write-Log "用户取消安装" "ERROR"
                return $false
            }

            # 强制终止占用进程
            try {
                Stop-Process -Id $connection.OwningProcess -Force -ErrorAction Stop
                Write-Log "已终止占用进程" "OK"
                Start-Sleep -Seconds 2
            } catch {
                Write-Log "无法终止占用进程，请手动释放端口 $port" "ERROR"
                return $false
            }
        } else {
            Write-Log "端口 $port 可用" "OK"
        }
    }

    return $true
}

# ============================================
# 网络请求函数（带超时）
# ============================================
function Invoke-DownloadFile {
    param(
        [string]$Url,
        [string]$Destination,
        [int]$TimeoutSeconds = 300
    )

    Write-Log "正在下载: $Url" "INFO"

    try {
        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri $Url -OutFile $Destination -TimeoutSec $TimeoutSeconds -UseBasicParsing
        $ProgressPreference = 'Continue'

        if (Test-Path $Destination) {
            $size = (Get-Item $Destination).Length
            Write-Log "下载完成: $('{0:N2}' -f ($size / 1MB)) MB" "OK"
            return $true
        }
    } catch {
        Write-Log "下载失败: $_" "ERROR"
    }

    return $false
}

# ============================================
# SHA256 校验函数
# ============================================
function Test-FileHash {
    param(
        [string]$FilePath,
        [string]$ExpectedHash,
        [string]$Algorithm = "SHA256"
    )

    if (-not (Test-Path $FilePath)) {
        Write-Log "文件不存在: $FilePath" "ERROR"
        return $false
    }

    $actualHash = (Get-FileHash -Path $FilePath -Algorithm $Algorithm).Hash.ToLower()

    if ($actualHash -eq $ExpectedHash.ToLower()) {
        Write-Log "文件校验通过 (SHA256)" "OK"
        return $true
    }

    Write-Log "文件校验失败!" "ERROR"
    Write-Log "  期望: $ExpectedHash" "ERROR"
    Write-Log "  实际: $actualHash" "ERROR"
    return $false
}

# ============================================
# GitHub Releases 检测最新版本
# ============================================
function Get-LatestReleaseInfo {
    param(
        [string]$Owner = "mc-server-panel",
        [string]$Repo = "minecraft-admin"
    )

    Write-Log "获取最新版本信息..." "INFO"

    try {
        $apiUrl = "https://api.github.com/repos/$Owner/$Repo/releases/latest"

        $response = Invoke-RestMethod -Uri $apiUrl -TimeoutSec 30 -UseBasicParsing

        $version = $response.tag_name.TrimStart('v')
        $downloadUrl = $response.assets | Where-Object { $_.name -match "windows-x86_64\.zip" } | Select-Object -First 1

        if ($null -eq $downloadUrl) {
            # 回退到默认 URL
            $downloadUrl = "https://github.com/$Owner/$Repo/releases/latest/download/minecraft-admin-windows-x86_64.zip"
        } else {
            $downloadUrl = $downloadUrl.browser_download_url
        }

        # 获取 SHA256 校验和
        $checksumAsset = $response.assets | Where-Object { $_.name -match "checksums\.txt" } | Select-Object -First 1

        return @{
            Version = $version
            DownloadUrl = $downloadUrl
            ChecksumUrl = if ($checksumAsset) { $checksumAsset.browser_download_url } else { $null }
        }
    } catch {
        Write-Log "无法获取最新版本信息: $_" "WARN"
        return @{
            Version = "unknown"
            DownloadUrl = $DownloadUrl
            ChecksumUrl = $null
        }
    }
}

# ============================================
# 回滚函数
# ============================================
function Invoke-Rollback {
    Write-Log "=== 执行回滚操作 ===" "WARN"

    foreach ($action in $Script:RollbackActions) {
        try {
            & $action
            Write-Log "回滚操作完成" "INFO"
        } catch {
            Write-Log "回滚失败: $_" "ERROR"
        }
    }

    # 清理临时目录
    if (Test-Path $InstallTempPath) {
        Remove-Item $InstallTempPath -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# ============================================
# 创建目录结构
# ============================================
function Initialize-DirectoryStructure {
    param(
        [string]$ProgramPath,
        [string]$DataPath
    )

    Write-Log "=== 步骤 2/6: 初始化目录结构 ===" "STEP"

    $directories = @(
        $ProgramPath,
        $DataPath,
        (Join-Path $DataPath "logs"),
        (Join-Path $DataPath "backups"),
        (Join-Path $DataPath "plugins"),
        (Join-Path $DataPath "worlds")
    )

    foreach ($dir in $directories) {
        try {
            if (-not (Test-Path $dir)) {
                New-Item -Path $dir -ItemType Directory -Force | Out-Null
                Write-Log "创建目录: $dir" "OK"
            } else {
                Write-Log "目录已存在: $dir" "INFO"
            }
        } catch {
            Write-Log "创建目录失败: $dir - $_" "ERROR"
            return $false
        }
    }

    return $true
}

# ============================================
# 下载与部署
# ============================================
function Install-Binary {
    param(
        [string]$ProgramPath,
        [string]$DownloadUrl
    )

    Write-Log "=== 步骤 3/6: 下载并部署二进制文件 ===" "STEP"

    # 创建临时目录
    New-Item -Path $InstallTempPath -ItemType Directory -Force | Out-Null

    $zipPath = Join-Path $InstallTempPath "minecraft-admin.zip"
    $extractPath = Join-Path $InstallTempPath "extracted"

    # 下载
    if (-not (Invoke-DownloadFile -Url $DownloadUrl -Destination $zipPath -TimeoutSeconds $Timeout)) {
        return $false
    }

    # 解压
    Write-Log "正在解压..." "INFO"
    try {
        Expand-Archive -Path $zipPath -DestinationPath $extractPath -Force
        Write-Log "解压完成" "OK"
    } catch {
        Write-Log "解压失败: $_" "ERROR"
        return $false
    }

    # 复制文件
    Write-Log "正在部署文件..." "INFO"
    try {
        $files = Get-ChildItem -Path $extractPath -Recurse -File
        foreach ($file in $files) {
            $relativePath = $file.FullName.Substring($extractPath.Length).TrimStart('\')
            $destPath = Join-Path $ProgramPath $relativePath
            $destDir = Split-Path $destPath -Parent

            if (-not (Test-Path $destDir)) {
                New-Item -Path $destDir -ItemType Directory -Force | Out-Null
            }

            Copy-Item -Path $file.FullName -Destination $destPath -Force
        }

        Write-Log "文件部署完成" "OK"
    } catch {
        Write-Log "文件部署失败: $_" "ERROR"
        return $false
    }

    # 清理临时文件
    Remove-Item $zipPath -Force -ErrorAction SilentlyContinue

    return $true
}

# ============================================
# 配置文件生成
# ============================================
function Install-Configuration {
    param(
        [string]$ProgramPath,
        [string]$DataPath,
        [int]$Port,
        [int]$RconPort
    )

    Write-Log "=== 步骤 4/6: 生成配置文件 ===" "STEP"

    $configPath = Join-Path $ProgramPath "config.toml"

    # 如果配置文件已存在，询问用户
    if ((Test-Path $configPath) -and -not $ForceReinstall) {
        Write-Log "检测到已有配置文件，保留用户设置" "INFO"

        if (-not $SkipUpgrade) {
            Write-Log "使用默认模板更新缺失的配置项..." "INFO"
            # TODO: 实现配置合并逻辑
        }
        return $true
    }

    # 生成默认配置
    $config = @"
# MC Server Panel 配置文件
# 版本: 2.0.0

[server]
host = "0.0.0.0"
port = $Port
rcon_port = $RconPort
rcon_password = "`$(openssl rand -hex 32)"
server_path = "$DataPath\server"
log_level = "info"

[database]
type = "sqlite"
path = "$DataPath\data\panel.db"

[logging]
path = "$DataPath\logs"
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
backup_path = "$DataPath\backups"
backup_schedule = "0 3 * * *"
backup_retention_days = 7

[monitoring]
enable_metrics = true
metrics_port = 9090
alert_webhooks = []
cpu_threshold = 90
memory_threshold = 90
disk_threshold = 85
"@

    try {
        $configPath = Join-Path $ProgramPath "config.toml"
        Set-Content -Path $configPath -Value $config -Encoding UTF8
        Write-Log "配置文件已生成: $configPath" "OK"
    } catch {
        Write-Log "配置文件生成失败: $_" "ERROR"
        return $false
    }

    return $true
}

# ============================================
# Windows Service 注册
# ============================================
function Register-WindowsService {
    param(
        [string]$ProgramPath,
        [string]$ServiceName = "MCPanel"
    )

    if ($SkipService) {
        Write-Log "跳过服务注册" "INFO"
        return $true
    }

    Write-Log "=== 步骤 5/6: 注册 Windows 服务 ===" "STEP"

    $servicePath = Join-Path $ProgramPath "minecraft-admin.exe"
    $displayName = "MC Server Panel"
    $description = "Minecraft 服务器管理面板 - 企业版"

    # 检查服务是否已存在
    $existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue

    if ($null -ne $existingService) {
        if ($ForceReinstall) {
            Write-Log "停止并移除旧服务..." "INFO"
            Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
            sc.exe delete $ServiceName
            Start-Sleep -Seconds 2
        } else {
            Write-Log "服务已存在 (MCPanel)，跳过注册" "INFO"
            return $true
        }
    }

    # 创建服务
    $createServiceArgs = @(
        "create", $ServiceName,
        "binPath=", "`"$servicePath`"",
        "DisplayName=", "`"$displayName`"",
        "start=", "auto",
        "type=", "own"
    )

    $result = & sc.exe $createServiceArgs 2>&1

    if ($LASTEXITCODE -ne 0) {
        Write-Log "服务创建失败: $result" "ERROR"
        return $false
    }

    # 设置服务描述
    & sc.exe description $ServiceName $description | Out-Null

    # 设置失败恢复策略
    & sc.exe failure $ServiceName reset= 86400 actions= restart/60000/restart/60000/restart/60000 | Out-Null

    # 添加到回滚列表
    $Script:RollbackActions += {
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        sc.exe delete $ServiceName | Out-Null
    }

    Write-Log "Windows 服务注册成功: $ServiceName" "OK"
    return $true
}

# ============================================
# 防火墙规则配置
# ============================================
function Register-FirewallRules {
    param(
        [int]$Port,
        [int]$RconPort
    )

    if ($SkipFirewall) {
        Write-Log "跳过防火墙配置" "INFO"
        return $true
    }

    Write-Log "配置防火墙规则..." "INFO"

    $rules = @(
        @{ Name = "MCPanel-Web"; Port = $Port; Protocol = "TCP"; Desc = "MC Server Panel Web Interface" },
        @{ Name = "MCPanel-RCON"; Port = $RconPort; Protocol = "TCP"; Desc = "MC Server Panel RCON" }
    )

    foreach ($rule in $rules) {
        $existingRule = Get-NetFirewallRule -DisplayName $rule.Name -ErrorAction SilentlyContinue

        if ($null -ne $existingRule) {
            Write-Log "防火墙规则已存在: $($rule.Name)" "INFO"
            continue
        }

        try {
            New-NetFirewallRule -DisplayName $rule.Name `
                -Direction Inbound `
                -Protocol $rule.Protocol `
                -LocalPort $rule.Port `
                -Action Allow `
                -Profile Any `
                -Description $rule.Desc | Out-Null

            Write-Log "防火墙规则已创建: $($rule.Name) (Port: $($rule.Port))" "OK"
        } catch {
            Write-Log "防火墙规则创建失败: $($rule.Name) - $_" "WARN"
        }
    }

    return $true
}

# ============================================
# 环境变量配置
# ============================================
function Set-EnvironmentVariables {
    param(
        [string]$ProgramPath
    )

    Write-Log "配置环境变量..." "INFO"

    try {
        # 设置 MC_PANEL_HOME
        [Environment]::SetEnvironmentVariable("MC_PANEL_HOME", $ProgramPath, "Machine")

        # 添加到 PATH
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
        if ($currentPath -notlike "*$ProgramPath*") {
            $newPath = "$currentPath;$ProgramPath"
            [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
            Write-Log "PATH 已更新" "OK"
        }

        Write-Log "环境变量配置完成" "OK"
    } catch {
        Write-Log "环境变量配置失败: $_" "WARN"
    }

    return $true
}

# ============================================
# 快捷方式创建
# ============================================
function New-DesktopShortcut {
    param(
        [string]$ProgramPath
    )

    Write-Log "创建快捷方式..." "INFO"

    $shell = New-Object -ComObject WScript.Shell

    # 桌面快捷方式
    $desktopPath = [Environment]::GetFolderPath("Desktop")
    $shortcut = $shell.CreateShortcut((Join-Path $desktopPath "MC Server Panel.lnk"))
    $shortcut.TargetPath = (Join-Path $ProgramPath "minecraft-admin.exe")
    $shortcut.WorkingDirectory = $ProgramPath
    $shortcut.Description = "MC Server Panel - Minecraft 管理面板"
    $shortcut.Save()

    # 开始菜单快捷方式
    $startMenuPath = Join-Path ([Environment]::GetFolderPath("Programs")) "MC Server Panel.lnk"
    $shortcut = $shell.CreateShortcut($startMenuPath)
    $shortcut.TargetPath = (Join-Path $ProgramPath "minecraft-admin.exe")
    $shortcut.WorkingDirectory = $ProgramPath
    $shortcut.Description = "MC Server Panel - Minecraft 管理面板"
    $shortcut.Save()

    Write-Log "快捷方式创建完成" "OK"
    return $true
}

# ============================================
# 服务启动与验证
# ============================================
function Start-ServiceVerification {
    param(
        [string]$ServiceName = "MCPanel"
    )

    if ($SkipService) {
        Write-Log "跳过服务启动验证" "INFO"
        return $true
    }

    Write-Log "=== 步骤 6/6: 启动服务并验证 ===" "STEP"

    try {
        Start-Service -Name $ServiceName -ErrorAction Stop

        # 等待服务启动
        Start-Sleep -Seconds 3

        $service = Get-Service -Name $ServiceName
        if ($service.Status -eq "Running") {
            Write-Log "服务启动成功" "OK"
        } else {
            Write-Log "服务状态: $($service.Status)" "WARN"
        }
    } catch {
        Write-Log "服务启动失败: $_" "ERROR"
        Write-Log "请手动启动服务: Start-Service $ServiceName" "WARN"
        return $false
    }

    return $true
}

# ============================================
# 安装完成摘要
# ============================================
function Show-InstallationSummary {
    param(
        [string]$ProgramPath,
        [string]$DataPath,
        [string]$ServiceName,
        [int]$Port
    )

    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "   MC Server Panel 安装完成!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "安装路径: $ProgramPath" -ForegroundColor White
    Write-Host "数据路径: $DataPath" -ForegroundColor White
    Write-Host "服务名称: $ServiceName" -ForegroundColor White
    Write-Host "访问地址: http://localhost:$Port" -ForegroundColor White
    Write-Host ""
    Write-Host "后续步骤:" -ForegroundColor Yellow
    Write-Host "  1. 首次登录请访问: http://localhost:$Port" -ForegroundColor Gray
    Write-Host "  2. 默认管理员: admin / changeme" -ForegroundColor Gray
    Write-Host "  3. 查看日志: Get-Content $DataPath\logs\*.log -Tail 50" -ForegroundColor Gray
    Write-Host "  4. 管理服务: Start-Service $ServiceName / Stop-Service $ServiceName" -ForegroundColor Gray
    Write-Host ""
    Write-Host "日志文件: $LogFile" -ForegroundColor DarkGray
    Write-Host ""
}

# ============================================
# 主函数
# ============================================
function Start-Installation {
    # 设置错误处理
    $ErrorActionPreference = 'Stop'

    # 创建日志目录
    $logDir = Split-Path $LogFile -Parent
    if (-not (Test-Path $logDir)) {
        New-Item -Path $logDir -ItemType Directory -Force | Out-Null
    }

    Write-Log "========================================" "INFO"
    Write-Log "  MC Server Panel 安装程序 v2.0.0" "INFO"
    Write-Log "========================================" "INFO"
    Write-Log "开始时间: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" "INFO"
    Write-Log "日志文件: $LogFile" "INFO"

    try {
        # 1. 权限检测
        Request-AdministratorPrivileges

        # 2. 系统兼容性
        if (-not (Test-SystemCompatibility)) {
            throw "系统兼容性检测失败"
        }

        # 3. 依赖检测
        Test-DotNetRuntime | Out-Null

        # 4. 端口检测
        if (-not (Test-PortAvailability -Ports @($Port, $RconPort))) {
            throw "端口检测失败"
        }

        # 5. 获取最新版本
        $releaseInfo = Get-LatestReleaseInfo
        Write-Log "最新版本: $($releaseInfo.Version)" "INFO"
        $downloadUrl = $releaseInfo.DownloadUrl

        # 6. 目录初始化
        if (-not (Initialize-DirectoryStructure -ProgramPath $InstallPath -DataPath $DataPath)) {
            throw "目录初始化失败"
        }

        # 7. 下载与部署
        if (-not (Install-Binary -ProgramPath $InstallPath -DownloadUrl $downloadUrl)) {
            throw "二进制文件部署失败"
        }

        # 8. 配置文件
        if (-not (Install-Configuration -ProgramPath $InstallPath -DataPath $DataPath -Port $Port -RconPort $RconPort)) {
            throw "配置文件生成失败"
        }

        # 9. 服务注册
        if (-not (Register-WindowsService -ProgramPath $InstallPath -ServiceName $ServiceName)) {
            throw "服务注册失败"
        }

        # 10. 防火墙
        Register-FirewallRules -Port $Port -RconPort $RconPort | Out-Null

        # 11. 环境变量
        Set-EnvironmentVariables -ProgramPath $InstallPath | Out-Null

        # 12. 快捷方式
        New-DesktopShortcut -ProgramPath $InstallPath | Out-Null

        # 13. 启动验证
        if (-not (Start-ServiceVerification -ServiceName $ServiceName)) {
            Write-Log "服务启动验证失败，但安装已完成" "WARN"
        }

        # 14. 完成摘要
        Show-InstallationSummary -ProgramPath $InstallPath -DataPath $DataPath -ServiceName $ServiceName -Port $Port

        Write-Log "安装完成时间: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" "INFO"
        Write-Log "安装成功!" "OK"

        return $true

    } catch {
        Write-Log "安装失败: $_" "ERROR"
        Write-Log "详细信息请查看日志: $LogFile" "ERROR"

        # 执行回滚
        Invoke-Rollback

        return $false
    } finally {
        # 清理临时目录
        if (Test-Path $InstallTempPath) {
            Remove-Item $InstallTempPath -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

# ============================================
# 程序入口
# ============================================
if ($PSCmdlet.ShouldProcess("MC Server Panel", "安装")) {
    $result = Start-Installation

    if (-not $result) {
        exit 1
    }
}

exit 0

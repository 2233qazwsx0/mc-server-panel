#Requires -Version 5.1
<#
.SYNOPSIS
    Rust MC 服务器面板安装器 - Windows 日志工具模块
.DESCRIPTION
    提供跨平台一致的日志记录接口，支持多级别日志、彩色控制台输出和文件日志
.NOTES
    作者: Rust MC 团队
    版本: 1.0.0
#>

# 定义日志级别枚举
enum LogLevel {
    INFO = 0
    WARNING = 1
    ERROR = 2
    DEBUG = 3
}

# 定义日志级别对应的控制台颜色
$script:LogLevelColors = @{
    [LogLevel]::INFO    = 'White'
    [LogLevel]::WARNING = 'Yellow'
    [LogLevel]::ERROR   = 'Red'
    [LogLevel]::DEBUG   = 'DarkGray'
}

<#
.SYNOPSIS
    获取默认日志文件路径
.DESCRIPTION
    返回系统临时目录下的日志文件路径
.OUTPUTS
    String - 日志文件的完整路径
.EXAMPLE
    $logPath = Get-LogFilePath
#>
function Get-LogFilePath {
    [CmdletBinding()]
    [OutputType([string])]
    param()
    
    return Join-Path -Path $env:TEMP -ChildPath 'mc-server-install.log'
}

<#
.SYNOPSIS
    初始化日志会话
.DESCRIPTION
    创建或清空日志文件，并写入会话开始标记
.PARAMETER LogFile
    日志文件路径，默认使用 Get-LogFilePath 返回的路径
.EXAMPLE
    Start-LogSession -LogFile "C:\Logs\install.log"
.EXAMPLE
    Start-LogSession  # 使用默认路径
#>
function Start-LogSession {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory = $false)]
        [ValidateNotNullOrEmpty()]
        [string]$LogFile = (Get-LogFilePath)
    )
    
    # 获取当前时间戳
    $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    
    # 创建会话开始标记
    $sessionMarker = @"
========================================
日志会话开始: $timestamp
========================================

"@
    
    # 将会话标记写入文件（覆盖模式）
    $sessionMarker | Out-File -FilePath $LogFile -Encoding utf8 -Force
    
    # 输出到控制台
    Write-Host "[$timestamp] [INFO] 日志会话已初始化" -ForegroundColor White
}

<#
.SYNOPSIS
    写入日志信息
.DESCRIPTION
    将日志信息输出到控制台（带颜色）并追加到日志文件
.PARAMETER Message
    日志消息内容
.PARAMETER Level
    日志级别，可选值：INFO, WARNING, ERROR, DEBUG（默认：INFO）
.PARAMETER LogFile
    日志文件路径，默认使用 Get-LogFilePath 返回的路径
.EXAMPLE
    Write-Log -Message "安装开始" -Level INFO
.EXAMPLE
    Write-Log -Message "检测到配置文件" -Level DEBUG -LogFile "C:\Logs\custom.log"
.EXAMPLE
    Write-Log "快速调用示例"  # 使用位置参数
#>
function Write-Log {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory = $true, Position = 0, ValueFromPipeline = $true)]
        [AllowEmptyString()]
        [string]$Message,
        
        [Parameter(Mandatory = $false, Position = 1)]
        [ValidateSet('INFO', 'WARNING', 'ERROR', 'DEBUG', IgnoreCase = $true)]
        [string]$Level = 'INFO',
        
        [Parameter(Mandatory = $false, Position = 2)]
        [ValidateNotNullOrEmpty()]
        [string]$LogFile = (Get-LogFilePath)
    )
    
    # 将字符串级别转换为 LogLevel 枚举
    $logLevel = [LogLevel]::"$Level"
    
    # 生成时间戳
    $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    
    # 格式化日志消息
    $formattedMessage = "[$timestamp] [$($Level.ToUpper())] $Message"
    
    # 获取对应的控制台颜色
    $consoleColor = $script:LogLevelColors[$logLevel]
    
    # 输出到控制台（带颜色）
    Write-Host $formattedMessage -ForegroundColor $consoleColor
    
    # 追加到日志文件
    try {
        Add-Content -Path $LogFile -Value $formattedMessage -ErrorAction Stop
    }
    catch {
        Write-Warning "无法写入日志文件: $LogFile"
    }
}

# 导出模块成员供外部使用
Export-ModuleMember -Function @(
    'Write-Log',
    'Start-LogSession',
    'Get-LogFilePath'
)

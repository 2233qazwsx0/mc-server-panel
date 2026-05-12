#Requires -Version 5.1
<#
.SYNOPSIS
    Rust MC 服务器面板安装器 - Windows 注册表和环境变量操作工具模块
.DESCRIPTION
    提供 Windows 注册表操作和环境变量管理的封装功能，
    支持 PATH 环境变量修改、注册表键值操作等
.NOTES
    作者: Rust MC 团队
    版本: 1.0.0
    依赖: PowerShell 5.1+
#>

$ErrorActionPreference = 'Stop'

if (-not (Test-Path -Path Variable:Script:LoggerLoaded)) {
    try {
        Import-Module (Join-Path -Path $PSScriptRoot -ChildPath '..\common\logger.ps1' -Resolve) -ErrorAction Stop
        $Script:LoggerLoaded = $true
    } catch {
        $Script:LoggerLoaded = $false
    }
}

function Add-PathToEnvironment {
    <#
    .SYNOPSIS
        将路径添加到系统或用户环境变量 PATH
    .DESCRIPTION
        获取指定作用域的当前 PATH 环境变量，检查目标路径是否已存在，
        如果不存在则追加到 PATH 末尾。PATH 变量用于指定可执行文件的搜索路径。
    .PARAMETER Path
        要添加到 PATH 的目录路径
    .PARAMETER Scope
        环境变量的作用域，可选值：'Machine'（系统级）、'User'（用户级）
        默认为 'User'。Machine 需要管理员权限。
    .OUTPUTS
        Boolean - 添加成功返回 True，已存在或失败返回 False
    .EXAMPLE
        Add-PathToEnvironment -Path "C:\Program Files\MyApp" -Scope "User"
    .EXAMPLE
        Add-PathToEnvironment -Path "C:\Tools\bin" -Scope "Machine"
    .EXAMPLE
        $env:INSTALL_DIR = "C:\MyApp"
        Add-PathToEnvironment -Path "$env:INSTALL_DIR\bin"
    .NOTES
        Machine 作用域需要管理员权限。添加后新开终端窗口即可生效。
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$Path,

        [Parameter(Mandatory = $false, Position = 1)]
        [ValidateSet('Machine', 'User', IgnoreCase = $true)]
        [string]$Scope = 'User'
    )

    try {
        $targetScope = [EnvironmentVariableTarget]$Scope

        $currentPath = [Environment]::GetEnvironmentVariable("Path", $targetScope)

        if ([string]::IsNullOrEmpty($currentPath)) {
            $currentPath = ""
        }

        $pathParts = $currentPath -split ';' | ForEach-Object { $_.Trim() } | Where-Object { -not [string]::IsNullOrEmpty($_) }

        $normalizedPath = $Path.TrimEnd('\')
        $pathExists = $false

        foreach ($existingPath in $pathParts) {
            $normalizedExisting = $existingPath.TrimEnd('\')
            if ($normalizedExisting -eq $normalizedPath) {
                $pathExists = $true
                break
            }
        }

        if ($pathExists) {
            Write-Verbose "路径 '$Path' 已存在于 $Scope PATH 中"
            return $false
        }

        if ([string]::IsNullOrEmpty($currentPath)) {
            $newPath = $Path
        } else {
            $newPath = "$currentPath;$Path"
        }

        [Environment]::SetEnvironmentVariable("Path", $newPath, $targetScope)

        Write-Host "已将路径添加到 $Scope PATH: $Path" -ForegroundColor Green
        Write-Host "注意：需要重新打开终端窗口使更改生效" -ForegroundColor Cyan
        return $true

    } catch {
        Write-Warning "添加路径到环境变量时发生错误: $_"
        return $false
    }
}

function Test-PathInEnvironment {
    <#
    .SYNOPSIS
        检查路径是否已存在于环境变量 PATH
    .DESCRIPTION
        获取指定作用域的 PATH 环境变量，规范化路径格式后进行比对，
        检查目标路径是否已存在于 PATH 中。
    .PARAMETER Path
        要检查的目录路径
    .PARAMETER Scope
        环境变量的作用域，可选值：'Machine'（系统级）、'User'（用户级）
        默认为 'User'
    .OUTPUTS
        Boolean - 路径存在返回 True，否则返回 False
    .EXAMPLE
        if (Test-PathInEnvironment -Path "C:\Program Files\MyApp") {
            Write-Host "Path is already in environment"
        }
    .EXAMPLE
        $hasJava = Test-PathInEnvironment -Path "C:\Program Files\Java\jdk-17\bin"
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$Path,

        [Parameter(Mandatory = $false, Position = 1)]
        [ValidateSet('Machine', 'User', IgnoreCase = $true)]
        [string]$Scope = 'User'
    )

    try {
        $targetScope = [EnvironmentVariableTarget]$Scope

        $currentPath = [Environment]::GetEnvironmentVariable("Path", $targetScope)

        if ([string]::IsNullOrEmpty($currentPath)) {
            return $false
        }

        $pathParts = $currentPath -split ';' | ForEach-Object { $_.Trim() } | Where-Object { -not [string]::IsNullOrEmpty($_) }

        $normalizedPath = $Path.TrimEnd('\')

        foreach ($existingPath in $pathParts) {
            $normalizedExisting = $existingPath.TrimEnd('\')
            if ($normalizedExisting -eq $normalizedPath) {
                Write-Verbose "路径 '$Path' 存在于 $Scope PATH 中"
                return $true
            }
        }

        Write-Verbose "路径 '$Path' 不存在于 $Scope PATH 中"
        return $false

    } catch {
        Write-Warning "检查路径时发生错误: $_"
        return $false
    }
}

function Set-RegistryValue {
    <#
    .SYNOPSIS
        设置注册表值
    .DESCRIPTION
        创建指定的注册表键（如不存在），并设置其值。
        支持多种注册表值类型，包括字符串、可扩展字符串、DWORD 等。
    .PARAMETER Path
        注册表路径，例如 'HKCU:\Software\MyApp'
    .PARAMETER Name
        要设置的值的名称
    .PARAMETER Value
        要设置的值
    .PARAMETER Type
        注册表值类型，可选值：
        - String: 字符串值
        - ExpandString: 可扩展字符串（包含环境变量）
        - DWord: 32 位整数
        - QWord: 64 位整数
        - Binary: 二进制数据
        - MultiString: 多字符串
        默认为 'String'
    .OUTPUTS
        Boolean - 设置成功返回 True，失败返回 False
    .EXAMPLE
        Set-RegistryValue -Path "HKCU:\Software\MyApp" -Name "InstallPath" -Value "C:\MyApp"
    .EXAMPLE
        Set-RegistryValue -Path "HKLM:\Software\MyService" -Name "Enabled" -Value 1 -Type "DWord"
    .EXAMPLE
        $path = "HKCU:\Software\MCServer"
        Set-RegistryValue -Path $path -Name "DataDir" -Value "%APPDATA%\MCServer" -Type "ExpandString"
    .NOTES
        修改 HKLM（本地计算机）需要管理员权限。
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$Path,

        [Parameter(Mandatory = $true, Position = 1)]
        [ValidateNotNullOrEmpty()]
        [string]$Name,

        [Parameter(Mandatory = $true, Position = 2)]
        [AllowNull()]
        $Value,

        [Parameter(Mandatory = $false, Position = 3)]
        [ValidateSet('String', 'ExpandString', 'DWord', 'QWord', 'Binary', 'MultiString', IgnoreCase = $true)]
        [string]$Type = 'String'
    )

    try {
        $registryType = switch ($Type.ToLower()) {
            'string'      { [Microsoft.Win32.RegistryValueKind]::String }
            'expandstring'{ [Microsoft.Win32.RegistryValueKind]::ExpandString }
            'dword'       { [Microsoft.Win32.RegistryValueKind]::DWord }
            'qword'       { [Microsoft.Win32.RegistryValueKind]::QWord }
            'binary'      { [Microsoft.Win32.RegistryValueKind]::Binary }
            'multistring' { [Microsoft.Win32.RegistryValueKind]::MultiString }
            default       { [Microsoft.Win32.RegistryValueKind]::String }
        }

        if (-not (Test-PathInEnvironment -Path $Path -Scope 'Machine' -ErrorAction SilentlyContinue)) {
            $keyExists = Test-RegistryKeyExists -Path $Path
            if (-not $keyExists) {
                $null = New-Item -Path $Path -Force -ErrorAction Stop
                Write-Verbose "创建注册表键: $Path"
            }
        } else {
            $keyExists = Test-RegistryKeyExists -Path $Path
            if (-not $keyExists) {
                $null = New-Item -Path $Path -Force -ErrorAction Stop
                Write-Verbose "创建注册表键: $Path"
            }
        }

        if (-not (Test-RegistryKeyExists -Path $Path)) {
            $null = New-Item -Path $Path -Force -ErrorAction Stop
            Write-Verbose "创建注册表键: $Path"
        }

        Set-ItemProperty -Path $Path -Name $Name -Value $Value -Type $registryType -ErrorAction Stop

        Write-Verbose "成功设置注册表值: $Path\$Name = $Value (类型: $Type)"
        return $true

    } catch {
        Write-Warning "设置注册表值时发生错误: $_"
        return $false
    }
}

function Test-RegistryKeyExists {
    <#
    .SYNOPSIS
        检查注册表项是否存在
    .DESCRIPTION
        使用 Test-Path 检查指定的注册表路径是否存在。
        支持所有标准的注册表根键。
    .PARAMETER Path
        注册表路径，例如 'HKCU:\Software\MyApp' 或 'HKLM:\Software\MyService'
    .OUTPUTS
        Boolean - 键存在返回 True，否则返回 False
    .EXAMPLE
        if (Test-RegistryKeyExists -Path "HKCU:\Software\MCServer") {
            Write-Host "Application registry key exists"
        }
    .EXAMPLE
        $exists = Test-RegistryKeyExists -Path "HKLM:\Software\Microsoft\Windows\CurrentVersion\Run"
    .NOTES
        HKCU = HKEY_CURRENT_USER, HKLM = HKEY_LOCAL_MACHINE
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$Path
    )

    try {
        $exists = Test-Path -Path $Path -ErrorAction SilentlyContinue
        return $exists
    } catch {
        Write-Verbose "检查注册表键时发生错误: $_"
        return $false
    }
}

function Remove-PathFromEnvironment {
    <#
    .SYNOPSIS
        从环境变量 PATH 中移除指定路径
    .DESCRIPTION
        获取指定作用域的当前 PATH 环境变量，移除目标路径（如果存在），
        并更新环境变量。
    .PARAMETER Path
        要从 PATH 移除的目录路径
    .PARAMETER Scope
        环境变量的作用域，可选值：'Machine'、'User'
        默认为 'User'
    .OUTPUTS
        Boolean - 移除成功返回 True，路径不存在或失败返回 False
    .EXAMPLE
        Remove-PathFromEnvironment -Path "C:\OldApp\bin" -Scope "User"
    .NOTES
        Machine 作用域需要管理员权限。
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$Path,

        [Parameter(Mandatory = $false, Position = 1)]
        [ValidateSet('Machine', 'User', IgnoreCase = $true)]
        [string]$Scope = 'User'
    )

    try {
        $targetScope = [EnvironmentVariableTarget]$Scope

        $currentPath = [Environment]::GetEnvironmentVariable("Path", $targetScope)

        if ([string]::IsNullOrEmpty($currentPath)) {
            Write-Verbose "PATH 为空，无需移除"
            return $false
        }

        $pathParts = $currentPath -split ';' | ForEach-Object { $_.Trim() } | Where-Object { -not [string]::IsNullOrEmpty($_) }

        $normalizedPath = $Path.TrimEnd('\')
        $found = $false
        $newPathParts = @()

        foreach ($existingPath in $pathParts) {
            $normalizedExisting = $existingPath.TrimEnd('\')
            if ($normalizedExisting -eq $normalizedPath) {
                $found = $true
            } else {
                $newPathParts += $existingPath
            }
        }

        if (-not $found) {
            Write-Verbose "路径 '$Path' 不存在于 $Scope PATH 中"
            return $false
        }

        $newPath = $newPathParts -join ';'
        [Environment]::SetEnvironmentVariable("Path", $newPath, $targetScope)

        Write-Host "已从 $Scope PATH 移除路径: $Path" -ForegroundColor Green
        return $true

    } catch {
        Write-Warning "从环境变量移除路径时发生错误: $_"
        return $false
    }
}

function Get-EnvironmentPath {
    <#
    .SYNOPSIS
        获取指定作用域的 PATH 环境变量
    .DESCRIPTION
        返回指定作用域的完整 PATH 环境变量值，可用于备份或调试。
    .PARAMETER Scope
        环境变量的作用域，可选值：'Machine'、'User'、'Process'
        默认为 'User'
    .OUTPUTS
        String - PATH 环境变量的值
    .EXAMPLE
        $userPath = Get-EnvironmentPath -Scope "User"
        $machinePath = Get-EnvironmentPath -Scope "Machine"
    #>
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory = $false, Position = 0)]
        [ValidateSet('Machine', 'User', 'Process', IgnoreCase = $true)]
        [string]$Scope = 'User'
    )

    try {
        $targetScope = [EnvironmentVariableTarget]$Scope
        $path = [Environment]::GetEnvironmentVariable("Path", $targetScope)
        return if ([string]::IsNullOrEmpty($path)) { "" } else { $path }
    } catch {
        Write-Warning "获取 PATH 环境变量时发生错误: $_"
        return ""
    }
}

Export-ModuleMember -Function @(
    'Add-PathToEnvironment',
    'Test-PathInEnvironment',
    'Set-RegistryValue',
    'Test-RegistryKeyExists',
    'Remove-PathFromEnvironment',
    'Get-EnvironmentPath'
)

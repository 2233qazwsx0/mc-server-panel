#Requires -Version 5.1
<#
.SYNOPSIS
    Rust MC 服务器面板安装器 - winget 包管理器工具模块
.DESCRIPTION
    提供 winget 包管理器的封装功能，支持包检测、安装和列表查询
.NOTES
    作者: Rust MC 团队
    版本: 1.0.0
    依赖: Windows Package Manager (winget)
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

function Test-Winget {
    <#
    .SYNOPSIS
        检查 winget 是否可用
    .DESCRIPTION
        验证 Windows Package Manager (winget) 命令是否存在于系统 PATH 中。
        用于在执行安装操作前确认包管理器可用。
    .OUTPUTS
        Boolean - winget 可用返回 True，否则返回 False
    .EXAMPLE
        if (Test-Winget) {
            Write-Host "winget is available"
        }
    .EXAMPLE
        $isWingetAvailable = Test-Winget
        if (-not $isWingetAvailable) {
            Write-Warning "winget is not installed. Please install it from Microsoft Store."
        }
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param()

    try {
        $null = Get-Command winget -ErrorAction Stop
        Write-Verbose "winget is available on this system"
        return $true
    } catch {
        Write-Verbose "winget is not available: $_"
        return $false
    }
}

function Install-PackageWithWinget {
    <#
    .SYNOPSIS
        使用 winget 安装指定的包
    .DESCRIPTION
        通过 Windows Package Manager 安装指定的应用程序包。
        自动接受包协议和源协议，减少交互提示。
    .PARAMETER PackageId
        要安装的包的唯一标识符 (Package Identifier)
    .PARAMETER AcceptAgreements
        是否自动接受包和源的协议条款，默认值为 True
    .OUTPUTS
        Boolean - 安装成功返回 True，失败返回 False
    .EXAMPLE
        Install-PackageWithWinget -PackageId "Microsoft.VCRedist.2015-2022.x64"
    .EXAMPLE
        $result = Install-PackageWithWinget -PackageId "Git.Git" -AcceptAgreements $true
        if ($result) {
            Write-Host "Package installed successfully"
        }
    .NOTES
        此函数会阻塞直到安装完成。安装过程可能需要几分钟时间。
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$PackageId,

        [Parameter(Mandatory = $false, Position = 1)]
        [bool]$AcceptAgreements = $true
    )

    try {
        if (-not (Test-Winget)) {
            Write-Warning "winget 不可用，无法安装包 '$PackageId'"
            Write-Warning "请确保已安装 Windows Package Manager (Microsoft Store)"
            return $false
        }

        Write-Host "正在安装包: $PackageId" -ForegroundColor Cyan

        $argumentList = @(
            "install",
            "--id", $PackageId,
            "-e",
            "--accept-package-agreements"
        )

        if ($AcceptAgreements) {
            $argumentList += "--accept-source-agreements"
        }

        $processParams = @{
            FilePath     = "winget"
            ArgumentList = ($argumentList -join " ")
            NoNewWindow  = $true
            Wait         = $true
            PassThru     = $true
            ErrorAction  = "Stop"
        }

        $installResult = Start-Process @processParams
        $exitCode = $installResult.ExitCode

        if ($exitCode -eq 0) {
            Write-Host "包 '$PackageId' 安装成功" -ForegroundColor Green
            return $true
        } elseif ($exitCode -eq 2316639751 -or $exitCode -eq -1978335185) {
            Write-Warning "包 '$PackageId' 已安装或不需要安装"
            return $true
        } else {
            Write-Warning "包 '$PackageId' 安装失败，退出码: $exitCode"
            return $false
        }
    } catch {
        Write-Warning "安装包 '$PackageId' 时发生错误: $_"
        return $false
    }
}

function Test-PackageInstalled {
    <#
    .SYNOPSIS
        检查指定包是否已安装
    .DESCRIPTION
        使用 winget list 命令查询系统中已安装的包列表，
        判断指定的包标识符是否存在于安装列表中。
    .PARAMETER PackageId
        要检查的包的唯一标识符
    .OUTPUTS
        Boolean - 包已安装返回 True，否则返回 False
    .EXAMPLE
        if (Test-PackageInstalled -PackageId "Microsoft.VCRedist.2015-2022.x64") {
            Write-Host "Visual C++ Redistributable is already installed"
        }
    .EXAMPLE
        $hasJava = Test-PackageInstalled -PackageId "Oracle.JDK.17"
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$PackageId
    )

    try {
        if (-not (Test-Winget)) {
            Write-Warning "winget 不可用，无法检查包状态"
            return $false
        }

        $argumentList = @(
            "list",
            "--id", $PackageId,
            "-e"
        )

        $processParams = @{
            FilePath     = "winget"
            ArgumentList = ($argumentList -join " ")
            NoNewWindow  = $true
            Wait         = $true
            PassThru     = $true
            ErrorAction  = "SilentlyContinue"
        }

        $result = Start-Process @processParams
        $exitCode = $result.ExitCode

        if ($exitCode -eq 0) {
            Write-Verbose "包 '$PackageId' 已安装"
            return $true
        } else {
            Write-Verbose "包 '$PackageId' 未安装"
            return $false
        }
    } catch {
        Write-Warning "检查包 '$PackageId' 状态时发生错误: $_"
        return $false
    }
}

function Get-InstalledPackages {
    <#
    .SYNOPSIS
        列出所有通过 winget 安装的包
    .DESCRIPTION
        执行 winget list 命令并解析输出，返回所有已安装包的列表。
        返回的包对象包含 Id、Version、Source 等属性。
    .OUTPUTS
        Array of package objects - 返回包对象数组，每个对象包含包的相关信息
    .EXAMPLE
        $packages = Get-InstalledPackages
        foreach ($pkg in $packages) {
            Write-Host "$($pkg.Id) - $($pkg.Version)"
        }
    .EXAMPLE
        $packages = Get-InstalledPackages | Where-Object { $_.Source -eq "winget" }
    .NOTES
        返回的数据仅包含通过 winget 安装的包，不包括其他方式安装的应用程序。
    #>
    [CmdletBinding()]
    [OutputType([array])]
    param()

    try {
        if (-not (Test-Winget)) {
            Write-Warning "winget 不可用，无法获取包列表"
            return @()
        }

        $argumentList = @(
            "list"
        )

        $processParams = @{
            FilePath     = "winget"
            ArgumentList = ($argumentList -join " ")
            NoNewWindow  = $true
            Wait         = $true
            PassThru     = $true
            RedirectStandardOutput = "$env:TEMP\winget_list_$PID.log"
            RedirectStandardError  = "$env:TEMP\winget_list_err_$PID.log"
            ErrorAction  = "SilentlyContinue"
        }

        $null = Start-Process @processParams

        $outputFile = "$env:TEMP\winget_list_$PID.log"

        if (Test-Path $outputFile) {
            $content = Get-Content $outputFile -Raw
            $packages = @()

            $lines = $content -split "`r?`n"

            $startParsing = $false
            foreach ($line in $lines) {
                if ($line -match "^-+\s+-+") {
                    $startParsing = $true
                    continue
                }

                if ($startParsing -and $line -match "^\S") {
                    $parts = $line -split '\s{2,}'
                    if ($parts.Count -ge 3) {
                        $packages += [PSCustomObject]@{
                            Id      = $parts[0].Trim()
                            Version = $parts[1].Trim()
                            Source  = $parts[2].Trim()
                        }
                    }
                }
            }

            Remove-Item $outputFile -Force -ErrorAction SilentlyContinue

            $errFile = "$env:TEMP\winget_list_err_$PID.log"
            Remove-Item $errFile -Force -ErrorAction SilentlyContinue

            return $packages
        }

        return @()
    } catch {
        Write-Warning "获取已安装包列表时发生错误: $_"
        return @()
    }
}

function Install-RequiredDependencies {
    <#
    .SYNOPSIS
        安装 MC 服务器面板所需的系统依赖
    .DESCRIPTION
        根据预定义的依赖列表，自动安装运行 MC 服务器面板所需的各种系统组件，
        包括 Visual C++ Redistributable、Java 运行时等。
    .PARAMETER Dependencies
        要安装的依赖包标识符数组，默认值为预定义的必需依赖列表
    .OUTPUTS
        Boolean - 所有依赖安装成功返回 True，否则返回 False
    .EXAMPLE
        Install-RequiredDependencies
    .EXAMPLE
        $deps = @("Microsoft.VCRedist.2015-2022.x64", "Oracle.JDK.17")
        Install-RequiredDependencies -Dependencies $deps
    .NOTES
        默认依赖列表包含 Visual C++ Redistributable 2015-2022。
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $false)]
        [string[]]$Dependencies = @("Microsoft.VCRedist.2015-2022.x64")
    )

    $allSuccess = $true

    Write-Host "开始安装系统依赖..." -ForegroundColor Cyan

    foreach ($packageId in $Dependencies) {
        if (Test-PackageInstalled -PackageId $packageId) {
            Write-Host "依赖 '$packageId' 已安装，跳过" -ForegroundColor Gray
            continue
        }

        Write-Host "正在安装依赖: $packageId" -ForegroundColor Yellow
        $result = Install-PackageWithWinget -PackageId $packageId

        if (-not $result) {
            Write-Warning "依赖 '$packageId' 安装失败"
            $allSuccess = $false
        }
    }

    if ($allSuccess) {
        Write-Host "所有依赖安装完成" -ForegroundColor Green
    } else {
        Write-Warning "部分依赖安装失败，请检查日志"
    }

    return $allSuccess
}

Export-ModuleMember -Function @(
    'Test-Winget',
    'Install-PackageWithWinget',
    'Test-PackageInstalled',
    'Get-InstalledPackages',
    'Install-RequiredDependencies'
)

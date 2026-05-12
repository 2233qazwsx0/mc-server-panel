#Requires -Version 5.1
<#
.SYNOPSIS
    Rust MC 服务器面板安装器 - Windows 桌面集成模块
.DESCRIPTION
    提供 Windows 桌面快捷方式、开始菜单和注册表卸载入口功能
.NOTES
    作者: MC Server Team
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

function Create-Shortcut {
    <#
    .SYNOPSIS
        创建 Windows 快捷方式
    .DESCRIPTION
        使用 COM 对象 WScript.Shell 创建 Windows 快捷方式 (.lnk)
        支持设置目标路径、描述、图标和工作目录
    .PARAMETER TargetPath
        快捷方式指向的目标程序路径
    .PARAMETER ShortcutPath
        快捷方式文件的保存路径（包含 .lnk 扩展名）
    .PARAMETER Description
        快捷方式的描述文字
    .PARAMETER IconLocation
        快捷方式图标的路径（格式：程序路径,图标索引）
    .OUTPUTS
        Boolean - 创建成功返回 True，失败返回 False
    .EXAMPLE
        Create-Shortcut -TargetPath "C:\Program Files\App\app.exe" -ShortcutPath "$env:USERPROFILE\Desktop\App.lnk"
    .EXAMPLE
        Create-Shortcut -TargetPath "C:\App\server.exe" -ShortcutPath "$env:USERPROFILE\Desktop\Server.lnk" -Description "Minecraft 服务器管理面板" -IconLocation "C:\App\server.exe,0"
    .NOTES
        需要目标文件存在才能成功创建快捷方式
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$TargetPath,

        [Parameter(Mandatory = $true, Position = 1)]
        [ValidateNotNullOrEmpty()]
        [string]$ShortcutPath,

        [Parameter(Mandatory = $false, Position = 2)]
        [AllowEmptyString()]
        [string]$Description = "",

        [Parameter(Mandatory = $false, Position = 3)]
        [AllowEmptyString()]
        [string]$IconLocation = ""
    )

    try {
        if (-not (Test-Path -LiteralPath $TargetPath)) {
            Write-Warning "目标路径不存在: $TargetPath"
            return $false
        }

        $shortcutDirectory = Split-Path -Path $ShortcutPath -Parent
        if (-not (Test-Path -LiteralPath $shortcutDirectory)) {
            $null = New-Item -ItemType Directory -Path $shortcutDirectory -Force -ErrorAction Stop
        }

        $wshell = New-Object -ComObject WScript.Shell -ErrorAction Stop

        $shortcut = $wshell.CreateShortcut($ShortcutPath)
        $shortcut.TargetPath = $TargetPath
        $shortcut.Description = $Description

        if (-not [string]::IsNullOrEmpty($IconLocation)) {
            $shortcut.IconLocation = $IconLocation
        }

        $workingDir = Split-Path -Path $TargetPath -Parent
        if (-not [string]::IsNullOrEmpty($workingDir)) {
            $shortcut.WorkingDirectory = $workingDir
        }

        $shortcut.Save()

        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($wshell) | Out-Null
        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()

        Write-Host "已创建快捷方式: $ShortcutPath" -ForegroundColor Green
        return $true

    } catch {
        Write-Warning "创建快捷方式时发生错误: $_"
        return $false
    }
}

function Install-DesktopIntegration {
    <#
    .SYNOPSIS
        安装 Windows 桌面集成（快捷方式和注册表入口）
    .DESCRIPTION
        创建桌面快捷方式、开始菜单快捷方式，并注册卸载程序入口到注册表
        返回创建的项列表用于回滚
    .PARAMETER InstallPath
        应用程序安装目录
    .PARAMETER AppName
        应用程序名称
    .PARAMETER AppVersion
        应用程序版本
    .PARAMETER UninstallScript
        卸载脚本的路径
    .PARAMETER CreateDesktopShortcut
        是否创建桌面快捷方式（默认 True）
    .PARAMETER CreateStartMenuShortcut
        是否创建开始菜单快捷方式（默认 True）
    .PARAMETER CreateUninstallEntry
        是否创建注册表卸载入口（默认 True）
    .OUTPUTS
        Hashtable - 包含所有创建的项，用于回滚
    .EXAMPLE
        $items = Install-DesktopIntegration -InstallPath "C:\Users\You\AppData\Local\MC Server Panel" -AppVersion "1.0.0"
    .NOTES
        此函数需要在安装程序完成后调用
    #>
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [ValidateNotNullOrEmpty()]
        [string]$InstallPath,

        [Parameter(Mandatory = $false, Position = 1)]
        [string]$AppName = "MC Server Panel",

        [Parameter(Mandatory = $false, Position = 2)]
        [string]$AppVersion = "1.0.0",

        [Parameter(Mandatory = $false, Position = 3)]
        [ValidateNotNullOrEmpty()]
        [string]$UninstallScript = "",

        [Parameter(Mandatory = $false, Position = 4)]
        [bool]$CreateDesktopShortcut = $true,

        [Parameter(Mandatory = $false, Position = 5)]
        [bool]$CreateStartMenuShortcut = $true,

        [Parameter(Mandatory = $false, Position = 6)]
        [bool]$CreateUninstallEntry = $true
    )

    $createdItems = @{
        DesktopShortcuts = @()
        StartMenuShortcuts = @()
        RegistryKeys = @()
        Success = $true
    }

    try {
        $binPath = Join-Path -Path $InstallPath -ChildPath "bin"
        $executablePath = Join-Path -Path $binPath -ChildPath "mc-server.exe"

        if (-not (Test-Path -LiteralPath $executablePath)) {
            Write-Warning "可执行文件不存在: $executablePath"
            $createdItems.Success = $false
            return $createdItems
        }

        if ($CreateDesktopShortcut) {
            Write-Host "正在创建桌面快捷方式..." -ForegroundColor Cyan

            $desktopPath = [Environment]::GetFolderPath("Desktop")
            $desktopShortcutPath = Join-Path -Path $desktopPath -ChildPath "$AppName.lnk"

            $result = Create-Shortcut `
                -TargetPath $executablePath `
                -ShortcutPath $desktopShortcutPath `
                -Description "Minecraft 服务器管理面板" `
                -IconLocation "$executablePath,0"

            if ($result) {
                $createdItems.DesktopShortcuts += $desktopShortcutPath
                Write-Host "桌面快捷方式已创建: $desktopShortcutPath" -ForegroundColor Green
            } else {
                Write-Warning "桌面快捷方式创建失败"
            }
        }

        if ($CreateStartMenuShortcut) {
            Write-Host "正在创建开始菜单快捷方式..." -ForegroundColor Cyan

            $startMenuPath = [Environment]::GetFolderPath("StartMenu")
            $programsPath = Join-Path -Path $startMenuPath -ChildPath "Programs"
            $appFolderPath = Join-Path -Path $programsPath -ChildPath $AppName

            $startMenuShortcutPath = Join-Path -Path $appFolderPath -ChildPath "$AppName.lnk"

            $result = Create-Shortcut `
                -TargetPath $executablePath `
                -ShortcutPath $startMenuShortcutPath `
                -Description "Minecraft 服务器管理面板" `
                -IconLocation "$executablePath,0"

            if ($result) {
                $createdItems.StartMenuShortcuts += $startMenuShortcutPath

                $uninstallShortcutPath = Join-Path -Path $appFolderPath -ChildPath "卸载 $AppName.lnk"
                if (-not [string]::IsNullOrEmpty($UninstallScript) -and (Test-Path -LiteralPath $UninstallScript)) {
                    $uninstallResult = Create-Shortcut `
                        -TargetPath "powershell.exe" `
                        -ShortcutPath $uninstallShortcutPath `
                        -Description "卸载 $AppName" `
                        -IconLocation "shell32.dll,40"

                    if ($uninstallResult) {
                        $createdItems.StartMenuShortcuts += $uninstallShortcutPath
                    }
                }

                Write-Host "开始菜单快捷方式已创建: $startMenuShortcutPath" -ForegroundColor Green
            } else {
                Write-Warning "开始菜单快捷方式创建失败"
            }
        }

        if ($CreateUninstallEntry) {
            Write-Host "正在注册卸载程序入口..." -ForegroundColor Cyan

            $uninstallRegistryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppName"

            $registryEntries = @{
                "DisplayName"      = $AppName
                "DisplayVersion"   = $AppVersion
                "Publisher"        = "MC Server Team"
                "InstallLocation"  = $InstallPath
                "UninstallString"  = if (-not [string]::IsNullOrEmpty($UninstallScript)) { "powershell.exe -ExecutionPolicy Bypass -File `"$UninstallScript`"" } else { "" }
                "DisplayIcon"       = $executablePath
                "NoModify"         = 1
                "NoRepair"         = 1
                "URLInfoAbout"     = "https://github.com/mc-server/panel"
                "Contact"          = "support@mc-server.dev"
                "EstimatedSize"    = 40960
                "VersionMajor"     = [int]($AppVersion.Split('.')[0])
                "VersionMinor"     = [int]($AppVersion.Split('.')[1])
            }

            try {
                if (-not (Test-Path -LiteralPath $uninstallRegistryPath)) {
                    $null = New-Item -Path $uninstallRegistryPath -Force -ErrorAction Stop
                }

                foreach ($entry in $registryEntries.GetEnumerator()) {
                    if (-not [string]::IsNullOrEmpty($entry.Value)) {
                        Set-ItemProperty -Path $uninstallRegistryPath -Name $entry.Key -Value $entry.Value -ErrorAction SilentlyContinue
                    }
                }

                $createdItems.RegistryKeys += $uninstallRegistryPath
                Write-Host "注册表卸载入口已创建: $uninstallRegistryPath" -ForegroundColor Green
            } catch {
                Write-Warning "创建注册表卸载入口失败: $_"
            }
        }

        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "  桌面集成安装完成!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        Write-Host ""

        if ($createdItems.DesktopShortcuts.Count -gt 0) {
            Write-Host "已创建 $($createdItems.DesktopShortcuts.Count) 个桌面快捷方式" -ForegroundColor Cyan
        }
        if ($createdItems.StartMenuShortcuts.Count -gt 0) {
            Write-Host "已创建 $($createdItems.StartMenuShortcuts.Count) 个开始菜单项" -ForegroundColor Cyan
        }
        if ($createdItems.RegistryKeys.Count -gt 0) {
            Write-Host "已注册 $($createdItems.RegistryKeys.Count) 个注册表项" -ForegroundColor Cyan
        }

        return $createdItems

    } catch {
        Write-Warning "桌面集成安装过程中发生错误: $_"
        $createdItems.Success = $false
        return $createdItems
    }
}

function Uninstall-DesktopIntegration {
    <#
    .SYNOPSIS
        卸载 Windows 桌面集成（回滚操作）
    .DESCRIPTION
        移除桌面快捷方式、开始菜单快捷方式和注册表卸载入口
    .PARAMETER CreatedItems
        Install-DesktopIntegration 返回的创建项列表
    .OUTPUTS
        Boolean - 回滚成功返回 True，失败返回 False
    .EXAMPLE
        $items = Install-DesktopIntegration -InstallPath "C:\Users\You\AppData\Local\MC Server Panel"
        Uninstall-DesktopIntegration -CreatedItems $items
    .NOTES
        此函数用于安装失败或用户请求卸载时回滚
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [hashtable]$CreatedItems
    )

    $allSuccess = $true

    try {
        foreach ($shortcut in $CreatedItems.DesktopShortcuts) {
            if (Test-Path -LiteralPath $shortcut) {
                try {
                    Remove-Item -Path $shortcut -Force -ErrorAction Stop
                    Write-Host "已删除桌面快捷方式: $shortcut" -ForegroundColor Yellow
                } catch {
                    Write-Warning "删除桌面快捷方式失败: $shortcut"
                    $allSuccess = $false
                }
            }
        }

        foreach ($shortcut in $CreatedItems.StartMenuShortcuts) {
            if (Test-Path -LiteralPath $shortcut) {
                try {
                    Remove-Item -Path $shortcut -Force -ErrorAction Stop
                    Write-Host "已删除开始菜单项: $shortcut" -ForegroundColor Yellow
                } catch {
                    Write-Warning "删除开始菜单项失败: $shortcut"
                    $allSuccess = $false
                }
            }
        }

        $startMenuPath = [Environment]::GetFolderPath("StartMenu")
        $programsPath = Join-Path -Path $startMenuPath -ChildPath "Programs"
        $appFolderPath = Join-Path -Path $programsPath -ChildPath "MC Server Panel"
        if (Test-Path -LiteralPath $appFolderPath) {
            $remainingItems = Get-ChildItem -Path $appFolderPath -ErrorAction SilentlyContinue
            if ($remainingItems.Count -eq 0) {
                try {
                    Remove-Item -Path $appFolderPath -Force -ErrorAction Stop
                    Write-Host "已删除应用文件夹: $appFolderPath" -ForegroundColor Yellow
                } catch {
                    Write-Warning "删除应用文件夹失败: $appFolderPath"
                }
            }
        }

        foreach ($registryKey in $CreatedItems.RegistryKeys) {
            if (Test-Path -LiteralPath $registryKey) {
                try {
                    Remove-Item -Path $registryKey -Recurse -Force -ErrorAction Stop
                    Write-Host "已删除注册表项: $registryKey" -ForegroundColor Yellow
                } catch {
                    Write-Warning "删除注册表项失败: $registryKey"
                    $allSuccess = $false
                }
            }
        }

        if ($allSuccess) {
            Write-Host "桌面集成回滚完成" -ForegroundColor Green
        } else {
            Write-Warning "桌面集成回滚部分完成，存在失败项"
        }

        return $allSuccess

    } catch {
        Write-Warning "桌面集成回滚过程中发生错误: $_"
        return $false
    }
}

Export-ModuleMember -Function @(
    'Create-Shortcut',
    'Install-DesktopIntegration',
    'Uninstall-DesktopIntegration'
)

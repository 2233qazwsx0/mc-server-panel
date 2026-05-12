$ErrorActionPreference = 'Stop'

function Test-Admin {
    <#
    .SYNOPSIS
        检查当前用户是否具有管理员权限

    .DESCRIPTION
        使用 Windows Security Principal API 检查当前进程是否以管理员身份运行。
        通过获取当前 Windows 标识并检查其是否属于管理员内置角色来判断。

    .OUTPUTS
        Boolean - 如果当前用户具有管理员权限返回 True，否则返回 False

    .EXAMPLE
        if (Test-Admin) {
            Write-Host "Running with administrator privileges"
        } else {
            Write-Host "Running without administrator privileges"
        }

    .EXAMPLE
        $isAdmin = Test-Admin
        if (-not $isAdmin) {
            Request-AdminPrivileges
        }
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param()

    try {
        $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
        $isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

        Write-Verbose "Administrator check result: $isAdmin"
        return $isAdmin
    } catch {
        Write-Error "Failed to check administrator privileges: $_"
        return $false
    }
}

function Request-AdminPrivileges {
    <#
    .SYNOPSIS
        请求管理员权限（UAC 提升）

    .DESCRIPTION
        检查当前用户是否具有管理员权限，如果没有则触发 UAC 对话框请求提升。
        使用 Start-Process 重新启动当前脚本并指定 -Verb RunAs 以请求管理员权限。
        如果用户同意 UAC 提示，新进程将以管理员权限运行。

    .PARAMETER PassThru
        如果指定，当用户拒绝 UAC 提升时返回 False，否则不返回任何值

    .OUTPUTS
        Boolean 或无 - 如果用户拒绝提升返回 False（仅当使用 -PassThru 时）

    .EXAMPLE
        Write-Host "Requesting administrator privileges..."
        Request-AdminPrivileges
        Write-Host "Now running with elevated privileges"

    .EXAMPLE
        if (-not (Test-Admin)) {
            Request-AdminPrivileges
        }

    .NOTES
        此函数会终止当前进程并启动新的提升进程。
        传递给脚本的参数会通过 $args 变量传递到新进程。
    #>
    [CmdletBinding()]
    [OutputType([bool])]
    param(
        [Parameter()]
        [switch]$PassThru
    )

    try {
        if (Test-Admin) {
            Write-Verbose "Already running with administrator privileges"
            return
        }

        Write-Host "正在请求管理员权限..." -ForegroundColor Yellow
        Write-Host "如果 UAC 对话框未自动显示，请检查系统任务栏" -ForegroundColor Cyan

        $argumentList = @(
            "-NoProfile"
            "-ExecutionPolicy Bypass"
            "-File `"$PSCommandPath`""
        )

        if ($args) {
            $argumentList += $args
        }

        $processParams = @{
            FilePath     = "powershell.exe"
            ArgumentList = ($argumentList -join " ")
            Verb         = "RunAs"
            ErrorAction  = "Stop"
        }

        Write-Verbose "Starting elevated PowerShell process..."
        Start-Process @processParams

        Write-Host "已启动提升进程，原进程即将退出..." -ForegroundColor Green
        Start-Sleep -Seconds 2

        if ($PassThru) {
            return $false
        }

        exit 0
    } catch {
        if ($_.Exception.Message -match "canceled|拒绝|denied|cancelled") {
            Write-Host "用户取消了 UAC 提升请求" -ForegroundColor Red
            if ($PassThru) {
                return $false
            }
            exit 1
        }

        Write-Error "请求管理员权限失败: $_"
        if ($PassThru) {
            return $false
        }
        exit 1
    }
}

function Require-AdminPrivileges {
    <#
    .SYNOPSIS
        如果无管理员权限则退出（严格的权限检查）

    .DESCRIPTION
        检查当前用户是否具有管理员权限。如果不具有，则抛出异常并退出脚本。
        此函数提供比 Request-AdminPrivileges 更严格的检查，适用于必须以管理员身份运行的安装脚本。

    .PARAMETER Message
        自定义错误消息，当权限不足时显示

    .EXAMPLE
        Require-AdminPrivileges
        Write-Host "Installation can proceed with elevated privileges"

    .EXAMPLE
        Require-AdminPrivileges -Message "This installation requires administrator privileges"

    .THROWS
        如果当前用户不具有管理员权限，则抛出终止错误

    .NOTES
        与 Request-AdminPrivileges 不同，此函数不会触发 UAC 提升对话框。
        它仅用于验证权限并在权限不足时终止脚本。
    #>
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter()]
        [string]$Message
    )

    if (-not (Test-Admin)) {
        $errorMessage = if ($Message) {
            $Message
        } else {
            "此操作需要管理员权限。请右键单击脚本并选择 '以管理员身份运行'，或手动提升权限后重试。"
        }

        Write-Error $errorMessage
        throw "权限不足: $errorMessage"
    }

    Write-Verbose "Administrator privileges verified"
}

Export-ModuleMember -Function Test-Admin, Request-AdminPrivileges, Require-AdminPrivileges

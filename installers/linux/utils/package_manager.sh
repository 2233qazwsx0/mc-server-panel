#!/bin/bash
#===============================================================================
#
#     Rust MC 服务器面板安装器 - Linux 包管理器工具模块
#
#===============================================================================
# 描述: 提供跨发行版的包管理器自动检测和依赖安装功能
#       支持 Debian/Ubuntu, RHEL/CentOS/Fedora, Arch Linux, openSUSE
# 作者: Rust MC 团队
# 版本: 1.0.0
#
# 支持的包管理器:
#   - debian: apt-get (Debian/Ubuntu)
#   - redhat: dnf/yum (RHEL/CentOS/Fedora)
#   - arch: pacman (Arch Linux/Manjaro)
#   - suse: zypper (openSUSE)
#
# 使用方法:
#   source package_manager.sh
#   pm=$(detect_package_manager)
#   install_package curl
#   check_package_installed openssl
#===============================================================================

# 全局变量：当前检测到的包管理器类型
PM_TYPE=""

# 全局变量：静默模式标志（由主安装脚本控制）
# 设置为 true 时抑制非错误输出
PM_QUIET="${PM_QUIET:-false}"

#===============================================================================
# 函数: _pm_log_info
# 描述: 包管理器模块内部日志函数（信息级别）
# 参数:
#   $1 - 日志消息内容
# 返回: 无
#===============================================================================
_pmf_log_info() {
    local message="$1"
    if [ "$PM_QUIET" = false ]; then
        echo "[PM] $message"
    fi
}

#===============================================================================
# 函数: _pm_log_error
# 描述: 包管理器模块内部日志函数（错误级别）
# 参数:
#   $1 - 错误消息内容
# 返回: 无
#===============================================================================
_pmf_log_error() {
    local message="$1"
    echo "[PM ERROR] $message" >&2
}

#===============================================================================
# 函数: detect_package_manager
# 描述: 自动检测系统使用的包管理器类型
#       通过检查 /etc/os-release 文件和包管理器命令可用性来判断
# 参数: 无
# 返回: 打印包管理器类型字符串，退出码为 0
#
# 返回值说明:
#   - debian: Debian/Ubuntu 系列 (使用 apt-get)
#   - redhat: RHEL/CentOS/Fedora 系列 (使用 dnf/yum)
#   - arch: Arch Linux/Manjaro 系列 (使用 pacman)
#   - suse: openSUSE 系列 (使用 zypper)
#   - unknown: 未知系统
#
# 示例:
#   pm=$(detect_package_manager)
#   echo "检测到包管理器: $pm"
#===============================================================================
detect_package_manager() {
    local os_release_file="/etc/os-release"
    local pm_type="unknown"

    # 优先级检测列表：按常见程度排序
    local priority_order="apt dnf yum pacman zypper"

    # 方法1: 通过 /etc/os-release 文件检测
    if [ -f "$os_release_file" ]; then
        # 读取 ID 字段（发行版标识符）
        local os_id
        os_id=$(grep -E '^ID=' "$os_release_file" 2>/dev/null | head -n1 | cut -d'=' -f2 | tr -d '"' || echo "")

        # 读取 ID_LIKE 字段（兼容的发行版）
        local os_id_like
        os_id_like=$(grep -E '^ID_LIKE=' "$os_release_file" 2>/dev/null | head -n1 | cut -d'=' -f2 | tr -d '"' || echo "")

        _pmf_log_info "检测到 OS ID: $os_id, ID_LIKE: $os_id_like"

        # 根据发行版标识符确定包管理器类型
        case "$os_id" in
            debian|ubuntu|linuxmint|pop)
                pm_type="debian"
                ;;
            fedora)
                pm_type="redhat"
                ;;
            arch|manjaro|endeavouros)
                pm_type="arch"
                ;;
            opensuse|opensuse-leap|opensuse-tumbleweed|suse|sles)
                pm_type="suse"
                ;;
            centos|rhel|rocky|alma)
                pm_type="redhat"
                ;;
        esac

        # 如果直接匹配未命中，尝试 ID_LIKE
        if [ "$pm_type" = "unknown" ] && [ -n "$os_id_like" ]; then
            case "$os_id_like" in
                *debian*|*ubuntu*)
                    pm_type="debian"
                    ;;
                *rhel*|*fedora*|*centos*)
                    pm_type="redhat"
                    ;;
                *arch*)
                    pm_type="arch"
                    ;;
                *suse*|*opensuse*)
                    pm_type="suse"
                    ;;
            esac
        fi
    fi

    # 方法2: 如果发行版检测失败，尝试直接检测包管理器命令
    if [ "$pm_type" = "unknown" ]; then
        _pmf_log_info "发行版检测失败，尝试直接检测包管理器命令..."

        # 按优先级检测包管理器命令
        for pm in $priority_order; do
            if command -v "$pm" > /dev/null 2>&1; then
                case "$pm" in
                    apt|apt-get)
                        pm_type="debian"
                        ;;
                    dnf|yum)
                        pm_type="redhat"
                        ;;
                    pacman)
                        pm_type="arch"
                        ;;
                    zypper)
                        pm_type="suse"
                        ;;
                esac
                break
            fi
        done
    fi

    # 缓存检测结果到全局变量
    PM_TYPE="$pm_type"

    # 输出检测结果
    if [ "$PM_QUIET" = false ]; then
        if [ "$pm_type" = "unknown" ]; then
            _pm_log_error "无法检测到支持的包管理器"
        else
            _pmf_log_info "检测到包管理器: $pm_type"
        fi
    fi

    echo "$pm_type"
    return 0
}

#===============================================================================
# 函数: get_current_package_manager
# 描述: 获取当前已缓存的包管理器类型（不重新检测）
# 参数: 无
# 返回: 打印包管理器类型字符串
# 示例:
#   pm=$(get_current_package_manager)
#===============================================================================
get_current_package_manager() {
    if [ -z "$PM_TYPE" ]; then
        detect_package_manager > /dev/null
    fi
    echo "$PM_TYPE"
}

#===============================================================================
# 函数: check_package_installed
# 描述: 检查指定包是否已安装在系统中
#       根据当前检测到的包管理器类型使用相应的检查命令
# 参数:
#   $1 - 包名
# 返回: 退出码 0 表示已安装，1 表示未安装
#
# 示例:
#   if check_package_installed curl; then
#       echo "curl 已安装"
#   else
#       echo "curl 未安装"
#   fi
#===============================================================================
check_package_installed() {
    local package_name="$1"
    local pm_type

    # 参数验证
    if [ -z "$package_name" ]; then
        _pm_log_error "check_package_installed: 缺少包名参数"
        return 1
    fi

    # 获取包管理器类型
    pm_type=$(get_current_package_manager)

    if [ "$pm_type" = "unknown" ]; then
        _pm_log_error "无法确定包管理器类型"
        return 1
    fi

    # 根据包管理器类型执行检查
    case "$pm_type" in
        debian)
            dpkg -s "$package_name" > /dev/null 2>&1
            ;;
        redhat)
            rpm -q "$package_name" > /dev/null 2>&1
            ;;
        arch)
            pacman -Q "$package_name" > /dev/null 2>&1
            ;;
        suse)
            rpm -q "$package_name" > /dev/null 2>&1
            ;;
        *)
            _pm_log_error "不支持的包管理器类型: $pm_type"
            return 1
            ;;
    esac
}

#===============================================================================
# 函数: install_package_debian
# 描述: 使用 apt-get 安装包（Debian/Ubuntu 系统）
# 参数:
#   $1 - 包名
# 返回: 退出码 0 表示成功，1 表示失败
#===============================================================================
install_package_debian() {
    local package_name="$1"
    local apt_cmd="apt-get"

    if [ -z "$package_name" ]; then
        _pm_log_error "install_package_debian: 缺少包名参数"
        return 1
    fi

    # 检查 apt-get 命令是否可用
    if ! command -v "$apt_cmd" > /dev/null 2>&1; then
        _pm_log_error "apt-get 命令不可用"
        return 1
    fi

    # 检查包是否已安装
    if check_package_installed "$package_name" 2>/dev/null; then
        _pmf_log_info "$package_name 已安装，跳过"
        return 0
    fi

    _pmf_log_info "安装包: $package_name (apt-get)"

    # 执行安装命令
    if [ "$DRY_RUN" = true ]; then
        _pmf_log_info "[DryRun] apt-get install -y $package_name"
        return 0
    fi

    if [ "$PM_QUIET" = true ]; then
        apt-get update -qq 2>/dev/null || true
        apt-get install -y -qq "$package_name" > /dev/null 2>&1
    else
        apt-get update -qq
        apt-get install -y -qq "$package_name"
    fi

    if [ $? -eq 0 ]; then
        _pmf_log_info "$package_name 安装成功"
        return 0
    else
        _pm_log_error "$package_name 安装失败"
        return 1
    fi
}

#===============================================================================
# 函数: install_package_redhat
# 描述: 使用 dnf 或 yum 安装包（RHEL/CentOS/Fedora 系统）
#       优先使用 dnf，如果不可用则回退到 yum
# 参数:
#   $1 - 包名
# 返回: 退出码 0 表示成功，1 表示失败
#===============================================================================
install_package_redhat() {
    local package_name="$1"
    local pm_cmd=""

    if [ -z "$package_name" ]; then
        _pm_log_error "install_package_redhat: 缺少包名参数"
        return 1
    fi

    # 检查包是否已安装
    if check_package_installed "$package_name" 2>/dev/null; then
        _pmf_log_info "$package_name 已安装，跳过"
        return 0
    fi

    # 优先使用 dnf，回退到 yum
    if command -v dnf > /dev/null 2>&1; then
        pm_cmd="dnf"
    elif command -v yum > /dev/null 2>&1; then
        pm_cmd="yum"
    else
        _pm_log_error "dnf/yum 命令均不可用"
        return 1
    fi

    _pmf_log_info "安装包: $package_name ($pm_cmd)"

    # 执行安装命令
    if [ "$DRY_RUN" = true ]; then
        _pmf_log_info "[DryRun] $pm_cmd install -y $package_name"
        return 0
    fi

    if [ "$PM_QUIET" = true ]; then
        $pm_cmd install -y -q "$package_name" > /dev/null 2>&1
    else
        $pm_cmd install -y -q "$package_name"
    fi

    if [ $? -eq 0 ]; then
        _pmf_log_info "$package_name 安装成功"
        return 0
    else
        _pm_log_error "$package_name 安装失败"
        return 1
    fi
}

#===============================================================================
# 函数: install_package_arch
# 描述: 使用 pacman 安装包（Arch Linux/Manjaro 系统）
# 参数:
#   $1 - 包名
# 返回: 退出码 0 表示成功，1 表示失败
#===============================================================================
install_package_arch() {
    local package_name="$1"
    local pacman_cmd="pacman"

    if [ -z "$package_name" ]; then
        _pm_log_error "install_package_arch: 缺少包名参数"
        return 1
    fi

    # 检查 pacman 命令是否可用
    if ! command -v "$pacman_cmd" > /dev/null 2>&1; then
        _pm_log_error "pacman 命令不可用"
        return 1
    fi

    # 检查包是否已安装
    if check_package_installed "$package_name" 2>/dev/null; then
        _pmf_log_info "$package_name 已安装，跳过"
        return 0
    fi

    _pmf_log_info "安装包: $package_name (pacman)"

    # 执行安装命令
    if [ "$DRY_RUN" = true ]; then
        _pmf_log_info "[DryRun] pacman -Sy --noconfirm $package_name"
        return 0
    fi

    if [ "$PM_QUIET" = true ]; then
        pacman -Sy --noconfirm "$package_name" > /dev/null 2>&1
    else
        pacman -Sy --noconfirm "$package_name"
    fi

    if [ $? -eq 0 ]; then
        _pmf_log_info "$package_name 安装成功"
        return 0
    else
        _pm_log_error "$package_name 安装失败"
        return 1
    fi
}

#===============================================================================
# 函数: install_package_suse
# 描述: 使用 zypper 安装包（openSUSE 系统）
# 参数:
#   $1 - 包名
# 返回: 退出码 0 表示成功，1 表示失败
#===============================================================================
install_package_suse() {
    local package_name="$1"
    local zypper_cmd="zypper"

    if [ -z "$package_name" ]; then
        _pm_log_error "install_package_suse: 缺少包名参数"
        return 1
    fi

    # 检查 zypper 命令是否可用
    if ! command -v "$zypper_cmd" > /dev/null 2>&1; then
        _pm_log_error "zypper 命令不可用"
        return 1
    fi

    # 检查包是否已安装（openSUSE 也使用 rpm）
    if rpm -q "$package_name" > /dev/null 2>&1; then
        _pmf_log_info "$package_name 已安装，跳过"
        return 0
    fi

    _pmf_log_info "安装包: $package_name (zypper)"

    # 执行安装命令
    if [ "$DRY_RUN" = true ]; then
        _pmf_log_info "[DryRun] zypper install -y $package_name"
        return 0
    fi

    if [ "$PM_QUIET" = true ]; then
        zypper install -y "$package_name" > /dev/null 2>&1
    else
        zypper install -y "$package_name"
    fi

    if [ $? -eq 0 ]; then
        _pmf_log_info "$package_name 安装成功"
        return 0
    else
        _pm_log_error "$package_name 安装失败"
        return 1
    fi
}

#===============================================================================
# 函数: install_package
# 描述: 安装指定包（自动使用正确的包管理器）
#       这是主要入口函数，会自动调用对应的包管理器安装函数
# 参数:
#   $1 - 包名
#   $2 - 可选：包管理器类型（如果省略则自动检测）
# 返回: 退出码 0 表示成功，1 表示失败
#
# 示例:
#   install_package curl
#   install_package openssl redhat
#===============================================================================
install_package() {
    local package_name="$1"
    local pm_type="${2:-}"
    local result=0

    # 参数验证
    if [ -z "$package_name" ]; then
        _pm_log_error "install_package: 缺少包名参数"
        return 1
    fi

    # 如果未指定包管理器类型，则自动检测
    if [ -z "$pm_type" ]; then
        pm_type=$(get_current_package_manager)
    fi

    if [ "$pm_type" = "unknown" ]; then
        _pm_log_error "无法确定包管理器类型"
        return 1
    fi

    # 根据包管理器类型调用相应的安装函数
    case "$pm_type" in
        debian)
            install_package_debian "$package_name"
            result=$?
            ;;
        redhat)
            install_package_redhat "$package_name"
            result=$?
            ;;
        arch)
            install_package_arch "$package_name"
            result=$?
            ;;
        suse)
            install_package_suse "$package_name"
            result=$?
            ;;
        *)
            _pm_log_error "不支持的包管理器类型: $pm_type"
            return 1
            ;;
    esac

    return $result
}

#===============================================================================
# 函数: install_dependencies
# 描述: 安装预定义的依赖列表
#       根据配置文件中的依赖列表进行安装
# 参数: 无
# 返回: 退出码 0 表示全部成功，非 0 表示有失败的安装
#
# 依赖列表:
#   - Debian/Ubuntu: curl, openssl
#   - RHEL/CentOS: curl, openssl
#   - Arch Linux: curl, openssl
#
# 示例:
#   install_dependencies
#===============================================================================
install_dependencies() {
    local pm_type
    local dependencies=()
    local failed=0
    local success_count=0
    local total_count=0

    # 获取包管理器类型
    pm_type=$(get_current_package_manager)

    if [ "$pm_type" = "unknown" ]; then
        _pm_log_error "无法确定包管理器类型"
        return 1
    fi

    # 根据包管理器类型定义依赖列表
    case "$pm_type" in
        debian)
            dependencies=("curl" "openssl")
            ;;
        redhat)
            dependencies=("curl" "openssl")
            ;;
        arch)
            dependencies=("curl" "openssl")
            ;;
        suse)
            dependencies=("curl" "openssl")
            ;;
        *)
            _pm_log_error "不支持的包管理器类型: $pm_type"
            return 1
            ;;
    esac

    total_count=${#dependencies[@]}
    _pmf_log_info "开始安装 $total_count 个依赖包..."

    # 循环安装每个依赖
    for dep in "${dependencies[@]}"; do
        _pmf_log_info "安装依赖: $dep"
        if install_package "$dep" "$pm_type"; then
            ((success_count++))
        else
            ((failed++))
            _pm_log_error "依赖 $dep 安装失败"
        fi
    done

    # 输出安装结果摘要
    _pmf_log_info "依赖安装完成: $success_count/$total_count 成功"

    if [ $failed -gt 0 ]; then
        _pm_log_error "$failed 个依赖安装失败"
        return 1
    fi

    return 0
}

#===============================================================================
# 函数: list_supported_packages
# 描述: 列出指定包管理器支持的包列表（用于调试）
# 参数:
#   $1 - 包管理器类型（可选，默认使用当前检测到的）
# 返回: 打印包列表
#===============================================================================
list_supported_packages() {
    local pm_type="${1:-$(get_current_package_manager)}"

    case "$pm_type" in
        debian)
            echo "Debian/Ubuntu 依赖包:"
            echo "  - curl (HTTP 客户端)"
            echo "  - openssl (SSL/TLS 工具)"
            ;;
        redhat)
            echo "RHEL/CentOS 依赖包:"
            echo "  - curl (HTTP 客户端)"
            echo "  - openssl (SSL/TLS 工具)"
            ;;
        arch)
            echo "Arch Linux 依赖包:"
            echo "  - curl (HTTP 客户端)"
            echo "  - openssl (SSL/TLS 工具)"
            ;;
        suse)
            echo "openSUSE 依赖包:"
            echo "  - curl (HTTP 客户端)"
            echo "  - openssl (SSL/TLS 工具)"
            ;;
        *)
            echo "未知包管理器类型"
            return 1
            ;;
    esac
}

#===============================================================================
# 导出所有函数供外部脚本使用
# 这样在其他脚本中 source 后可以调用这些函数
#===============================================================================
export -f detect_package_manager
export -f get_current_package_manager
export -f check_package_installed
export -f install_package
export -f install_package_debian
export -f install_package_redhat
export -f install_package_arch
export -f install_package_suse
export -f install_dependencies
export -f list_supported_packages

# 导出全局变量
export PM_TYPE PM_QUIET

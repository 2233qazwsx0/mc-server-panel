#!/bin/bash
#===============================================================================
#
#     Rust MC 服务器面板安装器 - Linux sudo 权限管理工具模块
#
#===============================================================================
# 描述: 提供 Linux 系统下的 sudo 权限检测、请求和验证功能
# 作者: Rust MC 团队
# 版本: 1.0.0
# 使用方法:
#   source installers/linux/utils/sudo.sh
#   check_sudo      # 检查是否有 sudo 权限
#   request_sudo    # 请求 sudo 权限并重新执行脚本
#   ensure_sudo     # 确保有 sudo 权限，否则退出
#   is_root         # 快速检查是否为 root 用户
#   require_root    # 必须是 root 用户，否则退出
#===============================================================================

set -e

#===============================================================================
# 尝试加载日志工具（如果存在）
# 如果 logger.sh 不存在，定义简单的日志函数作为后备
#===============================================================================
SUDO_UTILS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMMON_DIR="$(dirname "$SUDO_UTILS_DIR")/../common"

if [ -f "$COMMON_DIR/logger.sh" ]; then
    source "$COMMON_DIR/logger.sh"
else
    log_info() {
        echo "[INFO] $1"
    }
    log_error() {
        echo "[ERROR] $1" >&2
    }
    log_warning() {
        echo "[WARNING] $1"
    }
    export -f log_info
    export -f log_error
    export -f log_warning
fi

#===============================================================================
# 函数: check_sudo
# 描述: 检查当前用户是否具有 sudo 权限
# 参数: 无
# 返回: 0 (有权限) 或 1 (无权限)
# 使用示例:
#   if check_sudo; then
#       echo "有 sudo 权限"
#   else
#       echo "没有权限"
#   fi
#===============================================================================
check_sudo() {
    if [ "$EUID" -eq 0 ]; then
        return 0
    fi

    if sudo -n true 2>/dev/null; then
        return 0
    fi

    if sudo -v 2>/dev/null; then
        return 0
    fi

    return 1
}

#===============================================================================
# 函数: request_sudo
# 描述: 请求 sudo 权限（重新执行当前脚本）
# 参数: 无
# 返回: 无（会重新执行脚本）
# 说明: 这是一个阻塞调用，如果用户取消会退出
# 使用示例:
#   request_sudo  # 之后的代码将以 sudo 权限运行
#===============================================================================
request_sudo() {
    if [ "$EUID" -eq 0 ]; then
        log_info "当前已以 root 用户运行"
        return 0
    fi

    log_info "正在请求 sudo 权限..."

    exec sudo "$0" "$@"
}

#===============================================================================
# 函数: ensure_sudo
# 描述: 确保拥有 sudo 权限，否则退出
# 参数: 无
# 返回: 无（无权限时使用 exit 1 退出）
# 使用示例:
#   ensure_sudo || exit 1
#===============================================================================
ensure_sudo() {
    if check_sudo; then
        return 0
    fi

    log_error "此操作需要 sudo 权限"
    log_info ""
    log_info "使用方法: $0 [--dry-run] [--no-backup] [--quiet] [--help]"
    log_info ""
    log_info "提示: 您可以使用以下方式运行:"
    log_info "  sudo $0"
    log_info ""
    log_warning "安装程序需要 sudo 权限来创建系统目录和安装文件"

    exit 1
}

#===============================================================================
# 函数: is_root
# 描述: 快速检查是否为 root 用户
# 参数: 无
# 返回: 0 (是 root) 或 1 (不是 root)
# 使用示例:
#   if is_root; then
#       echo "当前是 root 用户"
#   fi
#===============================================================================
is_root() {
    [ "$EUID" -eq 0 ]
}

#===============================================================================
# 函数: require_root
# 描述: 必须是 root 用户，否则退出
# 参数: 无
# 返回: 无（不是 root 时使用 exit 1 退出）
# 说明: 与 ensure_sudo 不同，require_root 只接受真正的 root 用户
# 使用示例:
#   require_root || exit 1
#===============================================================================
require_root() {
    if is_root; then
        return 0
    fi

    log_error "此操作需要 root 权限"
    log_info ""
    log_info "提示: 请使用 root 用户或使用 'sudo $0' 运行此脚本"

    exit 1
}

#===============================================================================
# 函数: sudo_exec
# 描述: 以 sudo 权限执行命令
# 参数:
#   $@ - 要执行的命令及其参数
# 返回: 命令的退出码
# 说明: 如果已是 root 用户，直接执行命令
# 使用示例:
#   sudo_exec apt-get update
#   sudo_exec mkdir -p /opt/mc-server
#===============================================================================
sudo_exec() {
    if [ "$EUID" -eq 0 ]; then
        "$@"
    else
        sudo "$@"
    fi
}

#===============================================================================
# 函数: sudo_test
# 描述: 测试 sudo 权限（不缓存，保持权限有效）
# 参数: 无
# 返回: 0 (有权限) 或 1 (无权限)
# 说明: 与 check_sudo 类似，但会保持 sudo 票据新鲜
# 使用示例:
#   if sudo_test; then
#       echo "sudo 权限有效"
#   fi
#===============================================================================
sudo_test() {
    if ! sudo -n true 2>/dev/null; then
        sudo -v
    fi
    return 0
}

#===============================================================================
# 导出所有函数供外部脚本使用
# 这样在其他脚本中 source 后可以调用这些函数
#===============================================================================
export -f check_sudo
export -f request_sudo
export -f ensure_sudo
export -f is_root
export -f require_root
export -f sudo_exec
export -f sudo_test

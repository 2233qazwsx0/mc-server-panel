#!/bin/bash
#===============================================================================
#
#     Rust MC 服务器面板卸载脚本 - Linux
#
#===============================================================================
# 描述: MC Server Panel 的 Linux 系统卸载脚本
#       用于完全卸载程序及其所有相关组件
# 作者: Rust MC 团队
# 版本: 1.0.0
#
# 使用方法:
#   sudo ./uninstall.sh [--quiet]
#
# 选项说明:
#   --quiet, -q  静默模式，跳过所有确认提示
#   --help, -h  显示帮助信息
#
#===============================================================================

set -e

#===============================================================================
# 脚本配置
#===============================================================================
readonly APP_NAME="MC Server Panel"
readonly INSTALL_DIR="/opt/mc-server"
readonly BIN_DIR="$INSTALL_DIR/bin"
readonly DATA_DIR="$HOME/.config/mc-server"
readonly SERVICE_NAME="mc-server"
readonly SERVICE_USER="mc-server"
readonly SERVICE_GROUP="mc-server"
readonly DESKTOP_FILE="/usr/share/applications/mc-server.desktop"
readonly SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME.service"
readonly PROFILE_SCRIPT="/etc/profile.d/mc-server.sh"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

QUIET=false
FORCE=false

#===============================================================================
# 颜色定义
#===============================================================================
readonly COLOR_RESET='\033[0m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_CYAN='\033[0;36m'
readonly COLOR_WHITE='\033[0;37m'
readonly COLOR_GRAY='\033[1;30m'

#===============================================================================
# 日志函数
#===============================================================================
log_info() {
    local message="$1"
    if [ "$QUIET" = false ]; then
        echo -e "${COLOR_WHITE}[INFO]${COLOR_RESET} $message"
    fi
}

log_success() {
    local message="$1"
    if [ "$QUIET" = false ]; then
        echo -e "${COLOR_GREEN}[OK]${COLOR_RESET} $message"
    fi
}

log_warning() {
    local message="$1"
    if [ "$QUIET" = false ]; then
        echo -e "${COLOR_YELLOW}[WARNING]${COLOR_RESET} $message"
    fi
}

log_error() {
    local message="$1"
    echo -e "${COLOR_RED}[ERROR]${COLOR_RESET} $message" >&2
}

log_step() {
    local step="$1"
    local message="$2"
    if [ "$QUIET" = false ]; then
        echo -e "${COLOR_BLUE}[$step]${COLOR_RESET} $message"
    fi
}

#===============================================================================
# 打印横幅
#===============================================================================
print_banner() {
    echo -e "${COLOR_CYAN}"
    echo "========================================"
    echo "  $APP_NAME 卸载程序"
    echo "========================================"
    echo -e "${COLOR_RESET}"
    echo ""
}

#===============================================================================
# 帮助信息
#===============================================================================
show_help() {
    print_banner
    cat << EOF
用法: $0 [选项]

选项:
  --quiet, -q  静默模式，跳过所有确认提示
  --force, -f  强制卸载，不询问
  --help, -h   显示此帮助信息

示例:
  $0                    # 交互式卸载
  sudo $0 --quiet       # 静默卸载（使用默认选项）
  sudo $0 --force       # 强制卸载

注意:
  - 需要 root 权限运行
  - 默认不会删除用户数据目录 (\$HOME/.config/mc-server)
  - 使用 --quiet 时将保留用户数据
EOF
    echo ""
}

#===============================================================================
# 参数解析
#===============================================================================
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --quiet|-q)
                QUIET=true
                ;;
            --force|-f)
                FORCE=true
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                echo "使用 --help 查看帮助信息"
                exit 1
                ;;
        esac
        shift
    done
}

#===============================================================================
# 权限检查
#===============================================================================
check_privileges() {
    if [ "$EUID" -ne 0 ]; then
        log_error "此卸载需要 root 权限"
        echo ""
        log_info "请使用以下方式运行:"
        echo "  sudo $0"
        echo "  或者"
        echo "  su -c \"$0\""
        exit 1
    fi
}

#===============================================================================
# 确认卸载
#===============================================================================
confirm_uninstall() {
    if [ "$FORCE" = true ]; then
        log_info "强制模式，跳过确认"
        return 0
    fi

    if [ "$QUIET" = true ]; then
        return 0
    fi

    echo -e "${COLOR_YELLOW}警告: 此操作将卸载 $APP_NAME${COLOR_RESET}"
    echo ""

    local installed_items=()

    [ -d "$INSTALL_DIR" ] && installed_items+=("程序目录: $INSTALL_DIR")
    [ -d "$DATA_DIR" ] && installed_items+=("用户数据: $DATA_DIR")
    [ -f "$SERVICE_FILE" ] && installed_items+=("systemd 服务")
    [ -f "$DESKTOP_FILE" ] && installed_items+=("桌面集成")
    [ -f "$PROFILE_SCRIPT" ] && installed_items+=("PATH 配置")

    if [ ${#installed_items[@]} -gt 0 ]; then
        echo "将删除以下项目:"
        echo ""
        for item in "${installed_items[@]}"; do
            echo -e "  ${COLOR_RED}- $item${COLOR_RESET}"
        done
        echo ""
    fi

    echo -ne "${COLOR_YELLOW}确定要继续吗？${COLOR_RESET} [y/N] "
    read -r response
    echo ""

    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        log_info "卸载已取消"
        exit 0
    fi
}

#===============================================================================
# 检查是否已安装
#===============================================================================
check_installation() {
    local found=false

    [ -d "$INSTALL_DIR" ] && found=true
    [ -d "$BIN_DIR" ] && found=true
    [ -f "$SERVICE_FILE" ] && found=true
    [ -f "$DESKTOP_FILE" ] && found=true
    [ -f "$PROFILE_SCRIPT" ] && found=true

    if [ "$found" = false ]; then
        log_warning "未检测到 $APP_NAME 的安装"
        log_info "可能已经卸载或从未安装"
        exit 0
    fi

    log_info "检测到现有安装"
}

#===============================================================================
# 停止 systemd 服务
#===============================================================================
stop_systemd_service() {
    log_step "1" "停止 systemd 服务..."

    if ! command -v systemctl > /dev/null 2>&1; then
        log_warning "systemctl 不可用，跳过服务停止"
        return 0
    fi

    if [ ! -f "$SERVICE_FILE" ]; then
        log_info "服务文件不存在，跳过"
        return 0
    fi

    if systemctl is-active "$SERVICE_NAME" > /dev/null 2>&1; then
        log_info "停止服务..."
        systemctl stop "$SERVICE_NAME" 2>/dev/null || true
        log_success "服务已停止"
    else
        log_info "服务未运行，跳过"
    fi

    if systemctl is-enabled "$SERVICE_NAME" > /dev/null 2>&1; then
        log_info "禁用服务自启..."
        systemctl disable "$SERVICE_NAME" 2>/dev/null || true
        log_success "服务已禁用"
    fi

    return 0
}

#===============================================================================
# 删除 systemd 服务文件
#===============================================================================
remove_service_file() {
    log_step "2" "删除 systemd 服务文件..."

    if [ ! -f "$SERVICE_FILE" ]; then
        log_info "服务文件不存在，跳过"
        return 0
    fi

    rm -f "$SERVICE_FILE"
    log_success "服务文件已删除: $SERVICE_FILE"

    systemctl daemon-reload 2>/dev/null || true

    return 0
}

#===============================================================================
# 停止运行中的进程
#===============================================================================
kill_processes() {
    log_step "3" "停止运行中的进程..."

    local pids=$(pgrep -f "mc-server" 2>/dev/null || true)

    if [ -z "$pids" ]; then
        log_info "未发现运行中的 mc-server 进程"
        return 0
    fi

    log_info "发现运行中的进程: $pids"
    log_info "正在停止..."

    for pid in $pids; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            sleep 1
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null || true
            fi
        fi
    done

    log_success "进程已停止"

    return 0
}

#===============================================================================
# 删除用户和组
#===============================================================================
remove_user_and_group() {
    log_step "4" "删除服务用户和组..."

    if id "$SERVICE_USER" > /dev/null 2>&1; then
        log_info "删除用户: $SERVICE_USER"
        userdel "$SERVICE_USER" 2>/dev/null || true
        log_success "用户已删除"
    else
        log_info "用户不存在，跳过"
    fi

    if getent group "$SERVICE_GROUP" > /dev/null 2>&1; then
        log_info "删除组: $SERVICE_GROUP"
        groupdel "$SERVICE_GROUP" 2>/dev/null || true
        log_success "组已删除"
    else
        log_info "组不存在，跳过"
    fi

    return 0
}

#===============================================================================
# 删除桌面集成
#===============================================================================
remove_desktop_integration() {
    log_step "5" "删除桌面集成..."

    if [ -f "$DESKTOP_FILE" ]; then
        log_info "删除 .desktop 文件..."
        rm -f "$DESKTOP_FILE"
        log_success ".desktop 文件已删除"

        if command -v update-desktop-database > /dev/null 2>&1; then
            update-desktop-database /usr/share/applications/ 2>/dev/null || true
        fi
    else
        log_info ".desktop 文件不存在，跳过"
    fi

    return 0
}

#===============================================================================
# 删除 PATH 配置
#===============================================================================
remove_path_config() {
    log_step "6" "删除 PATH 配置..."

    if [ -f "$PROFILE_SCRIPT" ]; then
        log_info "删除 profile.d 脚本..."
        rm -f "$PROFILE_SCRIPT"
        log_success "PATH 配置已删除: $PROFILE_SCRIPT"
    else
        log_info "PATH 配置不存在，跳过"
    fi

    return 0
}

#===============================================================================
# 删除程序目录
#===============================================================================
remove_install_directory() {
    log_step "7" "删除程序目录..."

    if [ ! -d "$INSTALL_DIR" ]; then
        log_info "程序目录不存在，跳过"
        return 0
    fi

    log_info "删除目录: $INSTALL_DIR"
    rm -rf "$INSTALL_DIR"
    log_success "程序目录已删除"

    return 0
}

#===============================================================================
# 删除用户数据
#===============================================================================
remove_user_data() {
    if [ "$QUIET" = true ]; then
        log_info "静默模式，跳过用户数据删除"
        return 0
    fi

    if [ "$FORCE" = true ]; then
        log_step "8" "删除用户数据（强制模式）..."
    else
        log_step "8" "删除用户数据..."
    fi

    if [ ! -d "$DATA_DIR" ]; then
        log_info "用户数据目录不存在，跳过"
        return 0
    fi

    if [ "$FORCE" = true ]; then
        log_info "删除目录: $DATA_DIR"
        rm -rf "$DATA_DIR"
        log_success "用户数据已删除"
        return 0
    fi

    echo ""
    echo -ne "${COLOR_YELLOW}是否删除用户数据目录？${COLOR_RESET}"
    echo -e "${COLOR_GRAY}($DATA_DIR)${COLOR_RESET}"
    echo -ne "[y/N] "
    read -r response
    echo ""

    if [[ "$response" =~ ^[Yy]$ ]]; then
        log_info "删除目录: $DATA_DIR"
        rm -rf "$DATA_DIR"
        log_success "用户数据已删除"
    else
        log_info "保留用户数据目录"
    fi

    return 0
}

#===============================================================================
# 清理残留
#===============================================================================
cleanup_remaining() {
    log_step "9" "清理残留项..."

    local cleaned=0

    if [ -f "/tmp/mc-server-install.log" ]; then
        rm -f "/tmp/mc-server-install.log"
        ((cleaned++))
    fi

    local backup_dirs=()
    while IFS= read -r -d '' dir; do
        backup_dirs+=("$dir")
    done < <(find /tmp -maxdepth 1 -name "*mc-server*backup*" -type d 2>/dev/null || true)

    for backup in "${backup_dirs[@]}"; do
        log_info "发现旧备份目录: $backup"
        echo -ne "${COLOR_YELLOW}是否删除？${COLOR_RESET} [y/N] "
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            rm -rf "$backup"
            log_success "已删除: $backup"
            ((cleaned++))
        fi
    done

    if [ "$cleaned" -gt 0 ]; then
        log_success "清理完成"
    else
        log_info "未发现残留项"
    fi

    return 0
}

#===============================================================================
# 显示完成信息
#===============================================================================
show_completion() {
    echo ""
    echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
    echo -e "${COLOR_GREEN}  $APP_NAME 卸载完成！${COLOR_RESET}"
    echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
    echo ""
    echo -e "  ${COLOR_CYAN}已删除:${COLOR_RESET}"
    echo "    - 程序目录"
    echo "    - systemd 服务"
    echo "    - 桌面集成"
    echo "    - PATH 配置"
    echo "    - 服务用户和组"
    echo ""

    if [ "$QUIET" = true ]; then
        echo -e "  ${COLOR_YELLOW}注意:${COLOR_RESET} 用户数据目录保留"
        echo -e "  ${COLOR_GRAY}如需删除，请手动运行: rm -rf $DATA_DIR${COLOR_RESET}"
    fi

    echo ""
    echo -e "${COLOR_GRAY}感谢使用 $APP_NAME！${COLOR_RESET}"
    echo ""
}

#===============================================================================
# 显示摘要
#===============================================================================
show_summary() {
    local removed=0
    local skipped=0

    [ ! -d "$INSTALL_DIR" ] && ((removed++))
    [ ! -f "$SERVICE_FILE" ] && ((removed++))
    [ ! -f "$DESKTOP_FILE" ] && ((removed++))
    [ ! -f "$PROFILE_SCRIPT" ] && ((removed++))

    if [ "$QUIET" = false ] && [ "$FORCE" = false ]; then
        echo ""
        log_info "已清理 $removed 项组件"
    fi
}

#===============================================================================
# 主函数
#===============================================================================
main() {
    parse_args "$@"

    print_banner

    log_info "开始卸载 $APP_NAME"
    echo ""

    check_privileges

    check_installation

    confirm_uninstall

    echo ""

    stop_systemd_service

    remove_service_file

    kill_processes

    remove_user_and_group

    remove_desktop_integration

    remove_path_config

    remove_install_directory

    remove_user_data

    cleanup_remaining

    show_summary

    show_completion

    return 0
}

main "$@"

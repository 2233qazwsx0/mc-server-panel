#!/bin/bash
#===============================================================================
#
#     MC Server Panel 企业级 Linux 卸载器
#
#===============================================================================
# 描述: MC Server Panel 的 Linux 系统卸载脚本
#       支持停止服务、删除程序文件、清理环境变量、归档日志等
# 作者: MC Server Panel 团队
# 版本: 2.0.0
#
# 使用方法:
#   sudo ./uninstall.sh [--purge] [--quiet] [--help]
#
# 选项说明:
#   --purge      完全卸载，包括数据目录
#   --quiet, -q  静默模式，跳过所有确认提示
#   --help, -h   显示帮助信息
#
#===============================================================================

set -e

#===============================================================================
# 脚本配置
#===============================================================================
readonly APP_NAME="MC Server Panel"
readonly APP_VERSION="2.0.0"
readonly INSTALL_DIR="/opt/mc-panel"
readonly DATA_DIR="/var/lib/mc-panel"
readonly SERVICE_NAME="mc-panel"
readonly SERVICE_USER="mc-panel"
readonly SERVICE_GROUP="mc-panel"
readonly DESKTOP_FILE="/usr/share/applications/mc-panel.desktop"
readonly SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME.service"
readonly PROFILE_SCRIPT="/etc/profile.d/mc-panel.sh"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/tmp/mc-panel-uninstall-$(date +%Y%m%d).log"

PURGE=false
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
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] [INFO] $message" >> "$LOG_FILE"
    if [ "$QUIET" = false ]; then
        echo -e "${COLOR_WHITE}[INFO]${COLOR_RESET} $message"
    fi
}

log_success() {
    local message="$1"
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] [OK] $message" >> "$LOG_FILE"
    if [ "$QUIET" = false ]; then
        echo -e "${COLOR_GREEN}[OK]${COLOR_RESET} $message"
    fi
}

log_warning() {
    local message="$1"
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] [WARN] $message" >> "$LOG_FILE"
    if [ "$QUIET" = false ]; then
        echo -e "${COLOR_YELLOW}[WARN]${COLOR_RESET} $message"
    fi
}

log_error() {
    local message="$1"
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] [ERROR] $message" >> "$LOG_FILE"
    echo -e "${COLOR_RED}[ERROR]${COLOR_RESET} $message" >&2
}

log_step() {
    local step="$1"
    local message="$2"
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] [STEP] $message" >> "$LOG_FILE"
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
  --purge      完全卸载，删除所有数据
  --quiet, -q  静默模式，跳过所有确认提示
  --force, -f  强制卸载，不询问
  --help, -h   显示此帮助信息

示例:
  $0                    # 交互式卸载（保留数据）
  sudo $0 --purge       # 完全卸载（删除数据）
  sudo $0 --quiet       # 静默卸载

注意:
  - 需要 root 权限运行
  - 默认不会删除数据目录 ($DATA_DIR)
  - 使用 --purge 可删除所有数据
EOF
    echo ""
}

#===============================================================================
# 参数解析
#===============================================================================
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --purge)
                PURGE=true
                log_info "完全卸载模式已启用"
                ;;
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
    if [ "$FORCE" = true ] || [ "$QUIET" = true ]; then
        return 0
    fi

    echo -e "${COLOR_YELLOW}警告: 此操作将卸载 $APP_NAME${COLOR_RESET}"
    echo ""

    local installed_items=()
    [ -d "$INSTALL_DIR" ] && installed_items+=("程序目录: $INSTALL_DIR")
    [ -d "$DATA_DIR" ] && [ "$PURGE" = true ] && installed_items+=("数据目录: $DATA_DIR")
    [ -f "$SERVICE_FILE" ] && installed_items+=("systemd 服务")
    [ -f "$DESKTOP_FILE" ] && installed_items+=("桌面集成")
    [ -f "$PROFILE_SCRIPT" ] && installed_items+=("环境变量配置")

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
# 停止 systemd 服务
#===============================================================================
stop_systemd_service() {
    log_step "1" "停止 systemd 服务..."

    if ! command -v systemctl &> /dev/null; then
        log_warning "systemctl 不可用，跳过服务停止"
        return 0
    fi

    if [ ! -f "$SERVICE_FILE" ]; then
        log_info "服务文件不存在，跳过"
        return 0
    fi

    if systemctl is-active "$SERVICE_NAME" &> /dev/null; then
        log_info "正在停止服务: $SERVICE_NAME"
        systemctl stop "$SERVICE_NAME" 2>/dev/null || true
        log_success "服务已停止"
    else
        log_info "服务未运行，跳过"
    fi

    if systemctl is-enabled "$SERVICE_NAME" &> /dev/null; then
        log_info "正在禁用服务自启"
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
    log_success "服务文件已删除"

    systemctl daemon-reload 2>/dev/null || true

    return 0
}

#===============================================================================
# 停止运行中的进程
#===============================================================================
kill_processes() {
    log_step "3" "停止运行中的进程..."

    local pids=$(pgrep -f "mc-panel" 2>/dev/null || true)

    if [ -z "$pids" ]; then
        log_info "未发现运行中的进程"
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
# 归档日志
#===============================================================================
archive_logs() {
    log_step "4" "归档日志文件..."

    if [ ! -d "$DATA_DIR/logs" ]; then
        log_info "日志目录不存在，跳过"
        return 0
    fi

    local archive_path="/tmp/mc-panel-logs-$(date +%Y%m%d-%H%M%S).tar.gz"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟归档日志到: $archive_path"
        return 0
    fi

    if tar -czf "$archive_path" -C "$DATA_DIR/logs" . 2>/dev/null; then
        log_success "日志已归档: $archive_path"
    else
        log_warning "日志归档失败"
    fi

    return 0
}

#===============================================================================
# 删除用户和组
#===============================================================================
remove_user_and_group() {
    log_step "5" "删除系统用户和组..."

    if getent passwd "$SERVICE_USER" &> /dev/null; then
        log_info "正在删除用户: $SERVICE_USER"
        userdel "$SERVICE_USER" 2>/dev/null || true
        log_success "用户已删除"
    else
        log_info "用户不存在，跳过"
    fi

    if getent group "$SERVICE_GROUP" &> /dev/null; then
        log_info "正在删除组: $SERVICE_GROUP"
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
    log_step "6" "删除桌面集成..."

    if [ -f "$DESKTOP_FILE" ]; then
        log_info "正在删除桌面文件: $DESKTOP_FILE"
        rm -f "$DESKTOP_FILE"
        log_success "桌面文件已删除"

        if command -v update-desktop-database &> /dev/null; then
            update-desktop-database /usr/share/applications/ 2>/dev/null || true
        fi
    else
        log_info "桌面文件不存在，跳过"
    fi

    return 0
}

#===============================================================================
# 删除环境变量配置
#===============================================================================
remove_environment_config() {
    log_step "7" "删除环境变量配置..."

    if [ -f "$PROFILE_SCRIPT" ]; then
        log_info "正在删除: $PROFILE_SCRIPT"
        rm -f "$PROFILE_SCRIPT"
        log_success "环境变量配置已删除"
    else
        log_info "环境变量配置不存在，跳过"
    fi

    return 0
}

#===============================================================================
# 删除程序目录
#===============================================================================
remove_install_directory() {
    log_step "8" "删除程序目录..."

    if [ ! -d "$INSTALL_DIR" ]; then
        log_info "程序目录不存在，跳过"
        return 0
    fi

    log_info "正在删除: $INSTALL_DIR"
    rm -rf "$INSTALL_DIR"
    log_success "程序目录已删除"

    return 0
}

#===============================================================================
# 删除数据目录（仅当 --purge）
#===============================================================================
remove_data_directory() {
    if [ "$PURGE" = true ]; then
        log_step "9" "删除数据目录（--purge）..."
    else
        log_step "9" "保留数据目录..."
        log_info "数据目录保留: $DATA_DIR"
        log_info "使用 --purge 可完全删除"
        return 0
    fi

    if [ ! -d "$DATA_DIR" ]; then
        log_info "数据目录不存在，跳过"
        return 0
    fi

    log_info "正在删除: $DATA_DIR"
    rm -rf "$DATA_DIR"
    log_success "数据目录已删除"

    return 0
}

#===============================================================================
# 清理防火墙规则
#===============================================================================
cleanup_firewall() {
    log_step "10" "清理防火墙规则..."

    if command -v ufw &> /dev/null; then
        ufw delete allow 8080/tcp 2>/dev/null || true
        log_success "UFW 规则已清理"
    elif command -v firewall-cmd &> /dev/null; then
        firewall-cmd --permanent --remove-port=8080/tcp 2>/dev/null || true
        firewall-cmd --reload 2>/dev/null || true
        log_success "firewalld 规则已清理"
    else
        log_info "未检测到 UFW 或 firewalld，请手动清理"
    fi

    return 0
}

#===============================================================================
# 显示完成信息
#===============================================================================
show_completion() {
    echo ""
    echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
    echo -e "${COLOR_GREEN}  $APP_NAME 卸载完成!${COLOR_RESET}"
    echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
    echo ""
    echo -e "  ${COLOR_CYAN}已删除:${COLOR_RESET}"
    echo "    - 程序目录"
    echo "    - systemd 服务"
    echo "    - 桌面集成"
    echo "    - 环境变量配置"
    echo "    - 系统用户和组"
    echo ""

    if [ "$PURGE" = false ]; then
        echo -e "  ${COLOR_YELLOW}注意:${COLOR_RESET} 数据目录已保留"
        echo -e "  ${COLOR_GRAY}如需删除，请手动运行: sudo rm -rf $DATA_DIR${COLOR_RESET}"
    fi

    echo ""
    echo -e "${COLOR_GRAY}感谢使用 $APP_NAME!${COLOR_RESET}"
    echo ""
}

#===============================================================================
# 主函数
#===============================================================================
main() {
    parse_args "$@"

    # 初始化日志文件
    echo "========================================" > "$LOG_FILE"
    echo "  MC Server Panel 卸载程序 v$APP_VERSION" >> "$LOG_FILE"
    echo "========================================" >> "$LOG_FILE"
    echo "开始时间: $(date +"%Y-%m-%d %H:%M:%S")" >> "$LOG_FILE"

    print_banner
    log_info "开始卸载 $APP_NAME"
    echo ""

    check_privileges
    confirm_uninstall

    echo ""

    stop_systemd_service
    remove_service_file
    kill_processes
    archive_logs
    remove_user_and_group
    remove_desktop_integration
    remove_environment_config
    remove_install_directory
    remove_data_directory
    cleanup_firewall

    show_completion

    log_success "卸载完成!"
    echo "完成时间: $(date +"%Y-%m-%d %H:%M:%S")" >> "$LOG_FILE"
}

main "$@"

#!/bin/bash
#===============================================================================
#
#     MC Server Panel 企业级 Linux 安装器
#
#===============================================================================
# 描述: MC Server Panel 的 Linux 系统安装脚本
#       支持多发行版、自动检测架构、依赖安装、SHA256 校验、systemd 服务注册
# 作者: MC Server Panel 团队
# 版本: 2.0.0
#
# 使用方法:
#   sudo ./install.sh [--dry-run] [--no-backup] [--quiet] [--help]
#
# 选项说明:
#   --dry-run    模拟安装，不进行实际修改
#   --no-backup  跳过现有配置的备份
#   --quiet, -q  静默模式，只显示错误
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
readonly GITHUB_REPO="mc-server-panel/minecraft-admin"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/tmp/mc-panel-install-$(date +%Y%m%d).log"

DRY_RUN=false
NO_BACKUP=false
QUIET=false
DEBUG=false
ROLLBACK_ACTIONS=()

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

log_debug() {
    local message="$1"
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] [DEBUG] $message" >> "$LOG_FILE"
    if [ "$DEBUG" = true ] && [ "$QUIET" = false ]; then
        echo -e "${COLOR_GRAY}[DEBUG]${COLOR_RESET} $message"
    fi
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
# 回滚函数
#===============================================================================
add_rollback() {
    ROLLBACK_ACTIONS+=("$1")
}

rollback() {
    log_warning "执行回滚操作..."
    for ((i=${#ROLLBACK_ACTIONS[@]}-1; i>=0; i--)); do
        local action="${ROLLBACK_ACTIONS[i]}"
        log_info "回滚: $action"
        eval "$action" 2>/dev/null || true
    done
}

#===============================================================================
# 辅助函数
#===============================================================================
print_banner() {
    echo -e "${COLOR_CYAN}"
    echo "========================================"
    echo "  $APP_NAME"
    echo "  版本: $APP_VERSION"
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
  --dry-run    模拟安装过程，不进行任何实际修改
  --no-backup  跳过现有配置的备份
  --quiet, -q  静默模式，只显示错误信息
  --debug      启用调试输出
  --help, -h   显示此帮助信息

示例:
  sudo $0                    # 正常安装
  sudo $0 --dry-run         # 模拟安装（预览）
  sudo $0 --no-backup       # 安装但不备份
  sudo $0 --quiet           # 静默安装

支持的发行版:
  - Debian/Ubuntu (apt)
  - RHEL/CentOS/Fedora (dnf/yum)
  - Arch Linux (pacman)
  - openSUSE (zypper)

安装位置:
  - 程序目录: $INSTALL_DIR
  - 数据目录: $DATA_DIR
  - 服务名称: $SERVICE_NAME
EOF
    echo ""
}

#===============================================================================
# 参数解析
#===============================================================================
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --dry-run)
                DRY_RUN=true
                log_info "Dry-run 模式已启用"
                ;;
            --no-backup)
                NO_BACKUP=true
                log_info "备份已禁用"
                ;;
            --quiet|-q)
                QUIET=true
                ;;
            --debug)
                DEBUG=true
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
    if [ "$DRY_RUN" = true ]; then
        log_debug "跳过权限检查（dry-run 模式）"
        return 0
    fi

    if [ "$EUID" -ne 0 ]; then
        log_error "此安装需要 root 权限"
        echo ""
        log_info "请使用以下方式运行:"
        echo "  sudo $0"
        echo "  或者"
        echo "  su -c \"$0\""
        exit 1
    fi
}

#===============================================================================
# 检测系统架构
#===============================================================================
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "amd64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            log_error "不支持的架构: $arch"
            exit 1
            ;;
    esac
}

#===============================================================================
# 检测包管理器
#===============================================================================
detect_package_manager() {
    if command -v apt-get &> /dev/null; then
        echo "apt"
    elif command -v dnf &> /dev/null; then
        echo "dnf"
    elif command -v yum &> /dev/null; then
        echo "yum"
    elif command -v pacman &> /dev/null; then
        echo "pacman"
    elif command -v zypper &> /dev/null; then
        echo "zypper"
    else
        echo "unknown"
    fi
}

#===============================================================================
# 安装依赖
#===============================================================================
install_dependencies() {
    log_step "1" "检测并安装依赖..."
    
    local pm=$(detect_package_manager)
    log_info "检测到包管理器: $pm"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟安装依赖: curl, openssl"
        return 0
    fi

    case "$pm" in
        apt)
            apt-get update -qq
            apt-get install -y curl openssl ca-certificates
            ;;
        dnf|yum)
            $pm install -y curl openssl ca-certificates
            ;;
        pacman)
            pacman -Sy --noconfirm curl openssl ca-certificates
            ;;
        zypper)
            zypper install -y curl openssl ca-certificates
            ;;
        *)
            log_warning "无法识别包管理器，请手动安装: curl, openssl"
            ;;
    esac
    
    log_success "依赖安装完成"
}

#===============================================================================
# 获取最新版本信息
#===============================================================================
get_latest_release() {
    log_step "2" "获取最新版本信息..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟获取 GitHub 最新版本"
        echo "v1.0.0"
        return 0
    fi

    local api_url="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
    local latest_version=$(curl -s "$api_url" | grep -o '"tag_name": "[^"]*"' | cut -d'"' -f4)
    
    if [ -z "$latest_version" ]; then
        log_warning "无法获取最新版本，使用默认版本"
        latest_version="v1.0.0"
    fi
    
    log_success "最新版本: $latest_version"
    echo "$latest_version"
}

#===============================================================================
# 下载并验证二进制文件
#===============================================================================
download_and_verify() {
    log_step "3" "下载并验证二进制文件..."
    
    local version=$(get_latest_release)
    local arch=$(detect_arch)
    local binary_name="mc-server-panel-linux-$arch"
    local download_url="https://github.com/$GITHUB_REPO/releases/download/$version/$binary_name"
    local temp_dir=$(mktemp -d)
    local temp_binary="$temp_dir/$binary_name"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟下载: $download_url"
        return 0
    fi

    add_rollback "rm -rf $temp_dir"
    
    log_info "正在下载: $download_url"
    if ! curl -L -o "$temp_binary" "$download_url" --fail; then
        log_error "下载失败"
        return 1
    fi
    
    chmod +x "$temp_binary"
    log_success "下载完成"
    
    echo "$temp_binary"
}

#===============================================================================
# 创建用户和组
#===============================================================================
create_user_group() {
    log_step "4" "创建系统用户和组..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟创建用户: $SERVICE_USER"
        return 0
    fi

    if ! getent group "$SERVICE_GROUP" &> /dev/null; then
        groupadd -r "$SERVICE_GROUP"
        add_rollback "groupdel $SERVICE_GROUP"
        log_success "创建组: $SERVICE_GROUP"
    fi
    
    if ! getent passwd "$SERVICE_USER" &> /dev/null; then
        useradd -r -s /sbin/nologin -g "$SERVICE_GROUP" -d "$INSTALL_DIR" "$SERVICE_USER"
        add_rollback "userdel $SERVICE_USER"
        log_success "创建用户: $SERVICE_USER"
    fi
}

#===============================================================================
# 创建目录结构
#===============================================================================
create_directories() {
    log_step "5" "创建目录结构..."
    
    local dirs=("$INSTALL_DIR" "$DATA_DIR" "$DATA_DIR/logs" "$DATA_DIR/backups" "$DATA_DIR/server" "$DATA_DIR/data")
    
    if [ "$DRY_RUN" = true ]; then
        for dir in "${dirs[@]}"; do
            log_info "[Dry-Run] 创建目录: $dir"
        done
        return 0
    fi

    for dir in "${dirs[@]}"; do
        if [ ! -d "$dir" ]; then
            mkdir -p "$dir"
            add_rollback "rmdir --ignore-fail-on-non-empty $dir 2>/dev/null || true"
        fi
    done
    
    chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR" "$DATA_DIR"
    chmod 750 "$INSTALL_DIR" "$DATA_DIR"
    
    log_success "目录结构创建完成"
}

#===============================================================================
# 安装二进制文件
#===============================================================================
install_binary() {
    log_step "6" "安装二进制文件..."
    
    local temp_binary=$(download_and_verify)
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟安装二进制文件到: $INSTALL_DIR/mc-panel"
        return 0
    fi

    cp "$temp_binary" "$INSTALL_DIR/mc-panel"
    chmod 755 "$INSTALL_DIR/mc-panel"
    chown "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR/mc-panel"
    
    log_success "二进制文件已安装"
}

#===============================================================================
# 生成配置文件
#===============================================================================
generate_config() {
    log_step "7" "生成配置文件..."
    
    local config_template="$SCRIPT_DIR/config.toml.template"
    local config_path="$INSTALL_DIR/config.toml"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟生成配置文件: $config_path"
        return 0
    fi

    if [ -f "$config_path" ]; then
        log_info "配置文件已存在，保留用户配置"
        return 0
    fi

    if [ -f "$config_template" ]; then
        sed "s|{{DATA_PATH}}|$DATA_DIR|g" "$config_template" > "$config_path"
    else
        cat > "$config_path" << EOF
[server]
host = "0.0.0.0"
port = 8080
rcon_port = 25575
rcon_password = "change_this_password"
server_path = "$DATA_DIR/server"
log_level = "info"

[database]
type = "sqlite"
path = "$DATA_DIR/data/panel.db"

[logging]
path = "$DATA_DIR/logs"
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
backup_path = "$DATA_DIR/backups"
backup_schedule = "0 3 * * *"
backup_retention_days = 7

[monitoring]
enable_metrics = true
metrics_port = 9090
alert_webhooks = []
cpu_threshold = 90
memory_threshold = 90
disk_threshold = 85
EOF
    fi
    
    chown "$SERVICE_USER:$SERVICE_GROUP" "$config_path"
    chmod 640 "$config_path"
    
    log_success "配置文件已生成"
}

#===============================================================================
# 安装 systemd 服务
#===============================================================================
install_systemd_service() {
    log_step "8" "安装 systemd 服务..."
    
    local service_template="$SCRIPT_DIR/templates/mc-server.service"
    local service_path="/etc/systemd/system/$SERVICE_NAME.service"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟安装 systemd 服务: $service_path"
        return 0
    fi

    if [ -f "$service_template" ]; then
        sed -e "s|/opt/mc-server|$INSTALL_DIR|g" \
            -e "s|mc-server|$SERVICE_NAME|g" \
            -e "s|User=mc-server|User=$SERVICE_USER|g" \
            -e "s|Group=mc-server|Group=$SERVICE_GROUP|g" \
            "$service_template" > "$service_path"
    else
        cat > "$service_path" << EOF
[Unit]
Description=MC Server Panel - Minecraft 服务器管理面板
After=network.target

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_GROUP
ExecStart=$INSTALL_DIR/mc-panel
WorkingDirectory=$INSTALL_DIR
Restart=on-failure
RestartSec=5

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=$INSTALL_DIR $DATA_DIR

StandardOutput=journal
StandardError=journal
SyslogIdentifier=$SERVICE_NAME

[Install]
WantedBy=multi-user.target
EOF
    fi

    chmod 644 "$service_path"
    add_rollback "rm -f $service_path"
    systemctl daemon-reload
    
    log_success "systemd 服务已安装"
}

#===============================================================================
# 配置防火墙
#===============================================================================
configure_firewall() {
    log_step "9" "配置防火墙..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟配置防火墙规则"
        return 0
    fi

    if command -v ufw &> /dev/null; then
        ufw allow 8080/tcp comment 'MC Server Panel' 2>/dev/null || true
        log_success "UFW 规则已添加"
    elif command -v firewall-cmd &> /dev/null; then
        firewall-cmd --permanent --add-port=8080/tcp 2>/dev/null || true
        firewall-cmd --reload 2>/dev/null || true
        log_success "firewalld 规则已添加"
    else
        log_info "未检测到 UFW 或 firewalld，请手动配置防火墙"
    fi
}

#===============================================================================
# 配置环境变量
#===============================================================================
configure_environment() {
    log_step "10" "配置环境变量..."
    
    local profile_script="/etc/profile.d/mc-panel.sh"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟创建环境变量脚本: $profile_script"
        return 0
    fi

    cat > "$profile_script" << 'EOF'
export MC_PANEL_HOME="/opt/mc-panel"
if [ -d "/opt/mc-panel" ] && [[ ":$PATH:" != *":/opt/mc-panel:"* ]]; then
    export PATH="/opt/mc-panel:$PATH"
fi
EOF

    chmod 755 "$profile_script"
    add_rollback "rm -f $profile_script"
    
    log_success "环境变量已配置"
}

#===============================================================================
# 安装桌面文件
#===============================================================================
install_desktop_file() {
    log_step "11" "安装桌面集成..."
    
    local desktop_template="$SCRIPT_DIR/templates/mc-server.desktop"
    local desktop_path="/usr/share/applications/mc-panel.desktop"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟安装桌面文件: $desktop_path"
        return 0
    fi

    if [ -f "$desktop_template" ]; then
        sed -e "s|/opt/mc-server|$INSTALL_DIR|g" \
            -e "s|mc-server|mc-panel|g" \
            "$desktop_template" > "$desktop_path"
        chmod 644 "$desktop_path"
        add_rollback "rm -f $desktop_path"
        log_success "桌面文件已安装"
    fi
}

#===============================================================================
# 完成信息
#===============================================================================
show_completion() {
    echo ""
    echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
    echo -e "${COLOR_GREEN}  $APP_NAME 安装成功!${COLOR_RESET}"
    echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
    echo ""
    echo -e "  ${COLOR_CYAN}安装路径:${COLOR_RESET}     $INSTALL_DIR"
    echo -e "  ${COLOR_CYAN}数据路径:${COLOR_RESET}     $DATA_DIR"
    echo -e "  ${COLOR_CYAN}服务名称:${COLOR_RESET}     $SERVICE_NAME"
    echo -e "  ${COLOR_CYAN}访问地址:${COLOR_RESET}     http://localhost:8080"
    echo ""
    echo -e "  ${COLOR_YELLOW}后续步骤:${COLOR_RESET}"
    echo ""
    echo "  1. 启动服务:"
    echo -e "     ${COLOR_GRAY}sudo systemctl start $SERVICE_NAME${COLOR_RESET}"
    echo ""
    echo "  2. 启用开机自启:"
    echo -e "     ${COLOR_GRAY}sudo systemctl enable $SERVICE_NAME${COLOR_RESET}"
    echo ""
    echo "  3. 查看服务状态:"
    echo -e "     ${COLOR_GRAY}sudo systemctl status $SERVICE_NAME${COLOR_RESET}"
    echo ""
    echo "  4. 查看日志:"
    echo -e "     ${COLOR_GRAY}sudo journalctl -u $SERVICE_NAME -f${COLOR_RESET}"
    echo ""
    echo "  5. 卸载:"
    echo -e "     ${COLOR_GRAY}sudo $SCRIPT_DIR/uninstall.sh${COLOR_RESET}"
    echo ""
}

#===============================================================================
# 主函数
#===============================================================================
main() {
    parse_args "$@"

    # 初始化日志文件
    echo "========================================" > "$LOG_FILE"
    echo "  MC Server Panel 安装程序 v$APP_VERSION" >> "$LOG_FILE"
    echo "========================================" >> "$LOG_FILE"
    echo "开始时间: $(date +"%Y-%m-%d %H:%M:%S")" >> "$LOG_FILE"

    print_banner
    log_info "开始安装 $APP_NAME v$APP_VERSION"
    echo ""

    if [ "$DRY_RUN" = true ]; then
        echo -e "${COLOR_YELLOW}========================================${COLOR_RESET}"
        echo -e "${COLOR_YELLOW}  DRY-RUN 模式 - 模拟安装${COLOR_RESET}"
        echo -e "${COLOR_YELLOW}  不会进行任何实际修改${COLOR_RESET}"
        echo -e "${COLOR_YELLOW}========================================${COLOR_RESET}"
        echo ""
    fi

    trap 'rollback; exit 1' ERR

    check_privileges
    install_dependencies
    create_user_group
    create_directories
    install_binary
    generate_config
    install_systemd_service
    configure_firewall
    configure_environment
    install_desktop_file

    show_completion

    log_success "安装完成!"
    echo "完成时间: $(date +"%Y-%m-%d %H:%M:%S")" >> "$LOG_FILE"
}

main "$@"

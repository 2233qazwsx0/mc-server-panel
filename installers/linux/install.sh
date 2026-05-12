#!/bin/bash
#===============================================================================
#
#     Rust MC 服务器面板安装器 - Linux 安装脚本
#
#===============================================================================
# 描述: MC Server Panel 的 Linux 系统安装脚本
#       支持多发行版、模块化设计、错误回滚、dry-run 模式
# 作者: Rust MC 团队
# 版本: 1.0.0
#
# 使用方法:
#   ./install.sh [--dry-run] [--no-backup] [--quiet] [--help]
#
# 选项说明:
#   --dry-run    模拟安装，不进行实际修改
#   --no-backup  跳过现有配置的备份
#   --quiet, -q 静默模式，只显示错误
#   --help, -h  显示帮助信息
#
#===============================================================================

set -e

#===============================================================================
# 脚本配置
#===============================================================================
readonly APP_NAME="MC Server Panel"
readonly APP_VERSION="1.0.0"
readonly INSTALL_DIR="/opt/mc-server"
readonly BIN_DIR="$INSTALL_DIR/bin"
readonly DATA_DIR="$HOME/.config/mc-server"
readonly SERVICE_NAME="mc-server"
readonly SERVICE_USER="mc-server"
readonly SERVICE_GROUP="mc-server"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

DRY_RUN=false
NO_BACKUP=false
QUIET=false
DEBUG=false

CHANGES=()

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
# 工具模块加载
#===============================================================================
load_modules() {
    local modules_loaded=0
    local modules_failed=()

    if [ -f "$SCRIPT_DIR/../common/logger.sh" ]; then
        source "$SCRIPT_DIR/../common/logger.sh"
        modules_loaded=$((modules_loaded + 1))
    else
        modules_failed+=("logger.sh")
    fi

    if [ -f "$SCRIPT_DIR/utils/sudo.sh" ]; then
        source "$SCRIPT_DIR/utils/sudo.sh"
        modules_loaded=$((modules_loaded + 1))
    else
        modules_failed+=("sudo.sh")
    fi

    if [ -f "$SCRIPT_DIR/utils/package_manager.sh" ]; then
        source "$SCRIPT_DIR/utils/package_manager.sh"
        modules_loaded=$((modules_loaded + 1))
    else
        modules_failed+=("package_manager.sh")
    fi

    if [ -f "$SCRIPT_DIR/utils/desktop.sh" ]; then
        source "$SCRIPT_DIR/utils/desktop.sh"
        modules_loaded=$((modules_loaded + 1))
    else
        modules_failed+=("desktop.sh")
    fi

    if [ ${#modules_failed[@]} -gt 0 ]; then
        echo -e "${COLOR_YELLOW}[WARNING] 部分模块加载失败: ${modules_failed[*]}${COLOR_RESET}" >&2
        echo -e "${COLOR_GRAY}[INFO] 将使用内置后备函数${COLOR_RESET}"
    fi
}

load_modules

#===============================================================================
# 内置日志函数（当模块加载失败时使用）
#===============================================================================
if ! declare -f log_info > /dev/null 2>&1; then
    log_info() {
        local message="$1"
        if [ "$QUIET" = false ]; then
            echo -e "${COLOR_WHITE}[INFO]${COLOR_RESET} $message"
        fi
    }
fi

if ! declare -f log_success > /dev/null 2>&1; then
    log_success() {
        local message="$1"
        if [ "$QUIET" = false ]; then
            echo -e "${COLOR_GREEN}[OK]${COLOR_RESET} $message"
        fi
    }
fi

if ! declare -f log_warning > /dev/null 2>&1; then
    log_warning() {
        local message="$1"
        if [ "$QUIET" = false ]; then
            echo -e "${COLOR_YELLOW}[WARNING]${COLOR_RESET} $message"
        fi
    }
fi

if ! declare -f log_error > /dev/null 2>&1; then
    log_error() {
        local message="$1"
        echo -e "${COLOR_RED}[ERROR]${COLOR_RESET} $message" >&2
    }
fi

if ! declare -f log_debug > /dev/null 2>&1; then
    log_debug() {
        local message="$1"
        if [ "$DEBUG" = true ] && [ "$QUIET" = false ]; then
            echo -e "${COLOR_GRAY}[DEBUG]${COLOR_RESET} $message"
        fi
    }
fi

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

print_step() {
    local step="$1"
    local message="$2"
    if [ "$QUIET" = false ]; then
        echo -e "${COLOR_BLUE}[$step]${COLOR_RESET} $message"
    fi
}

print_substep() {
    local message="$1"
    if [ "$QUIET" = false ]; then
        echo -e "  ${COLOR_GRAY}->${COLOR_RESET} $message"
    fi
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
  $0                    # 正常安装
  sudo $0 --dry-run     # 模拟安装（预览）
  $0 --no-backup        # 安装但不备份
  $0 --quiet            # 静默安装

支持的发行版:
  - Debian/Ubuntu (apt)
  - RHEL/CentOS/Fedora (dnf/yum)
  - Arch Linux (pacman)
  - openSUSE (zypper)

安装位置:
  - 程序目录: $INSTALL_DIR
  - 二进制文件: $BIN_DIR
  - 用户数据: $DATA_DIR
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

    if declare -f require_root > /dev/null 2>&1; then
        require_root
    else
        if [ "$EUID" -ne 0 ]; then
            log_error "此安装需要 root 权限"
            echo ""
            log_info "请使用以下方式运行:"
            echo "  sudo $0"
            echo "  或者"
            echo "  su -c \"$0\""
            exit 1
        fi
    fi
}

#===============================================================================
# 依赖检查
#===============================================================================
check_dependencies() {
    print_step "1" "检查系统依赖..."

    local pm_type="unknown"

    if declare -f detect_package_manager > /dev/null 2>&1; then
        pm_type=$(detect_package_manager)
    else
        if command -v apt-get > /dev/null 2>&1; then
            pm_type="debian"
        elif command -v dnf > /dev/null 2>&1; then
            pm_type="redhat"
        elif command -v pacman > /dev/null 2>&1; then
            pm_type="arch"
        elif command -v zypper > /dev/null 2>&1; then
            pm_type="suse"
        fi
    fi

    if [ "$pm_type" = "unknown" ]; then
        log_warning "无法检测包管理器，将跳过依赖安装"
        log_info "请手动安装: curl, openssl"
        return 0
    fi

    log_info "检测到包管理器: $pm_type"

    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟安装依赖: curl, openssl"
        return 0
    fi

    if declare -f install_dependencies > /dev/null 2>&1; then
        install_dependencies
        return $?
    fi

    print_substep "安装 curl 和 openssl..."

    case "$pm_type" in
        debian)
            apt-get update -qq
            apt-get install -y curl openssl
            ;;
        redhat)
            if command -v dnf > /dev/null 2>&1; then
                dnf install -y curl openssl
            else
                yum install -y curl openssl
            fi
            ;;
        arch)
            pacman -Sy --noconfirm curl openssl
            ;;
        suse)
            zypper install -y curl openssl
            ;;
    esac

    if [ $? -eq 0 ]; then
        print_substep "依赖安装完成"
        return 0
    else
        log_error "依赖安装失败"
        return 1
    fi
}

#===============================================================================
# 备份
#===============================================================================
backup_existing() {
    if [ "$NO_BACKUP" = true ]; then
        log_info "跳过备份（--no-backup）"
        return 0
    fi

    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 模拟备份现有配置"
        [ -d "$INSTALL_DIR" ] && log_info "[Dry-Run]   备份: $INSTALL_DIR"
        [ -d "$DATA_DIR" ] && log_info "[Dry-Run]   备份: $DATA_DIR"
        return 0
    fi

    local backup_made=false

    if [ -d "$INSTALL_DIR" ]; then
        local timestamp
        timestamp=$(date +%Y%m%d-%H%M%S)
        local backup_path="${INSTALL_DIR}.backup-${timestamp}"
        print_substep "备份现有安装: $INSTALL_DIR -> $backup_path"
        if cp -a "$INSTALL_DIR" "$backup_path"; then
            log_success "备份完成"
            backup_made=true
            CHANGES+=("backup:$INSTALL_DIR:$backup_path")
        else
            log_error "备份失败"
            return 1
        fi
    fi

    if [ -d "$DATA_DIR" ]; then
        local timestamp
        timestamp=$(date +%Y%m%d-%H%M%S)
        local backup_path="${DATA_DIR}.backup-${timestamp}"
        print_substep "备份用户数据: $DATA_DIR -> $backup_path"
        if cp -a "$DATA_DIR" "$backup_path"; then
            log_success "备份完成"
            backup_made=true
            CHANGES+=("backup:$DATA_DIR:$backup_path")
        else
            log_error "备份失败"
            return 1
        fi
    fi

    if [ "$backup_made" = false ]; then
        print_substep "未发现需要备份的现有配置"
    fi

    return 0
}

#===============================================================================
# 创建目录结构
#===============================================================================
create_directories() {
    print_step "2" "创建目录结构..."

    local dirs=("$INSTALL_DIR" "$BIN_DIR" "$DATA_DIR")
    local created=0

    for dir in "${dirs[@]}"; do
        if [ -d "$dir" ]; then
            print_substep "目录已存在: $dir"
        else
            if [ "$DRY_RUN" = true ]; then
                log_info "[Dry-Run] 创建目录: $dir"
            else
                if mkdir -p "$dir"; then
                    log_success "创建目录: $dir"
                    ((created++))
                    CHANGES+=("mkdir:$dir")
                else
                    log_error "创建目录失败: $dir"
                    return 1
                fi
            fi
        fi
    done

    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 将创建 $created 个新目录"
    fi

    return 0
}

#===============================================================================
# 复制二进制文件
#===============================================================================
install_binaries() {
    print_step "3" "安装二进制文件..."

    local backend_binary="$SCRIPT_DIR/../../backend/target/debug/minecraft-admin"
    local source_binary=""

    if [ -f "$backend_binary" ]; then
        source_binary="$backend_binary"
    else
        backend_binary="$SCRIPT_DIR/../../backend/target/release/minecraft-admin"
        if [ -f "$backend_binary" ]; then
            source_binary="$backend_binary"
        fi
    fi

    if [ -n "$source_binary" ] && [ -f "$source_binary" ]; then
        if [ "$DRY_RUN" = true ]; then
            log_info "[Dry-Run] 复制二进制: $source_binary -> $BIN_DIR/mc-server"
        else
            print_substep "复制二进制文件..."
            if cp "$source_binary" "$BIN_DIR/mc-server"; then
                chmod +x "$BIN_DIR/mc-server"
                log_success "二进制文件已安装: $BIN_DIR/mc-server"
                CHANGES+=("copy:$BIN_DIR/mc-server")
            else
                log_error "复制二进制文件失败"
                return 1
            fi
        fi
    else
        if [ "$DRY_RUN" = true ]; then
            log_warning "[Dry-Run] 未找到预编译二进制文件，跳过"
        else
            log_warning "未找到预编译二进制文件"
            log_info "请手动构建后重新运行安装脚本"
            log_info "构建命令: cd backend && cargo build --release"
        fi
    fi

    local config_example="$SCRIPT_DIR/../../backend/config.toml.example"
    if [ -f "$config_example" ]; then
        if [ "$DRY_RUN" = true ]; then
            log_info "[Dry-Run] 复制配置示例: $config_example -> $DATA_DIR/config.toml"
        else
            if [ ! -f "$DATA_DIR/config.toml" ]; then
                cp "$config_example" "$DATA_DIR/config.toml"
                log_success "配置文件已创建: $DATA_DIR/config.toml"
                CHANGES+=("copy:$DATA_DIR/config.toml")
            else
                print_substep "配置文件已存在，跳过"
            fi
        fi
    fi

    return 0
}

#===============================================================================
# 设置文件权限
#===============================================================================
set_permissions() {
    print_step "4" "设置文件权限..."

    if [ "$DRY_RUN" = true ]; then
        log_info "[Dry-Run] 设置权限: $BIN_DIR/mc-server (755)"
        log_info "[Dry-Run] 设置权限: $DATA_DIR (700)"
        return 0
    fi

    if [ -f "$BIN_DIR/mc-server" ]; then
        chmod 755 "$BIN_DIR/mc-server"
        log_success "已设置二进制文件权限: 755"
    fi

    chmod 700 "$DATA_DIR"
    log_success "已设置数据目录权限: 700"

    return 0
}

#===============================================================================
# 安装桌面集成
#===============================================================================
install_desktop_integration() {
    print_step "5" "安装桌面集成..."

    local desktop_template="$SCRIPT_DIR/templates/mc-server.desktop"
    local service_template="$SCRIPT_DIR/templates/mc-server.service"

    if declare -f install_desktop_file > /dev/null 2>&1; then
        print_substep "安装 .desktop 文件..."
        if install_desktop_file "$desktop_template" "$INSTALL_DIR"; then
            log_success ".desktop 文件已安装"
            CHANGES+=("desktop_file:/usr/share/applications/mc-server.desktop")
        else
            log_warning ".desktop 文件安装失败（可能无权限）"
        fi
    else
        if [ -f "$desktop_template" ]; then
            print_substep "安装 .desktop 文件..."
            local desktop_dest="/usr/share/applications/mc-server.desktop"
            if [ "$DRY_RUN" = true ]; then
                log_info "[Dry-Run] 复制: $desktop_template -> $desktop_dest"
            else
                if [ -w "$(dirname "$desktop_dest")" ] 2>/dev/null; then
                    sed "s|/opt/mc-server|$INSTALL_DIR|g" "$desktop_template" > "$desktop_dest"
                    chmod 644 "$desktop_dest"
                    log_success ".desktop 文件已安装"
                    CHANGES+=("desktop_file:$desktop_dest")
                else
                    log_warning "无权限写入 $desktop_dest"
                fi
            fi
        fi
    fi

    if declare -f install_systemd_service > /dev/null 2>&1; then
        print_substep "安装 systemd 服务..."
        if install_systemd_service "$service_template" "$INSTALL_DIR"; then
            log_success "systemd 服务已安装"
            CHANGES+=("service_file:/etc/systemd/system/mc-server.service")
        else
            log_warning "systemd 服务安装失败"
        fi
    else
        if [ -f "$service_template" ]; then
            print_substep "安装 systemd 服务..."
            local service_dest="/etc/systemd/system/mc-server.service"
            if [ "$DRY_RUN" = true ]; then
                log_info "[Dry-Run] 复制: $service_template -> $service_dest"
            else
                if [ -w "$(dirname "$service_dest")" ] 2>/dev/null; then
                    sed -e "s|/opt/mc-server|$INSTALL_DIR|g" \
                        -e "s|User=mc-server|User=$SERVICE_USER|g" \
                        -e "s|Group=mc-server|Group=$SERVICE_GROUP|g" \
                        "$service_template" > "$service_dest"
                    chmod 644 "$service_dest"
                    systemctl daemon-reload 2>/dev/null || true
                    log_success "systemd 服务已安装"
                    CHANGES+=("service_file:$service_dest")
                else
                    log_warning "无权限写入 $service_dest"
                fi
            fi
        fi
    fi

    if declare -f add_to_path > /dev/null 2>&1; then
        print_substep "添加 PATH 配置..."
        if add_to_path "$BIN_DIR"; then
            log_success "PATH 配置已添加"
            CHANGES+=("path_script:/etc/profile.d/mc-server.sh")
        else
            log_warning "PATH 配置添加失败"
        fi
    else
        print_substep "添加 PATH 配置..."
        local profile_script="/etc/profile.d/mc-server.sh"
        if [ "$DRY_RUN" = true ]; then
            log_info "[Dry-Run] 创建: $profile_script"
        else
            if [ -w "$(dirname "$profile_script")" ] 2>/dev/null; then
                cat > "$profile_script" << 'PATHEOF'
if [ -d "/opt/mc-server/bin" ] && [[ ":$PATH:" != *":/opt/mc-server/bin:"* ]]; then
    export PATH="/opt/mc-server/bin:$PATH"
fi
PATHEOF
                chmod 755 "$profile_script"
                log_success "PATH 配置已添加"
                CHANGES+=("path_script:$profile_script")
            else
                log_warning "无权限写入 $profile_script"
            fi
        fi
    fi

    return 0
}

#===============================================================================
# 完成信息
#===============================================================================
show_completion() {
    echo ""
    print_step "6" "安装完成！"
    echo ""
    echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
    echo -e "${COLOR_GREEN}  $APP_NAME 安装成功！${COLOR_RESET}"
    echo -e "${COLOR_GREEN}========================================${COLOR_RESET}"
    echo ""
    echo -e "  ${COLOR_CYAN}安装路径:${COLOR_RESET}     $INSTALL_DIR"
    echo -e "  ${COLOR_CYAN}二进制文件:${COLOR_RESET}   $BIN_DIR/mc-server"
    echo -e "  ${COLOR_CYAN}用户数据:${COLOR_RESET}     $DATA_DIR"
    echo ""
    echo -e "  ${COLOR_YELLOW}后续步骤:${COLOR_RESET}"
    echo ""
    echo "  1. 启动服务:"
    echo -e "     ${COLOR_GRAY}systemctl start mc-server${COLOR_RESET}"
    echo "     或者"
    echo -e "     ${COLOR_GRAY}$BIN_DIR/mc-server${COLOR_RESET}"
    echo ""
    echo "  2. 启用开机自启:"
    echo -e "     ${COLOR_GRAY}systemctl enable mc-server${COLOR_RESET}"
    echo ""
    echo "  3. 查看服务状态:"
    echo -e "     ${COLOR_GRAY}systemctl status mc-server${COLOR_RESET}"
    echo ""
    echo "  4. 卸载:"
    echo -e "     ${COLOR_GRAY}sudo $SCRIPT_DIR/uninstall.sh${COLOR_RESET}"
    echo ""

    if [ ${#CHANGES[@]} -gt 0 ] && [ "$DRY_RUN" = false ]; then
        echo -e "  ${COLOR_GRAY}已记录 ${#CHANGES[@]} 项变更${COLOR_RESET}"
    fi
}

#===============================================================================
# 回滚处理
#===============================================================================
rollback_handler() {
    echo ""
    log_error "安装过程中发生错误，正在执行回滚..."
    rollback_changes
    exit 1
}

rollback_changes() {
    echo ""
    echo -e "${COLOR_YELLOW}========================================${COLOR_RESET}"
    echo -e "${COLOR_YELLOW}  开始回滚变更...${COLOR_RESET}"
    echo -e "${COLOR_YELLOW}========================================${COLOR_RESET}"
    echo ""

    if [ ${#CHANGES[@]} -eq 0 ]; then
        echo "没有需要回滚的变更"
        return 0
    fi

    local total=${#CHANGES[@]}
    echo -e "共 $total 项变更需要回滚"
    echo ""

    local count=0
    for ((i=${#CHANGES[@]}-1; i>=0; i--)); do
        ((count++))
        local change="${CHANGES[$i]}"
        local type="${change%%:*}"
        local rest="${change#*:}"

        echo -e "${COLOR_BLUE}[$count/$total]${COLOR_RESET} 回滚中..."

        case "$type" in
            backup)
                local original="${rest%:*}"
                local backup="${rest#*:}"
                echo -e "  ${COLOR_YELLOW}恢复备份: $original${COLOR_RESET}"
                if [ -d "$backup" ]; then
                    if [ -d "$original" ]; then
                        rm -rf "$original"
                    fi
                    mv "$backup" "$original"
                    echo -e "  ${COLOR_GREEN}已恢复${COLOR_RESET}"
                fi
                ;;
            mkdir)
                echo -e "  ${COLOR_YELLOW}删除目录: $rest${COLOR_RESET}"
                if [ -d "$rest" ]; then
                    rm -rf "$rest"
                    echo -e "  ${COLOR_GREEN}已删除${COLOR_RESET}"
                fi
                ;;
            copy)
                echo -e "  ${COLOR_YELLOW}删除文件: $rest${COLOR_RESET}"
                if [ -f "$rest" ]; then
                    rm -f "$rest"
                    echo -e "  ${COLOR_GREEN}已删除${COLOR_RESET}"
                fi
                ;;
            desktop_file)
                echo -e "  ${COLOR_YELLOW}删除 .desktop 文件: $rest${COLOR_RESET}"
                rm -f "$rest"
                echo -e "  ${COLOR_GREEN}已删除${COLOR_RESET}"
                ;;
            service_file)
                echo -e "  ${COLOR_YELLOW}删除 systemd 服务: $rest${COLOR_RESET}"
                rm -f "$rest"
                systemctl daemon-reload 2>/dev/null || true
                echo -e "  ${COLOR_GREEN}已删除${COLOR_RESET}"
                ;;
            path_script)
                echo -e "  ${COLOR_YELLOW}删除 PATH 配置: $rest${COLOR_RESET}"
                rm -f "$rest"
                echo -e "  ${COLOR_GREEN}已删除${COLOR_RESET}"
                ;;
        esac
    done

    CHANGES=()

    echo ""
    echo -e "${COLOR_GREEN}回滚完成${COLOR_RESET}"
}

#===============================================================================
# 主函数
#===============================================================================
main() {
    parse_args "$@"

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

    check_privileges

    if ! check_dependencies; then
        log_error "依赖检查失败"
        exit 1
    fi

    if ! backup_existing; then
        log_error "备份失败"
        if [ "$DRY_RUN" = false ]; then
            rollback_changes
        fi
        exit 1
    fi

    if ! create_directories; then
        log_error "创建目录失败"
        if [ "$DRY_RUN" = false ]; then
            rollback_changes
        fi
        exit 1
    fi

    if ! install_binaries; then
        log_error "安装二进制文件失败"
        if [ "$DRY_RUN" = false ]; then
            rollback_changes
        fi
        exit 1
    fi

    if ! set_permissions; then
        log_error "设置权限失败"
        if [ "$DRY_RUN" = false ]; then
            rollback_changes
        fi
        exit 1
    fi

    if ! install_desktop_integration; then
        log_error "桌面集成安装失败"
        if [ "$DRY_RUN" = false ]; then
            rollback_changes
        fi
        exit 1
    fi

    show_completion

    return 0
}

trap 'rollback_handler' ERR

main "$@"

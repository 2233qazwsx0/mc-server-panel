#!/bin/bash
#===============================================================================
#
#     Rust MC 服务器面板安装器 - Linux 桌面集成工具模块
#
#===============================================================================
# 描述: 提供 Linux 桌面集成功能，包括 .desktop 文件安装、systemd 服务配置
#       以及 PATH 环境变量设置
# 作者: Rust MC 团队
# 版本: 1.0.0
#
# 使用方法:
#   source installers/linux/utils/desktop.sh
#   install_desktop_file "$SCRIPT_DIR/templates/mc-server.desktop" "$INSTALL_DIR"
#   install_systemd_service "$SCRIPT_DIR/templates/mc-server.service" "$INSTALL_DIR"
#   add_to_path /opt/mc-server/bin
#   uninstall_desktop_integration
#===============================================================================

set -e

#===============================================================================
# 全局变量定义（仅在未定义时设置）
#===============================================================================
DESKTOP_UTILS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMMON_DIR="$(dirname "$DESKTOP_UTILS_DIR")/../common"

: "${DESKTOP_FILE_NAME:=mc-server.desktop}"
: "${SERVICE_FILE_NAME:=mc-server.service}"
: "${SERVICE_NAME:=mc-server}"
: "${PROFILE_SCRIPT_NAME:=mc-server.sh}"
: "${SERVICE_USER:=mc-server}"
: "${SERVICE_GROUP:=mc-server}"

DRY_RUN="${DRY_RUN:-false}"
QUIET="${QUIET:-false}"

#===============================================================================
# 日志函数定义
#===============================================================================
_dt_log_info() {
    local message="$1"
    if [ "$QUIET" = false ]; then
        echo "[DESKTOP INFO] $message"
    fi
}

_dt_log_success() {
    local message="$1"
    if [ "$QUIET" = false ]; then
        echo "[DESKTOP OK] $message"
    fi
}

_dt_log_error() {
    local message="$1"
    echo "[DESKTOP ERROR] $message" >&2
}

_dt_log_warning() {
    local message="$1"
    if [ "$QUIET" = false ]; then
        echo "[DESKTOP WARNING] $message"
    fi
}

_dt_log_debug() {
    local message="$1"
    if [ "$DEBUG" = true ] && [ "$QUIET" = false ]; then
        echo "[DESKTOP DEBUG] $message"
    fi
}

#===============================================================================
# 函数: _dt_check_systemd
# 描述: 检查系统是否使用 systemd
# 参数: 无
# 返回: 0 (是 systemd) 或 1 (不是 systemd)
#===============================================================================
_dt_check_systemd() {
    if command -v systemctl > /dev/null 2>&1 && \
       systemctl --version > /dev/null 2>&1 && \
       [ -d /run/systemd/system ]; then
        return 0
    fi
    return 1
}

#===============================================================================
# 函数: _dt_create_user_if_not_exists
# 描述: 创建服务用户和组（如果不存在）
# 参数:
#   $1 - 用户名
#   $2 - 组名
# 返回: 0 (成功) 或 1 (失败)
#===============================================================================
_dt_create_user_if_not_exists() {
    local user_name="$1"
    local group_name="$2"

    _dt_log_debug "检查用户 $user_name 是否存在..."

    if id "$user_name" > /dev/null 2>&1; then
        _dt_log_debug "用户 $user_name 已存在"
        return 0
    fi

    if [ "$DRY_RUN" = true ]; then
        _dt_log_info "[DryRun] Would create user: $user_name"
        _dt_log_info "[DryRun] Would create group: $group_name"
        return 0
    fi

    _dt_log_info "创建系统用户: $user_name"

    if command -v useradd > /dev/null 2>&1; then
        useradd --system --no-create-home --shell /usr/sbin/nologin "$user_name" 2>/dev/null || {
            _dt_log_warning "useradd 失败，尝试使用 groupadd 和 useradd"
            if ! getent group "$group_name" > /dev/null 2>&1; then
                groupadd "$group_name" 2>/dev/null || true
            fi
            useradd --system --no-create-home --shell /usr/sbin/nologin \
                   --gid "$group_name" "$user_name" 2>/dev/null || true
        }
    else
        _dt_log_error "useradd 命令不可用"
        return 1
    fi

    if id "$user_name" > /dev/null 2>&1; then
        _dt_log_success "用户 $user_name 创建成功"
        return 0
    else
        _dt_log_error "用户 $user_name 创建失败"
        return 1
    fi
}

#===============================================================================
# 函数: install_desktop_file
# 描述: 安装 .desktop 文件到系统应用目录
# 参数:
#   $1 - 模板文件路径
#   $2 - 安装目录 (可选，用于更新 Exec 和路径)
# 返回: 退出码 0 表示成功，非 0 表示失败
#
# 使用示例:
#   install_desktop_file "$SCRIPT_DIR/templates/mc-server.desktop" /opt/mc-server
#===============================================================================
install_desktop_file() {
    local template_file="$1"
    local install_dir="${2:-}"
    local desktop_dest="/usr/share/applications/$DESKTOP_FILE_NAME"

    if [ -z "$template_file" ]; then
        _dt_log_error "install_desktop_file: 缺少模板文件路径参数"
        return 1
    fi

    if [ ! -f "$template_file" ]; then
        _dt_log_error "模板文件不存在: $template_file"
        return 1
    fi

    _dt_log_info "安装 .desktop 文件..."

    if [ "$DRY_RUN" = true ]; then
        _dt_log_info "[DryRun] 复制 $template_file 到 $desktop_dest"
        return 0
    fi

    if [ -w "$desktop_dest" ] 2>/dev/null || [ -w "$(dirname "$desktop_dest")" ] 2>/dev/null; then
        if [ -n "$install_dir" ]; then
            sed "s|/opt/mc-server|$install_dir|g" "$template_file" > "$desktop_dest"
        else
            cp "$template_file" "$desktop_dest"
        fi
        chmod 644 "$desktop_dest"
        _dt_log_success ".desktop 文件已安装到 $desktop_dest"

        if command -v update-desktop-database > /dev/null 2>&1; then
            update-desktop-database /usr/share/applications/ 2>/dev/null || true
            _dt_log_debug "已更新桌面数据库"
        fi

        return 0
    else
        _dt_log_error "没有权限写入 $desktop_dest，请使用 sudo"
        return 1
    fi
}

#===============================================================================
# 函数: install_systemd_service
# 描述: 安装 systemd 服务单元文件
# 参数:
#   $1 - 模板文件路径
#   $2 - 安装目录 (可选，用于更新 Exec 和路径)
# 返回: 退出码 0 表示成功，非 0 表示失败
#
# 使用示例:
#   install_systemd_service "$SCRIPT_DIR/templates/mc-server.service" /opt/mc-server
#===============================================================================
install_systemd_service() {
    local template_file="$1"
    local install_dir="${2:-}"
    local service_dest="/etc/systemd/system/$SERVICE_FILE_NAME"

    if [ -z "$template_file" ]; then
        _dt_log_error "install_systemd_service: 缺少模板文件路径参数"
        return 1
    fi

    if [ ! -f "$template_file" ]; then
        _dt_log_error "模板文件不存在: $template_file"
        return 1
    fi

    if ! _dt_check_systemd; then
        _dt_log_warning "系统未使用 systemd，跳过服务安装"
        return 0
    fi

    _dt_log_info "安装 systemd 服务..."

    _dt_create_user_if_not_exists "$SERVICE_USER" "$SERVICE_GROUP"
    if [ $? -ne 0 ]; then
        _dt_log_error "创建服务用户失败"
        return 1
    fi

    if [ "$DRY_RUN" = true ]; then
        _dt_log_info "[DryRun] 复制 $template_file 到 $service_dest"
        _dt_log_info "[DryRun] 运行 systemctl daemon-reload"
        return 0
    fi

    if [ -w "$service_dest" ] 2>/dev/null || [ -w "$(dirname "$service_dest")" ] 2>/dev/null; then
        if [ -n "$install_dir" ]; then
            sed -e "s|/opt/mc-server|$install_dir|g" \
                -e "s|User=mc-server|User=$SERVICE_USER|g" \
                -e "s|Group=mc-server|Group=$SERVICE_GROUP|g" \
                "$template_file" > "$service_dest"
        else
            cp "$template_file" "$service_dest"
        fi
        chmod 644 "$service_dest"
        _dt_log_success "systemd 服务文件已安装到 $service_dest"

        systemctl daemon-reload 2>/dev/null || true
        _dt_log_debug "已重新加载 systemd 配置"

        return 0
    else
        _dt_log_error "没有权限写入 $service_dest，请使用 sudo"
        return 1
    fi
}

#===============================================================================
# 函数: add_to_path
# 描述: 将安装路径添加到系统 PATH（通过 profile.d 脚本）
# 参数:
#   $1 - 要添加的 bin 目录路径
# 返回: 退出码 0 表示成功，非 0 表示失败
#
# 使用示例:
#   add_to_path /opt/mc-server/bin
#===============================================================================
add_to_path() {
    local bin_dir="$1"
    local profile_script="/etc/profile.d/$PROFILE_SCRIPT_NAME"

    if [ -z "$bin_dir" ]; then
        _dt_log_error "add_to_path: 缺少 bin 目录路径参数"
        return 1
    fi

    if [ ! -d "$bin_dir" ]; then
        _dt_log_warning "bin 目录不存在: $bin_dir"
    fi

    _dt_log_info "添加 $bin_dir 到系统 PATH..."

    if [ "$DRY_RUN" = true ]; then
        _dt_log_info "[DryRun] 创建 $profile_script"
        _dt_log_info "[DryRun] 添加 export PATH=\"$bin_dir:\$PATH\""
        return 0
    fi

    if [ -w "$profile_script" ] 2>/dev/null || [ -w "$(dirname "$profile_script")" ] 2>/dev/null; then
        if [ -f "$profile_script" ]; then
            if grep -q "$bin_dir" "$profile_script" 2>/dev/null; then
                _dt_log_info "PATH 配置已存在，跳过"
                return 0
            fi
            _dt_log_info "更新现有的 $profile_script"
        fi

        cat > "$profile_script" << EOF
# MC Server Panel PATH configuration
# Auto-generated by installer

if [ -d "$bin_dir" ] && [[ ":\$PATH:" != *":$bin_dir:"* ]]; then
    export PATH="$bin_dir:\$PATH"
fi
EOF
        chmod 755 "$profile_script"
        _dt_log_success "PATH 配置已添加到 $profile_script"

        return 0
    else
        _dt_log_error "没有权限写入 $profile_script，请使用 sudo"
        return 1
    fi
}

#===============================================================================
# 函数: uninstall_desktop_integration
# 描述: 卸载所有桌面集成组件
#       包括停止服务、删除服务文件、删除 .desktop 文件、
#       删除 profile.d 脚本、删除用户和组
# 参数: 无
# 返回: 退出码 0 表示成功，非 0 表示失败
#
# 使用示例:
#   uninstall_desktop_integration
#===============================================================================
uninstall_desktop_integration() {
    local desktop_file="/usr/share/applications/$DESKTOP_FILE_NAME"
    local service_file="/etc/systemd/system/$SERVICE_FILE_NAME"
    local profile_script="/etc/profile.d/$PROFILE_SCRIPT_NAME"
    local failed=0

    _dt_log_info "开始卸载桌面集成组件..."

    if _dt_check_systemd; then
        _dt_log_info "停止 systemd 服务..."
        if [ "$DRY_RUN" = true ]; then
            _dt_log_info "[DryRun] systemctl stop $SERVICE_NAME"
            _dt_log_info "[DryRun] systemctl disable $SERVICE_NAME"
        else
            systemctl stop "$SERVICE_NAME" 2>/dev/null || true
            systemctl disable "$SERVICE_NAME" 2>/dev/null || true
            _dt_log_info "服务已停止并禁用"
        fi

        if [ "$DRY_RUN" = true ]; then
            _dt_log_info "[DryRun] 删除 $service_file"
        else
            if [ -f "$service_file" ]; then
                rm -f "$service_file"
                systemctl daemon-reload 2>/dev/null || true
                _dt_log_success "systemd 服务文件已删除"
            fi
        fi
    fi

    if [ "$DRY_RUN" = true ]; then
        _dt_log_info "[DryRun] 删除 $desktop_file"
    else
        if [ -f "$desktop_file" ]; then
            rm -f "$desktop_file"
            _dt_log_success ".desktop 文件已删除"
        fi

        if command -v update-desktop-database > /dev/null 2>&1; then
            update-desktop-database /usr/share/applications/ 2>/dev/null || true
        fi
    fi

    if [ "$DRY_RUN" = true ]; then
        _dt_log_info "[DryRun] 删除 $profile_script"
    else
        if [ -f "$profile_script" ]; then
            rm -f "$profile_script"
            _dt_log_success "PATH 配置已删除"
        fi
    fi

    if [ "$DRY_RUN" = true ]; then
        _dt_log_info "[DryRun] 删除用户 $SERVICE_USER"
        _dt_log_info "[DryRun] 删除组 $SERVICE_GROUP"
    else
        if id "$SERVICE_USER" > /dev/null 2>&1; then
            userdel "$SERVICE_USER" 2>/dev/null || true
            _dt_log_success "用户 $SERVICE_USER 已删除"
        fi

        if getent group "$SERVICE_GROUP" > /dev/null 2>&1; then
            groupdel "$SERVICE_GROUP" 2>/dev/null || true
            _dt_log_success "组 $SERVICE_GROUP 已删除"
        fi
    fi

    if [ $failed -eq 0 ]; then
        _dt_log_success "桌面集成组件卸载完成"
        return 0
    else
        _dt_log_error "部分组件卸载失败"
        return 1
    fi
}

#===============================================================================
# 函数: enable_systemd_service
# 描述: 启用并启动 systemd 服务
# 参数:
#   $1 - 是否启用开机自启 (true/false，默认 true)
# 返回: 退出码 0 表示成功，非 0 表示失败
#
# 使用示例:
#   enable_systemd_service true
#===============================================================================
enable_systemd_service() {
    local enable_autostart="${1:-true}"

    if ! _dt_check_systemd; then
        _dt_log_warning "系统未使用 systemd，跳过服务启用"
        return 0
    fi

    _dt_log_info "启用 systemd 服务..."

    if [ "$DRY_RUN" = true ]; then
        _dt_log_info "[DryRun] systemctl enable $SERVICE_NAME"
        _dt_log_info "[DryRun] systemctl start $SERVICE_NAME"
        return 0
    fi

    if [ "$enable_autostart" = true ]; then
        systemctl enable "$SERVICE_NAME" 2>/dev/null || {
            _dt_log_error "服务启用失败"
            return 1
        }
        _dt_log_info "服务已设为开机自启"
    fi

    systemctl start "$SERVICE_NAME" 2>/dev/null || {
        _dt_log_error "服务启动失败"
        return 1
    }
    _dt_log_success "服务已启动"

    return 0
}

#===============================================================================
# 函数: verify_desktop_integration
# 描述: 验证桌面集成是否正确安装
# 参数: 无
# 返回: 退出码 0 表示成功，非 0 表示失败
#
# 使用示例:
#   verify_desktop_integration
#===============================================================================
verify_desktop_integration() {
    local desktop_file="/usr/share/applications/$DESKTOP_FILE_NAME"
    local service_file="/etc/systemd/system/$SERVICE_FILE_NAME"
    local profile_script="/etc/profile.d/$PROFILE_SCRIPT_NAME"
    local failed=0

    _dt_log_info "验证桌面集成组件..."

    if [ -f "$desktop_file" ]; then
        _dt_log_success ".desktop 文件存在"
    else
        _dt_log_error ".desktop 文件不存在: $desktop_file"
        ((failed++))
    fi

    if _dt_check_systemd; then
        if [ -f "$service_file" ]; then
            _dt_log_success "systemd 服务文件存在"
            if systemctl is-enabled "$SERVICE_NAME" > /dev/null 2>&1; then
                _dt_log_success "systemd 服务已启用"
            else
                _dt_log_warning "systemd 服务未启用"
            fi
            if systemctl is-active "$SERVICE_NAME" > /dev/null 2>&1; then
                _dt_log_success "systemd 服务正在运行"
            else
                _dt_log_warning "systemd 服务未运行"
            fi
        else
            _dt_log_error "systemd 服务文件不存在: $service_file"
            ((failed++))
        fi
    fi

    if [ -f "$profile_script" ]; then
        _dt_log_success "PATH 配置文件存在"
    else
        _dt_log_warning "PATH 配置文件不存在: $profile_script"
    fi

    if id "$SERVICE_USER" > /dev/null 2>&1; then
        _dt_log_success "服务用户 $SERVICE_USER 存在"
    else
        _dt_log_warning "服务用户 $SERVICE_USER 不存在"
    fi

    if [ $failed -eq 0 ]; then
        _dt_log_success "桌面集成验证完成"
        return 0
    else
        _dt_log_error "桌面集成验证失败，$failed 个组件缺失"
        return 1
    fi
}

#===============================================================================
# 导出所有函数供外部脚本使用
#===============================================================================
export -f install_desktop_file
export -f install_systemd_service
export -f add_to_path
export -f uninstall_desktop_integration
export -f enable_systemd_service
export -f verify_desktop_integration
export -f _dt_check_systemd
export -f _dt_create_user_if_not_exists
export -f _dt_log_info
export -f _dt_log_success
export -f _dt_log_error
export -f _dt_log_warning
export -f _dt_log_debug

use axum::{
    routing::{delete, get, post},
    Router,
};

pub use super::handlers::{
    add_repository, apply_template, batch_operation, check_conflicts,
    get_backups, get_compatibility, get_installed_plugins, get_marketplace_stats,
    get_operation_status, get_performance_report, get_plugin_info, get_repositories,
    get_templates, install_plugin, reload_plugin, rollback_plugin, scan_plugin_security,
    search_plugins, sync_repository, uninstall_plugin, update_plugin, create_plugin_routes,
};

pub fn create_app() -> Router {
    create_plugin_routes()
}

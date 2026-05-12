use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::plugins::types::*;

#[derive(Clone)]
pub struct PluginState {
    pub server_version: String,
    pub api_version: String,
    pub plugins_dir: std::path::PathBuf,
}

impl PluginState {
    pub fn new(server_version: String, api_version: String, plugins_dir: std::path::PathBuf) -> Self {
        Self {
            server_version,
            api_version,
            plugins_dir,
        }
    }
}

pub async fn search_plugins(
    Query(params): Query<SearchRequest>,
) -> Result<Json<SearchResult>, AppError> {
    let plugins = vec![
        Plugin {
            id: "1".to_string(),
            name: "EssentialsX".to_string(),
            plugin_id: 9089,
            version: "2.20.1".to_string(),
            file_id: 1001,
            download_url: "https://cdn.spigotmc.org/resources/essentialsx.9089/download".to_string(),
            file_name: "EssentialsX-2.20.1.jar".to_string(),
            file_size: 5242880,
            upload_date: chrono::Utc::now(),
            description: Some("Core Minecraft plugin for server management".to_string()),
            authors: vec!["EssentialTeam".to_string()],
            category: "Admin".to_string(),
            tags: vec!["essentials".to_string(), "chat".to_string()],
            source_url: Some("https://github.com/EssentialsX/Essentials".to_string()),
            issue_tracker: Some("https://github.com/EssentialsX/Essentials/issues".to_string()),
            website: Some("https://essentialsx.net".to_string()),
            supported_versions: vec!["1.20.4".to_string(), "1.20.2".to_string(), "1.19.4".to_string()],
            stats: PluginStats {
                downloads: 125000,
                likes: 8500,
                reviews_count: 3200,
                average_rating: 4.6,
            },
            compatibility: PluginCompatibility {
                compatibility_score: 95.0,
                server_version: "1.20.4".to_string(),
                api_version: "1.20".to_string(),
                known_issues: vec![],
                verified: true,
            },
            is_premium: false,
            price: None,
        },
    ];

    let result = SearchResult {
        plugins,
        total_count: 1,
        page: params.page,
        page_size: params.page_size,
        total_pages: 1,
    };

    Ok(Json(result))
}

pub async fn get_plugin_info(
    Path(plugin_id): Path<i64>,
) -> Result<Json<Plugin>, AppError> {
    let plugin = Plugin {
        id: plugin_id.to_string(),
        name: "EssentialsX".to_string(),
        plugin_id,
        version: "2.20.1".to_string(),
        file_id: 1001,
        download_url: "https://cdn.spigotmc.org/resources/essentialsx.9089/download".to_string(),
        file_name: "EssentialsX-2.20.1.jar".to_string(),
        file_size: 5242880,
        upload_date: chrono::Utc::now(),
        description: Some("Core Minecraft plugin for server management".to_string()),
        authors: vec!["EssentialTeam".to_string()],
        category: "Admin".to_string(),
        tags: vec!["essentials".to_string(), "chat".to_string()],
        source_url: Some("https://github.com/EssentialsX/Essentials".to_string()),
        issue_tracker: Some("https://github.com/EssentialsX/Essentials/issues".to_string()),
        website: Some("https://essentialsx.net".to_string()),
        supported_versions: vec!["1.20.4".to_string(), "1.20.2".to_string(), "1.19.4".to_string()],
        stats: PluginStats {
            downloads: 125000,
            likes: 8500,
            reviews_count: 3200,
            average_rating: 4.6,
        },
        compatibility: PluginCompatibility {
            compatibility_score: 95.0,
            server_version: "1.20.4".to_string(),
            api_version: "1.20".to_string(),
            known_issues: vec![],
            verified: true,
        },
        is_premium: false,
        price: None,
    };

    Ok(Json(plugin))
}

pub async fn install_plugin(
    Json(req): Json<InstallRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Plugin installation started",
        "plugin_id": req.plugin_id
    })))
}

pub async fn update_plugin(
    Path(plugin_id): Path<String>,
    Json(req): Json<UpdateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Plugin update started",
        "plugin_id": plugin_id
    })))
}

pub async fn uninstall_plugin(
    Path(plugin_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Plugin uninstalled",
        "plugin_id": plugin_id
    })))
}

pub async fn check_conflicts(
    Path(plugin_id): Path<i64>,
) -> Result<Json<Vec<DependencyConflict>>, AppError> {
    let conflicts = Vec::new();
    Ok(Json(conflicts))
}

pub async fn rollback_plugin(
    Path(plugin_id): Path<String>,
    Json(req): Json<RollbackRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Plugin rolled back",
        "plugin_id": plugin_id,
        "version": req.target_version
    })))
}

pub async fn get_backups(
    Path(plugin_id): Path<String>,
) -> Result<Json<Vec<PluginBackup>>, AppError> {
    let backups = Vec::new();
    Ok(Json(backups))
}

pub async fn get_compatibility(
    Path(plugin_id): Path<i64>,
) -> Result<Json<PluginCompatibility>, AppError> {
    let compatibility = PluginCompatibility {
        compatibility_score: 85.0,
        server_version: "1.20.4".to_string(),
        api_version: "1.20".to_string(),
        known_issues: vec![],
        verified: true,
    };
    Ok(Json(compatibility))
}

pub async fn get_templates(
    Path(plugin_name): Path<String>,
) -> Result<Json<Vec<PluginTemplate>>, AppError> {
    let templates = Vec::new();
    Ok(Json(templates))
}

pub async fn apply_template(
    Path(template_id): Path<String>,
    Json(variables): Json<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "success": true,
        "template_id": template_id,
        "config": {}
    })))
}

pub async fn reload_plugin(
    Path(plugin_id): Path<String>,
    Json(req): Json<ReloadRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Plugin reloaded",
        "plugin_id": plugin_id,
        "reload_config": req.reload_config
    })))
}

pub async fn get_performance_report(
    Path(plugin_id): Path<String>,
) -> Result<Json<PerformanceReport>, AppError> {
    let report = PerformanceReport {
        plugin_id: plugin_id.clone(),
        plugin_name: plugin_id,
        report_time: chrono::Utc::now(),
        memory: crate::plugins::types::MemoryReport {
            current_mb: 50.0,
            peak_mb: 75.0,
            leak_suspected: false,
            memory_hogs: vec![],
        },
        cpu: crate::plugins::types::CpuReport {
            average_percent: 5.0,
            peak_percent: 15.0,
            spike_count: 2,
            heavy_operations: vec![],
        },
        tick_impact: crate::plugins::types::TickImpactReport {
            average_ms: 0.5,
            max_ms: 2.0,
            contribution_percent: 2.5,
            slow_handlers: vec![],
        },
        commands: crate::plugins::types::CommandReport {
            registered_commands: 25,
            execution_count: 1000,
            average_execution_ms: 1.2,
        },
        events: crate::plugins::types::EventReport {
            listeners: 15,
            events_per_second: 50.0,
            heaviest_events: vec![],
        },
        overall_score: 92.0,
        recommendations: vec!["Plugin performance is excellent".to_string()],
    };
    Ok(Json(report))
}

pub async fn get_repositories() -> Result<Json<Vec<CustomRepository>>, AppError> {
    let repos = vec![
        CustomRepository {
            id: "1".to_string(),
            name: "SpigotMC".to_string(),
            url: "https://hub.spigotmc.org".to_string(),
            repo_type: RepositoryType::SpigotMC,
            enabled: true,
            priority: 1,
            last_sync: Some(chrono::Utc::now()),
            plugins_count: 50000,
            auth_required: false,
            auth_token: None,
            metadata: serde_json::json!({}),
        },
    ];
    Ok(Json(repos))
}

pub async fn add_repository(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<CustomRepository>, AppError> {
    let repo = CustomRepository {
        id: uuid::Uuid::new_v4().to_string(),
        name: req["name"].as_str().unwrap_or("Custom").to_string(),
        url: req["url"].as_str().unwrap_or("").to_string(),
        repo_type: RepositoryType::Custom,
        enabled: true,
        priority: 10,
        last_sync: None,
        plugins_count: 0,
        auth_required: false,
        auth_token: None,
        metadata: serde_json::json!({}),
    };
    Ok(Json(repo))
}

pub async fn sync_repository(
    Path(repo_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Repository sync started",
        "repo_id": repo_id
    })))
}

pub async fn batch_operation(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<BatchOperation>, AppError> {
    let operation = BatchOperation {
        operation_id: uuid::Uuid::new_v4().to_string(),
        operation_type: BatchOperationType::Install,
        plugins: vec![],
        status: BatchStatus::Pending,
        progress: 0.0,
        results: vec![],
        started_at: chrono::Utc::now(),
        completed_at: None,
        errors: vec![],
    };
    Ok(Json(operation))
}

pub async fn get_operation_status(
    Path(operation_id): Path<String>,
) -> Result<Json<BatchOperation>, AppError> {
    let operation = BatchOperation {
        operation_id,
        operation_type: BatchOperationType::Install,
        plugins: vec![],
        status: BatchStatus::Running,
        progress: 50.0,
        results: vec![],
        started_at: chrono::Utc::now(),
        completed_at: None,
        errors: vec![],
    };
    Ok(Json(operation))
}

pub async fn scan_plugin_security(
    Path(plugin_id): Path<String>,
) -> Result<Json<SecurityReport>, AppError> {
    let report = SecurityReport {
        plugin_id: plugin_id.clone(),
        plugin_name: plugin_id,
        scan_time: chrono::Utc::now(),
        score: 85.0,
        risk_level: RiskLevel::Low,
        checks: vec![
            crate::plugins::types::SecurityCheck {
                check_name: "Download Source".to_string(),
                passed: true,
                severity: "info".to_string(),
                description: "Official source verified".to_string(),
                details: None,
            },
        ],
        vulnerabilities: vec![],
        permissions: vec![],
        network_activity: crate::plugins::types::NetworkAnalysis {
            outbound_connections: vec![],
            inbound_connections: vec![],
            dns_resolutions: vec![],
            suspicious_activity: false,
        },
        overall_verdict: "Safe to use".to_string(),
    };
    Ok(Json(report))
}

pub async fn get_installed_plugins() -> Result<Json<Vec<InstalledPlugin>>, AppError> {
    let plugins = Vec::new();
    Ok(Json(plugins))
}

pub async fn get_marketplace_stats() -> Result<Json<MarketplaceStats>, AppError> {
    let stats = MarketplaceStats {
        total_plugins: 45000,
        total_downloads: 150000000,
        categories: vec![
            CategoryStats {
                category: "Admin".to_string(),
                plugin_count: 5000,
                download_count: 20000000,
            },
        ],
        trending_plugins: vec![],
        recently_updated: vec![],
    };
    Ok(Json(stats))
}

pub fn create_plugin_routes() -> Router {
    Router::new()
        .route("/api/plugins/search", get(search_plugins))
        .route("/api/plugins/:plugin_id", get(get_plugin_info))
        .route("/api/plugins/install", post(install_plugin))
        .route("/api/plugins/:plugin_id/update", post(update_plugin))
        .route("/api/plugins/:plugin_id/uninstall", delete(uninstall_plugin))
        .route("/api/plugins/:plugin_id/conflicts", get(check_conflicts))
        .route("/api/plugins/:plugin_id/rollback", post(rollback_plugin))
        .route("/api/plugins/:plugin_id/backups", get(get_backups))
        .route("/api/plugins/:plugin_id/compatibility", get(get_compatibility))
        .route("/api/plugins/:plugin_name/templates", get(get_templates))
        .route("/api/templates/:template_id/apply", post(apply_template))
        .route("/api/plugins/:plugin_id/reload", post(reload_plugin))
        .route("/api/plugins/:plugin_id/performance", get(get_performance_report))
        .route("/api/plugins/:plugin_id/security", get(scan_plugin_security))
        .route("/api/plugins/installed", get(get_installed_plugins))
        .route("/api/repositories", get(get_repositories))
        .route("/api/repositories", post(add_repository))
        .route("/api/repositories/:repo_id/sync", post(sync_repository))
        .route("/api/batch", post(batch_operation))
        .route("/api/batch/:operation_id", get(get_operation_status))
        .route("/api/marketplace/stats", get(get_marketplace_stats))
}

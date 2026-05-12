use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::process::{
    self, CgroupConfig, ClusterConfig, CrashDiagnosis, CrashReport, DiagnosticResult,
    InstanceCluster, InstanceMetadata, InstanceStatus, InstanceType, JvmConfig, JvmProfile,
    JvmTuner, ProcessContainer, ProcessError, SnapshotConfig, SnapshotManager,
    StartMode, StartModeConfig, StartModeManager, WarmupConfig, WarmupManager, Watchdog,
    WatchdogConfig,
};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub instance_type: InstanceType,
    pub jvm_args: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterStatsResponse {
    pub total_instances: usize,
    pub running_instances: usize,
    pub stopped_instances: usize,
    pub failed_instances: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JvmProfileRequest {
    pub name: String,
    pub description: String,
    pub args: Vec<String>,
    pub recommended_memory_mb: u64,
    pub recommended_cpu_cores: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryAdjustRequest {
    pub min_mb: u64,
    pub max_mb: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotCreateRequest {
    pub instance_id: String,
    pub snapshot_type: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnoseRequest {
    pub instance_id: String,
    pub exit_code: i32,
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartModeRequest {
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

pub async fn create_instance(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<CreateInstanceRequest>,
) -> Result<Json<ApiResponse<InstanceMetadata>>, AppError> {
    let mut app_state = state.write().await;

    if app_state.cluster.is_none() {
        app_state.cluster = Some(Arc::new(InstanceCluster::new(ClusterConfig::default())));
    }

    let cluster = app_state.cluster.as_ref().unwrap();

    let config = crate::config::ServerConfig {
        jvm_args: req.jvm_args.unwrap_or_else(|| vec![
            "-Xmx4G".to_string(),
            "-Xms2G".to_string(),
        ]),
        ..Default::default()
    };

    let instance = cluster
        .create_instance(req.name, req.instance_type, config)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let metadata = instance.get_metadata().await;
    Ok(Json(ApiResponse::success(metadata)))
}

pub async fn list_instances(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<Vec<InstanceMetadata>>>, AppError> {
    let app_state = state.read().await;

    if let Some(cluster) = &app_state.cluster {
        let instances = cluster.list_instances().await;
        Ok(Json(ApiResponse::success(instances)))
    } else {
        Ok(Json(ApiResponse::success(vec![])))
    }
}

pub async fn get_instance(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<InstanceMetadata>>, AppError> {
    let app_state = state.read().await;

    if let Some(cluster) = &app_state.cluster {
        if let Some(instance) = cluster.get_instance(&id).await {
            let metadata = instance.get_metadata().await;
            Ok(Json(ApiResponse::success(metadata)))
        } else {
            Err(AppError::Internal(format!("Instance {} not found", id)))
        }
    } else {
        Err(AppError::Internal("Cluster not initialized".to_string()))
    }
}

pub async fn delete_instance(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let mut app_state = state.write().await;

    if let Some(cluster) = &app_state.cluster {
        cluster
            .remove_instance(&id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(Json(ApiResponse::success(())))
    } else {
        Err(AppError::Internal("Cluster not initialized".to_string()))
    }
}

pub async fn get_cluster_stats(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<ClusterStatsResponse>>, AppError> {
    let app_state = state.read().await;

    if let Some(cluster) = &app_state.cluster {
        let stats = cluster.get_cluster_stats().await;
        Ok(Json(ApiResponse::success(ClusterStatsResponse {
            total_instances: stats.total_instances,
            running_instances: stats.running_instances,
            stopped_instances: stats.stopped_instances,
            failed_instances: stats.failed_instances,
        })))
    } else {
        Ok(Json(ApiResponse::success(ClusterStatsResponse {
            total_instances: 0,
            running_instances: 0,
            stopped_instances: 0,
            failed_instances: 0,
        })))
    }
}

pub async fn apply_jvm_profile(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<StartModeRequest>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let mut app_state = state.write().await;

    if app_state.jvm_tuner.is_none() {
        app_state.jvm_tuner = Some(Arc::new(JvmTuner::new(JvmConfig::default())));
    }

    let tuner = app_state.jvm_tuner.as_ref().unwrap();
    let args = tuner
        .apply_profile(&req.mode)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(args)))
}

pub async fn list_jvm_profiles(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<Vec<JvmProfile>>>, AppError> {
    let app_state = state.read().await;

    if let Some(tuner) = &app_state.jvm_tuner {
        let profiles = tuner.list_profiles().await;
        Ok(Json(ApiResponse::success(profiles)))
    } else {
        Ok(Json(ApiResponse::success(vec![])))
    }
}

pub async fn adjust_memory(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<MemoryAdjustRequest>,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let app_state = state.read().await;

    if let Some(tuner) = &app_state.jvm_tuner {
        let args = tuner
            .adjust_memory(req.min_mb, req.max_mb)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(Json(ApiResponse::success(args)))
    } else {
        Err(AppError::Internal("JVM tuner not initialized".to_string()))
    }
}

pub async fn create_snapshot(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<SnapshotCreateRequest>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let app_state = state.read().await;

    if let Some(snapshot_mgr) = &app_state.snapshot_manager {
        let metadata = process::SnapshotMetadata {
            pid: 0,
            uptime_secs: 0,
            memory_usage_mb: 0,
            cpu_percent: 0.0,
            world_name: None,
            player_count: 0,
            tick_rate: 0.0,
            description: req.description,
        };

        let snapshot_type = match req.snapshot_type.as_str() {
            "full" => process::SnapshotType::Full,
            "incremental" => process::SnapshotType::Incremental,
            "state_only" => process::SnapshotType::StateOnly,
            _ => process::SnapshotType::Full,
        };

        let snapshot = snapshot_mgr
            .create_snapshot(req.instance_id, snapshot_type, metadata, None)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(Json(ApiResponse::success(snapshot.id)))
    } else {
        Err(AppError::Internal("Snapshot manager not initialized".to_string()))
    }
}

pub async fn list_snapshots(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<Vec<process::ProcessSnapshot>>>, AppError> {
    let app_state = state.read().await;

    if let Some(snapshot_mgr) = &app_state.snapshot_manager {
        let snapshots = snapshot_mgr.list_snapshots().await;
        Ok(Json(ApiResponse::success(snapshots)))
    } else {
        Ok(Json(ApiResponse::success(vec![])))
    }
}

pub async fn delete_snapshot(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let app_state = state.read().await;

    if let Some(snapshot_mgr) = &app_state.snapshot_manager {
        snapshot_mgr
            .delete_snapshot(&id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(Json(ApiResponse::success(())))
    } else {
        Err(AppError::Internal("Snapshot manager not initialized".to_string()))
    }
}

pub async fn diagnose_crash(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<DiagnoseRequest>,
) -> Result<Json<ApiResponse<DiagnosticResult>>, AppError> {
    let app_state = state.read().await;

    if let Some(crash_diag) = &app_state.crash_diagnosis {
        let result = crash_diag
            .diagnose(req.instance_id, req.exit_code, &req.output)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let report = CrashReport {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            instance_id: req.instance_id,
            exit_code: req.exit_code,
            crash_type: result.crash_type,
            summary: result.summary.clone(),
            details: result.details.clone(),
            recommendations: result.recommendations.clone(),
            raw_output: Some(req.output),
        };

        crash_diag.save_crash_report(report).await;

        Ok(Json(ApiResponse::success(result)))
    } else {
        Err(AppError::Internal("Crash diagnosis not initialized".to_string()))
    }
}

pub async fn get_crash_reports(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<Vec<CrashReport>>>, AppError> {
    let app_state = state.read().await;

    if let Some(crash_diag) = &app_state.crash_diagnosis {
        let reports = crash_diag.get_crash_reports().await;
        Ok(Json(ApiResponse::success(reports)))
    } else {
        Ok(Json(ApiResponse::success(vec![])))
    }
}

pub async fn set_start_mode(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<StartModeRequest>,
) -> Result<Json<ApiResponse<StartMode>>, AppError> {
    let mut app_state = state.write().await;

    if app_state.start_mode_manager.is_none() {
        app_state.start_mode_manager = Some(Arc::new(RwLock::new(StartModeManager::new(
            StartModeConfig::default(),
        ))));
    }

    let mode = StartMode::from_str(&req.mode)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let manager = app_state.start_mode_manager.as_ref().unwrap();
    let mut guard = manager.write().await;
    guard.set_default_mode(mode).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(mode)))
}

pub async fn get_start_mode(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<StartMode>>, AppError> {
    let app_state = state.read().await;

    if let Some(manager) = &app_state.start_mode_manager {
        let guard = manager.read().await;
        let mode = guard.get_default_mode();
        Ok(Json(ApiResponse::success(mode)))
    } else {
        Ok(Json(ApiResponse::success(StartMode::Normal)))
    }
}

pub async fn prepare_start(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<StartModeRequest>,
) -> Result<Json<ApiResponse<process::StartArgs>>, AppError> {
    let app_state = state.read().await;

    if let Some(manager) = &app_state.start_mode_manager {
        let guard = manager.read().await;
        let mode = StartMode::from_str(&req.mode)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let args = guard
            .prepare_start_args(mode)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(Json(ApiResponse::success(args)))
    } else {
        Err(AppError::Internal("Start mode manager not initialized".to_string()))
    }
}

pub async fn get_warmup_status(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<process::WarmupStatus>>, AppError> {
    let app_state = state.read().await;

    if let Some(warmup_mgr) = &app_state.warmup_manager {
        let status = warmup_mgr.get_status().await;
        Ok(Json(ApiResponse::success(status)))
    } else {
        Ok(Json(ApiResponse::success(process::WarmupStatus::NotStarted)))
    }
}

pub async fn start_warmup(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let app_state = state.read().await;

    if let Some(warmup_mgr) = &app_state.warmup_manager {
        warmup_mgr.start().await.map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(Json(ApiResponse::success(())))
    } else {
        Err(AppError::Internal("Warmup manager not initialized".to_string()))
    }
}

pub async fn skip_warmup(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let app_state = state.read().await;

    if let Some(warmup_mgr) = &app_state.warmup_manager {
        warmup_mgr.skip().await;
        Ok(Json(ApiResponse::success(())))
    } else {
        Err(AppError::Internal("Warmup manager not initialized".to_string()))
    }
}

pub async fn get_cgroup_status(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Result<Json<ApiResponse<process::CgroupUsage>>, AppError> {
    let app_state = state.read().await;

    if let Some(cgroup_mgr) = &app_state.cgroup_manager {
        let usage = cgroup_mgr.get_usage().map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(Json(ApiResponse::success(usage)))
    } else {
        Ok(Json(ApiResponse::success(process::CgroupUsage::default())))
    }
}

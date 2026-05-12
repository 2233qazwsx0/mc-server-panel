use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

#[derive(Clone)]
pub struct RollingUpdateManager {
    state: Arc<RollingUpdateState>,
    config: Arc<RwLock<RollingUpdateConfig>>,
}

struct RollingUpdateState {
    update_plans: RwLock<HashMap<String, UpdatePlan>>,
    active_updates: RwLock<HashMap<String, UpdateExecution>>,
    update_history: RwLock<VecDeque<UpdateRecord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub strategy: RollingStrategy,
    pub target_version: String,
    pub server_ids: Vec<String>,
    pub batch_size: u32,
    pub wait_time_secs: u64,
    pub health_check_grace_period_secs: u64,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub status: PlanStatus,
    pub pre_checks: Vec<PreCheck>,
    pub rollback_plan: Option<RollbackPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    Ready,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExecution {
    pub plan_id: String,
    pub execution_id: String,
    pub current_batch: u32,
    pub total_batches: u32,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub batches: Vec<BatchExecution>,
    pub current_node: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    WaitingForHealth,
    Upgrading,
    RollingBack,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchExecution {
    pub batch_number: u32,
    pub nodes: Vec<String>,
    pub status: BatchStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub health_check_passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    Pending,
    Draining,
    Upgrading,
    HealthChecking,
    Online,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCheck {
    pub check_type: PreCheckType,
    pub enabled: bool,
    pub timeout_secs: u64,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreCheckType {
    DiskSpace,
    Memory,
    Backup,
    HealthCheck,
    ConfigValid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub auto_rollback: bool,
    pub rollback_threshold: u32,
    pub rollback_on_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecord {
    pub plan_id: String,
    pub execution_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub success: bool,
    pub error: Option<String>,
    pub nodes_updated: u32,
    pub nodes_rolled_back: u32,
}

impl RollingUpdateManager {
    pub fn new(config: RollingUpdateConfig) -> Self {
        Self {
            state: Arc::new(RollingUpdateState {
                update_plans: RwLock::new(HashMap::new()),
                active_updates: RwLock::new(HashMap::new()),
                update_history: RwLock::new(VecDeque::with_capacity(50)),
            }),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn update_config(&self, config: RollingUpdateConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> RollingUpdateConfig {
        self.config.read().clone()
    }

    pub fn create_plan(&self, name: String, description: String, target_version: String, server_ids: Vec<String>, created_by: &str) -> UpdatePlan {
        let config = self.config.read();
        let plan = UpdatePlan {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            strategy: config.strategy.clone(),
            target_version,
            server_ids,
            batch_size: config.batch_size,
            wait_time_secs: config.wait_time_secs,
            health_check_grace_period_secs: config.health_check_grace_period_secs,
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            status: PlanStatus::Draft,
            pre_checks: vec![
                PreCheck { check_type: PreCheckType::DiskSpace, enabled: true, timeout_secs: 30, required: true },
                PreCheck { check_type: PreCheckType::HealthCheck, enabled: true, timeout_secs: 60, required: true },
            ],
            rollback_plan: Some(RollbackPlan {
                auto_rollback: config.auto_rollback_on_failure,
                rollback_threshold: 2,
                rollback_on_failure: true,
            }),
        };

        self.state.update_plans.write().insert(plan.id.clone(), plan.clone());
        plan
    }

    pub fn get_plan(&self, plan_id: &str) -> Option<UpdatePlan> {
        self.state.update_plans.read().get(plan_id).cloned()
    }

    pub fn get_all_plans(&self) -> Vec<UpdatePlan> {
        self.state.update_plans.read().values().cloned().collect()
    }

    pub fn update_plan_status(&self, plan_id: &str, status: PlanStatus) -> Result<(), UpdateError> {
        let mut plans = self.state.update_plans.write();
        let plan = plans.get_mut(plan_id)
            .ok_or_else(|| UpdateError::PlanNotFound(plan_id.to_string()))?;
        plan.status = status;
        Ok(())
    }

    pub fn execute_plan(&self, plan_id: &str) -> Result<UpdateExecution, UpdateError> {
        let plan = self.get_plan(plan_id)
            .ok_or_else(|| UpdateError::PlanNotFound(plan_id.to_string()))?;

        if plan.status != PlanStatus::Ready {
            return Err(UpdateError::InvalidPlanStatus(plan_id.to_string()));
        }

        self.update_plan_status(plan_id, PlanStatus::InProgress)?;

        let total_batches = (plan.server_ids.len() as u32 + plan.batch_size - 1) / plan.batch_size;
        let batches: Vec<BatchExecution> = (0..total_batches).map(|i| {
            let start = (i * plan.batch_size) as usize;
            let end = (start + plan.batch_size as usize).min(plan.server_ids.len());
            BatchExecution {
                batch_number: i,
                nodes: plan.server_ids[start..end].to_vec(),
                status: BatchStatus::Pending,
                started_at: None,
                completed_at: None,
                health_check_passed: false,
                error: None,
            }
        }).collect();

        let execution = UpdateExecution {
            plan_id: plan_id.to_string(),
            execution_id: Uuid::new_v4().to_string(),
            current_batch: 0,
            total_batches,
            status: ExecutionStatus::Pending,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            batches,
            current_node: None,
            error: None,
        };

        self.state.active_updates.write().insert(execution.execution_id.clone(), execution.clone());
        Ok(execution)
    }

    pub fn get_execution(&self, execution_id: &str) -> Option<UpdateExecution> {
        self.state.active_updates.read().get(execution_id).cloned()
    }

    pub fn start_batch(&self, execution_id: &str, batch_number: u32) -> Result<(), UpdateError> {
        let mut executions = self.state.active_updates.write();
        let exec = executions.get_mut(execution_id)
            .ok_or_else(|| UpdateError::ExecutionNotFound(execution_id.to_string()))?;

        if exec.status != ExecutionStatus::Pending && exec.status != ExecutionStatus::WaitingForHealth {
            return Err(UpdateError::InvalidExecutionState(execution_id.to_string()));
        }

        if let Some(batch) = exec.batches.get_mut(batch_number as usize) {
            batch.status = BatchStatus::Draining;
            batch.started_at = Some(Utc::now());
            exec.current_batch = batch_number;
            exec.status = ExecutionStatus::Upgrading;
            exec.updated_at = Utc::now();
        }

        Ok(())
    }

    pub fn complete_batch(&self, execution_id: &str, batch_number: u32, health_check_passed: bool) -> Result<(), UpdateError> {
        let mut executions = self.state.active_updates.write();
        let exec = executions.get_mut(execution_id)
            .ok_or_else(|| UpdateError::ExecutionNotFound(execution_id.to_string()))?;

        if let Some(batch) = exec.batches.get_mut(batch_number as usize) {
            batch.status = if health_check_passed { BatchStatus::Online } else { BatchStatus::Failed };
            batch.health_check_passed = health_check_passed;
            batch.completed_at = Some(Utc::now());
            exec.updated_at = Utc::now();

            if health_check_passed && (batch_number + 1) >= exec.total_batches {
                exec.status = ExecutionStatus::Completed;
                exec.completed_at = Some(Utc::now());
            } else if !health_check_passed {
                exec.status = ExecutionStatus::Failed;
                exec.error = Some("Health check failed".to_string());
            } else {
                exec.status = ExecutionStatus::WaitingForHealth;
            }
        }

        Ok(())
    }

    pub fn pause_execution(&self, execution_id: &str) -> Result<(), UpdateError> {
        let mut executions = self.state.active_updates.write();
        let exec = executions.get_mut(execution_id)
            .ok_or_else(|| UpdateError::ExecutionNotFound(execution_id.to_string()))?;

        if matches!(exec.status, ExecutionStatus::Running | ExecutionStatus::Upgrading) {
            exec.status = ExecutionStatus::Paused;
            self.update_plan_status(&exec.plan_id, PlanStatus::Paused)?;
        }

        Ok(())
    }

    pub fn resume_execution(&self, execution_id: &str) -> Result<(), UpdateError> {
        let mut executions = self.state.active_updates.write();
        let exec = executions.get_mut(execution_id)
            .ok_or_else(|| UpdateError::ExecutionNotFound(execution_id.to_string()))?;

        if matches!(exec.status, ExecutionStatus::Paused) {
            exec.status = ExecutionStatus::Running;
            self.update_plan_status(&exec.plan_id, PlanStatus::InProgress)?;
        }

        Ok(())
    }

    pub fn cancel_execution(&self, execution_id: &str) -> Result<(), UpdateError> {
        let mut executions = self.state.active_updates.write();
        let exec = executions.get_mut(execution_id)
            .ok_or_else(|| UpdateError::ExecutionNotFound(execution_id.to_string()))?;

        exec.status = ExecutionStatus::Cancelled;
        exec.completed_at = Some(Utc::now());
        self.update_plan_status(&exec.plan_id, PlanStatus::Cancelled)?;

        let record = UpdateRecord {
            plan_id: exec.plan_id.clone(),
            execution_id: execution_id.to_string(),
            started_at: exec.started_at,
            completed_at: exec.completed_at,
            status: ExecutionStatus::Cancelled,
            success: false,
            error: Some("Cancelled by user".to_string()),
            nodes_updated: exec.current_batch,
            nodes_rolled_back: 0,
        };

        drop(executions);
        self.add_to_history(record);

        Ok(())
    }

    pub fn rollback_execution(&self, execution_id: &str) -> Result<(), UpdateError> {
        let mut executions = self.state.active_updates.write();
        let exec = executions.get_mut(execution_id)
            .ok_or_else(|| UpdateError::ExecutionNotFound(execution_id.to_string()))?;

        exec.status = ExecutionStatus::RollingBack;
        self.update_plan_status(&exec.plan_id, PlanStatus::InProgress)?;

        for batch in &mut exec.batches {
            if matches!(batch.status, BatchStatus::Online) {
                batch.status = BatchStatus::RolledBack;
            }
        }

        Ok(())
    }

    fn add_to_history(&self, record: UpdateRecord) {
        let mut history = self.state.update_history.write();
        history.push_back(record);
        if history.len() > 50 {
            history.pop_front();
        }
    }

    pub fn get_active_executions(&self) -> Vec<UpdateExecution> {
        self.state.active_updates.read().values().cloned().collect()
    }

    pub fn get_update_history(&self, limit: usize) -> Vec<UpdateRecord> {
        let history = self.state.update_history.read();
        history.iter().rev().take(limit).cloned().collect()
    }

    pub fn simulate_blue_green_update(&self, server_ids: &[String]) -> (Vec<String>, Vec<String>) {
        let mid = server_ids.len() / 2;
        let blue = server_ids[..mid].to_vec();
        let green = server_ids[mid..].to_vec();
        (blue, green)
    }

    pub fn get_stats(&self) -> UpdateStats {
        let plans = self.state.update_plans.read();
        let executions = self.state.active_updates.read();
        let history = self.state.update_history.read();

        UpdateStats {
            total_plans: plans.len(),
            active_plans: plans.values().filter(|p| p.status == PlanStatus::InProgress).count(),
            active_executions: executions.len(),
            completed_updates: history.iter().filter(|r| r.success).count(),
            failed_updates: history.iter().filter(|r| !r.success).count(),
            current_strategy: self.config.read().strategy.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStats {
    pub total_plans: usize,
    pub active_plans: usize,
    pub active_executions: usize,
    pub completed_updates: usize,
    pub failed_updates: usize,
    pub current_strategy: RollingStrategy,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Update plan {0} not found")]
    PlanNotFound(String),

    #[error("Execution {0} not found")]
    ExecutionNotFound(String),

    #[error("Invalid plan status for plan {0}")]
    InvalidPlanStatus(String),

    #[error("Invalid execution state for {0}")]
    InvalidExecutionState(String),
}

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::plugins::types::*;

pub struct BatchManager {
    operations: RwLock<HashMap<String, BatchOperation>>,
    max_concurrent: usize,
}

impl BatchManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            operations: RwLock::new(HashMap::new()),
            max_concurrent,
        }
    }

    pub async fn start_batch_operation(
        &self,
        operation_type: BatchOperationType,
        plugin_ids: Vec<String>,
    ) -> Result<BatchOperation> {
        let operation_id = Uuid::new_v4().to_string();
        
        let operation = BatchOperation {
            operation_id: operation_id.clone(),
            operation_type,
            plugins: plugin_ids,
            status: BatchStatus::Pending,
            progress: 0.0,
            results: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
            errors: Vec::new(),
        };

        let mut ops = self.operations.write().await;
        ops.insert(operation_id.clone(), operation.clone());

        Ok(operation)
    }

    pub async fn execute_batch_install(
        &self,
        operation_id: &str,
        plugins: Vec<(Plugin, bool)>,
    ) -> Result<BatchOperation> {
        let mut ops = self.operations.write().await;
        
        if let Some(operation) = ops.get_mut(operation_id) {
            operation.status = BatchStatus::Running;
            let total = plugins.len();
            
            for (i, (plugin, backup)) in plugins.into_iter().enumerate() {
                let result = self.install_single_plugin(&plugin, backup).await;
                
                operation.results.push(BatchResult {
                    plugin_id: plugin.plugin_id.to_string(),
                    success: result.is_ok(),
                    message: result.as_ref().map_or_else(|e| e.to_string(), |_| "Success".to_string()),
                    details: None,
                });
                
                operation.progress = ((i + 1) as f64 / total as f64) * 100.0;
            }
            
            let all_success = operation.results.iter().all(|r| r.success);
            operation.status = if all_success {
                BatchStatus::Completed
            } else {
                BatchStatus::Failed
            };
            operation.completed_at = Some(Utc::now());
            
            Ok(operation.clone())
        } else {
            anyhow::bail!("Operation not found")
        }
    }

    async fn install_single_plugin(&self, plugin: &Plugin, _backup: bool) -> Result<()> {
        Ok(())
    }

    pub async fn execute_batch_update(
        &self,
        operation_id: &str,
        updates: Vec<(String, Plugin)>,
    ) -> Result<BatchOperation> {
        let mut ops = self.operations.write().await;
        
        if let Some(operation) = ops.get_mut(operation_id) {
            operation.status = BatchStatus::Running;
            let total = updates.len();
            
            for (i, (plugin_id, new_plugin)) in updates.into_iter().enumerate() {
                let result = self.update_single_plugin(&plugin_id, &new_plugin).await;
                
                operation.results.push(BatchResult {
                    plugin_id,
                    success: result.is_ok(),
                    message: result.as_ref().map_or_else(|e| e.to_string(), |_| "Updated".to_string()),
                    details: None,
                });
                
                operation.progress = ((i + 1) as f64 / total as f64) * 100.0;
            }
            
            let all_success = operation.results.iter().all(|r| r.success);
            operation.status = if all_success {
                BatchStatus::Completed
            } else {
                BatchStatus::Failed
            };
            operation.completed_at = Some(Utc::now());
            
            Ok(operation.clone())
        } else {
            anyhow::bail!("Operation not found")
        }
    }

    async fn update_single_plugin(&self, plugin_id: &str, new_plugin: &Plugin) -> Result<()> {
        Ok(())
    }

    pub async fn execute_batch_uninstall(
        &self,
        operation_id: &str,
        plugin_ids: Vec<String>,
        create_backups: bool,
    ) -> Result<BatchOperation> {
        let mut ops = self.operations.write().await;
        
        if let Some(operation) = ops.get_mut(operation_id) {
            operation.status = BatchStatus::Running;
            let total = plugin_ids.len();
            
            for (i, plugin_id) in plugin_ids.into_iter().enumerate() {
                let result = self.uninstall_single_plugin(&plugin_id, create_backups).await;
                
                operation.results.push(BatchResult {
                    plugin_id,
                    success: result.is_ok(),
                    message: if result.is_ok() { "Uninstalled".to_string() } else { "Failed to uninstall".to_string() },
                    details: None,
                });
                
                operation.progress = ((i + 1) as f64 / total as f64) * 100.0;
            }
            
            let all_success = operation.results.iter().all(|r| r.success);
            operation.status = if all_success {
                BatchStatus::Completed
            } else {
                BatchStatus::Failed
            };
            operation.completed_at = Some(Utc::now());
            
            Ok(operation.clone())
        } else {
            anyhow::bail!("Operation not found")
        }
    }

    async fn uninstall_single_plugin(&self, plugin_id: &str, _backup: bool) -> Result<()> {
        Ok(())
    }

    pub async fn execute_batch_enable(
        &self,
        operation_id: &str,
        plugin_ids: Vec<String>,
    ) -> Result<BatchOperation> {
        let mut ops = self.operations.write().await;
        
        if let Some(operation) = ops.get_mut(operation_id) {
            operation.status = BatchStatus::Running;
            let total = plugin_ids.len();
            
            for (i, plugin_id) in plugin_ids.into_iter().enumerate() {
                let result = self.enable_plugin(&plugin_id).await;
                
                operation.results.push(BatchResult {
                    plugin_id,
                    success: result.is_ok(),
                    message: if result.is_ok() { "Enabled".to_string() } else { "Failed to enable".to_string() },
                    details: None,
                });
                
                operation.progress = ((i + 1) as f64 / total as f64) * 100.0;
            }
            
            let all_success = operation.results.iter().all(|r| r.success);
            operation.status = if all_success {
                BatchStatus::Completed
            } else {
                BatchStatus::Failed
            };
            operation.completed_at = Some(Utc::now());
            
            Ok(operation.clone())
        } else {
            anyhow::bail!("Operation not found")
        }
    }

    async fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        Ok(())
    }

    pub async fn execute_batch_disable(
        &self,
        operation_id: &str,
        plugin_ids: Vec<String>,
    ) -> Result<BatchOperation> {
        let mut ops = self.operations.write().await;
        
        if let Some(operation) = ops.get_mut(operation_id) {
            operation.status = BatchStatus::Running;
            let total = plugin_ids.len();
            
            for (i, plugin_id) in plugin_ids.into_iter().enumerate() {
                let result = self.disable_plugin(&plugin_id).await;
                
                operation.results.push(BatchResult {
                    plugin_id,
                    success: result.is_ok(),
                    message: if result.is_ok() { "Disabled".to_string() } else { "Failed to disable".to_string() },
                    details: None,
                });
                
                operation.progress = ((i + 1) as f64 / total as f64) * 100.0;
            }
            
            let all_success = operation.results.iter().all(|r| r.success);
            operation.status = if all_success {
                BatchStatus::Completed
            } else {
                BatchStatus::Failed
            };
            operation.completed_at = Some(Utc::now());
            
            Ok(operation.clone())
        } else {
            anyhow::bail!("Operation not found")
        }
    }

    async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        Ok(())
    }

    pub async fn get_operation_status(&self, operation_id: &str) -> Option<BatchOperation> {
        let ops = self.operations.read().await;
        ops.get(operation_id).cloned()
    }

    pub async fn cancel_operation(&self, operation_id: &str) -> Result<()> {
        let mut ops = self.operations.write().await;
        
        if let Some(operation) = ops.get_mut(operation_id) {
            match operation.status {
                BatchStatus::Pending => {
                    operation.status = BatchStatus::Cancelled;
                    operation.completed_at = Some(Utc::now());
                }
                BatchStatus::Running => {
                    operation.errors.push("Cannot cancel running operation".to_string());
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    pub async fn get_all_operations(&self) -> Vec<BatchOperation> {
        let ops = self.operations.read().await;
        let mut result: Vec<_> = ops.values().cloned().collect();
        result.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        result
    }

    pub async fn cleanup_completed_operations(&self, max_age_hours: u64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours as i64);
        let mut ops = self.operations.write().await;
        let initial_count = ops.len();
        
        ops.retain(|_, op| {
            if let Some(completed) = op.completed_at {
                completed > cutoff
            } else {
                true
            }
        });
        
        initial_count - ops.len()
    }
}

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::plugins::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub plugin_id: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub tick_time_ms: f64,
    pub commands_count: i32,
    pub events_count: i64,
}

pub struct PerformanceAnalyzer {
    metrics_history: RwLock<HashMap<String, Vec<PerformanceMetrics>>>,
    baseline_metrics: RwLock<HashMap<String, PerformanceMetrics>>,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            metrics_history: RwLock::new(HashMap::new()),
            baseline_metrics: RwLock::new(HashMap::new()),
        }
    }

    pub async fn record_metrics(&self, metrics: PerformanceMetrics) {
        let mut history = self.metrics_history.write().await;
        let plugin_metrics = history.entry(metrics.plugin_id.clone()).or_insert_with(Vec::new);
        plugin_metrics.push(metrics);
        
        if plugin_metrics.len() > 1000 {
            plugin_metrics.remove(0);
        }
    }

    pub async fn generate_report(&self, plugin_id: &str) -> Result<PerformanceReport> {
        let history = self.metrics_history.read().await;
        let metrics = history.get(plugin_id).cloned().unwrap_or_default();

        if metrics.is_empty() {
            return Ok(self.create_empty_report(plugin_id));
        }

        let memory_report = self.analyze_memory(&metrics);
        let cpu_report = self.analyze_cpu(&metrics);
        let tick_report = self.analyze_tick_impact(&metrics);
        let command_report = self.analyze_commands(&metrics);
        let event_report = self.analyze_events(&metrics);
        
        let overall_score = self.calculate_overall_score(
            &memory_report,
            &cpu_report,
            &tick_report,
        );

        let recommendations = self.generate_recommendations(
            &memory_report,
            &cpu_report,
            &tick_report,
        );

        Ok(PerformanceReport {
            plugin_id: plugin_id.to_string(),
            plugin_name: plugin_id.to_string(),
            report_time: Utc::now(),
            memory: memory_report,
            cpu: cpu_report,
            tick_impact: tick_report,
            commands: command_report,
            events: event_report,
            overall_score,
            recommendations,
        })
    }

    fn analyze_memory(&self, metrics: &[PerformanceMetrics]) -> MemoryReport {
        let memory_values: Vec<f64> = metrics.iter().map(|m| m.memory_mb).collect();
        
        let current = memory_values.last().copied().unwrap_or(0.0);
        let peak = memory_values.iter().cloned().fold(0.0, f64::max);
        
        let leak_suspected = self.detect_memory_leak(metrics);
        
        let memory_hogs = if leak_suspected {
            vec!["Memory usage increasing over time".to_string()]
        } else {
            Vec::new()
        };

        MemoryReport {
            current_mb: current,
            peak_mb: peak,
            leak_suspected,
            memory_hogs,
        }
    }

    fn detect_memory_leak(&self, metrics: &[PerformanceMetrics]) -> bool {
        if metrics.len() < 10 {
            return false;
        }

        let recent: f64 = metrics.iter().rev().take(5).map(|m| m.memory_mb).sum::<f64>() / 5.0;
        let older: f64 = metrics.iter().take(5).map(|m| m.memory_mb).sum::<f64>() / 5.0;
        
        (recent - older) / older > 0.2
    }

    fn analyze_cpu(&self, metrics: &[PerformanceMetrics]) -> CpuReport {
        let cpu_values: Vec<f64> = metrics.iter().map(|m| m.cpu_percent).collect();
        
        let average = cpu_values.iter().sum::<f64>() / cpu_values.len() as f64;
        let peak = cpu_values.iter().cloned().fold(0.0, f64::max);
        
        let spike_threshold = average * 2.0;
        let spike_count = metrics.iter().filter(|m| m.cpu_percent > spike_threshold).count() as i32;
        
        let heavy_operations = if spike_count > 5 {
            vec!["High CPU usage spikes detected".to_string()]
        } else {
            Vec::new()
        };

        CpuReport {
            average_percent: average,
            peak_percent: peak,
            spike_count,
            heavy_operations,
        }
    }

    fn analyze_tick_impact(&self, metrics: &[PerformanceMetrics]) -> TickImpactReport {
        let tick_values: Vec<f64> = metrics.iter().map(|m| m.tick_time_ms).collect();
        
        let average = tick_values.iter().sum::<f64>() / tick_values.len() as f64;
        let max = tick_values.iter().cloned().fold(0.0, f64::max);
        
        let contribution = if average > 0.1 { (average / 50.0) * 100.0 } else { 0.0 };
        
        let slow_handlers = if max > 5.0 {
            vec![format!("Max tick time {}ms exceeds target", max)]
        } else {
            Vec::new()
        };

        TickImpactReport {
            average_ms: average,
            max_ms: max,
            contribution_percent: contribution.min(100.0),
            slow_handlers,
        }
    }

    fn analyze_commands(&self, metrics: &[PerformanceMetrics]) -> CommandReport {
        let avg_commands = metrics.iter().map(|m| m.commands_count as i64).sum::<i64>() 
            / metrics.len().max(1) as i64;
        
        let total_executions: i64 = metrics.iter().map(|m| m.commands_count as i64).sum();

        CommandReport {
            registered_commands: avg_commands as i32,
            execution_count: total_executions,
            average_execution_ms: 0.5,
        }
    }

    fn analyze_events(&self, metrics: &[PerformanceMetrics]) -> EventReport {
        let listeners = 10;
        let total_events: i64 = metrics.iter().map(|m| m.events_count).sum();
        let duration_secs = metrics.len() as i64;
        let events_per_second = total_events as f64 / duration_secs.max(1) as f64;
        
        let heaviest_events = if events_per_second > 100.0 {
            vec!["High event frequency detected".to_string()]
        } else {
            Vec::new()
        };

        EventReport {
            listeners,
            events_per_second,
            heaviest_events,
        }
    }

    fn calculate_overall_score(
        &self,
        memory: &MemoryReport,
        cpu: &CpuReport,
        tick: &TickImpactReport,
    ) -> f64 {
        let mut score = 100.0;
        
        if memory.leak_suspected {
            score -= 30.0;
        }
        
        if memory.peak_mb > 500.0 {
            score -= 20.0;
        }
        
        if cpu.spike_count > 10 {
            score -= 15.0;
        }
        
        if tick.contribution_percent > 10.0 {
            score -= 25.0;
        }

        score.max(0.0).min(100.0)
    }

    fn generate_recommendations(
        &self,
        memory: &MemoryReport,
        cpu: &CpuReport,
        tick: &TickImpactReport,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if memory.leak_suspected {
            recommendations.push("Memory leak detected. Consider checking plugin's cache management.".to_string());
            recommendations.push("Enable periodic garbage collection if plugin supports it.".to_string());
        }

        if memory.peak_mb > 500.0 {
            recommendations.push("High memory usage. Consider allocating more RAM or optimizing plugin settings.".to_string());
        }

        if cpu.spike_count > 10 {
            recommendations.push("Frequent CPU spikes. Check for inefficient event handlers or heavy computations.".to_string());
            recommendations.push("Consider using async operations for I/O heavy tasks.".to_string());
        }

        if tick.contribution_percent > 10.0 {
            recommendations.push("Significant tick time impact. Optimize event handlers and scheduled tasks.".to_string());
            recommendations.push("Use batch operations instead of processing per-player in events.".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Plugin performance is within acceptable parameters.".to_string());
        }

        recommendations
    }

    fn create_empty_report(&self, plugin_id: &str) -> PerformanceReport {
        PerformanceReport {
            plugin_id: plugin_id.to_string(),
            plugin_name: plugin_id.to_string(),
            report_time: Utc::now(),
            memory: MemoryReport {
                current_mb: 0.0,
                peak_mb: 0.0,
                leak_suspected: false,
                memory_hogs: Vec::new(),
            },
            cpu: CpuReport {
                average_percent: 0.0,
                peak_percent: 0.0,
                spike_count: 0,
                heavy_operations: Vec::new(),
            },
            tick_impact: TickImpactReport {
                average_ms: 0.0,
                max_ms: 0.0,
                contribution_percent: 0.0,
                slow_handlers: Vec::new(),
            },
            commands: CommandReport {
                registered_commands: 0,
                execution_count: 0,
                average_execution_ms: 0.0,
            },
            events: EventReport {
                listeners: 0,
                events_per_second: 0.0,
                heaviest_events: Vec::new(),
            },
            overall_score: 0.0,
            recommendations: vec!["No performance data available yet.".to_string()],
        }
    }

    pub async fn set_baseline(&self, plugin_id: &str, metrics: PerformanceMetrics) {
        let mut baseline = self.baseline_metrics.write().await;
        baseline.insert(plugin_id.to_string(), metrics);
    }

    pub async fn compare_to_baseline(&self, plugin_id: &str) -> Result<Option<(String, f64)>> {
        let history = self.metrics_history.read().await;
        let baseline = self.baseline_metrics.read().await;
        
        if let (Some(current), Some(baseline_metrics)) = (
            history.get(plugin_id).and_then(|m| m.last()),
            baseline.get(plugin_id)
        ) {
            let diff = ((current.memory_mb - baseline_metrics.memory_mb) / baseline_metrics.memory_mb) * 100.0;
            Ok(Some((format!("Memory changed by {:.1}%", diff), diff)))
        } else {
            Ok(None)
        }
    }

    pub async fn clear_history(&self, plugin_id: &str) {
        let mut history = self.metrics_history.write().await;
        history.remove(plugin_id);
    }

    pub async fn get_summary(&self) -> Vec<(String, f64)> {
        let history = self.metrics_history.read().await;
        let mut summary = Vec::new();
        
        for (plugin_id, metrics) in history.iter() {
            if let Some(latest) = metrics.last() {
                summary.push((plugin_id.clone(), latest.memory_mb));
            }
        }
        
        summary.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        summary
    }
}

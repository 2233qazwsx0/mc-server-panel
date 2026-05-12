use crate::monitor::types::{MetricType, AlertLevel};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub widgets: Vec<Widget>,
    pub layout: DashboardLayout,
    pub refresh_interval_secs: u64,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub columns: u32,
    pub rows: u32,
    pub widget_positions: HashMap<String, WidgetPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub widget_type: WidgetType,
    pub title: String,
    pub metrics: Vec<WidgetMetric>,
    pub visualization: VisualizationConfig,
    pub size: WidgetSize,
    pub refresh_interval_secs: u64,
    pub alert_config: Option<WidgetAlertConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    LineChart,
    AreaChart,
    BarChart,
    Gauge,
    StatCard,
    AlertList,
    Table,
    Heatmap,
    Counter,
    ProgressBar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetMetric {
    pub metric_type: MetricType,
    pub label: String,
    pub color: Option<String>,
    pub aggregation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub visualization_type: WidgetType,
    pub show_legend: bool,
    pub show_grid: bool,
    pub show_axis_labels: bool,
    pub color_scheme: Option<String>,
    pub thresholds: Option<Vec<VisualizationThreshold>>,
    pub time_range_secs: u64,
    pub animations_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationThreshold {
    pub value: f64,
    pub color: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetAlertConfig {
    pub enabled: bool,
    pub warning_threshold: Option<f64>,
    pub critical_threshold: Option<f64>,
    pub comparison_operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub dashboard_id: String,
    pub timestamp: DateTime<Utc>,
    pub widget_data: HashMap<String, WidgetData>,
    pub active_alerts: Vec<AlertSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    pub widget_id: String,
    pub values: Vec<MetricValue>,
    pub chart_data: Option<ChartData>,
    pub stats: WidgetStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub metric_type: MetricType,
    pub value: f64,
    pub formatted_value: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetStats {
    pub current: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub change_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub id: String,
    pub level: AlertLevel,
    pub metric_type: MetricType,
    pub message: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: DashboardCategory,
    pub dashboard: Dashboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardCategory {
    Overview,
    Performance,
    Network,
    Security,
    Custom,
}

#[derive(Clone)]
pub struct DashboardManager {
    dashboards: Arc<RwLock<HashMap<String, Dashboard>>>,
    templates: Arc<RwLock<HashMap<String, DashboardTemplate>>>,
    snapshots: Arc<RwLock<VecDeque<DashboardSnapshot>>>,
    max_snapshots: usize,
}

impl DashboardManager {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(max_snapshots))),
            max_snapshots,
        }
    }

    pub fn with_default() -> Self {
        Self::new(100)
    }

    pub fn create_dashboard(&self, name: &str, description: &str, created_by: &str) -> Dashboard {
        let dashboard = Dashboard {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            widgets: Vec::new(),
            layout: DashboardLayout {
                columns: 12,
                rows: 6,
                widget_positions: HashMap::new(),
            },
            refresh_interval_secs: 5,
            is_default: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: created_by.to_string(),
        };

        let mut dashboards = self.dashboards.write();
        dashboards.insert(dashboard.id.clone(), dashboard.clone());

        dashboard
    }

    pub fn get_dashboard(&self, id: &str) -> Option<Dashboard> {
        let dashboards = self.dashboards.read();
        dashboards.get(id).cloned()
    }

    pub fn get_all_dashboards(&self) -> Vec<Dashboard> {
        let dashboards = self.dashboards.read();
        dashboards.values().cloned().collect()
    }

    pub fn update_dashboard(&self, dashboard: Dashboard) -> Result<(), String> {
        let mut dashboards = self.dashboards.write();
        if !dashboards.contains_key(&dashboard.id) {
            return Err("Dashboard not found".to_string());
        }

        let mut updated = dashboard;
        updated.updated_at = Utc::now();
        dashboards.insert(updated.id.clone(), updated);
        Ok(())
    }

    pub fn delete_dashboard(&self, id: &str) -> Result<Dashboard, String> {
        let mut dashboards = self.dashboards.write();
        dashboards.remove(id).ok_or_else(|| "Dashboard not found".to_string())
    }

    pub fn add_widget(&self, dashboard_id: &str, widget: Widget) -> Result<Dashboard, String> {
        let mut dashboards = self.dashboards.write();
        let dashboard = dashboards.get_mut(dashboard_id).ok_or_else(|| "Dashboard not found".to_string())?;

        let mut new_widget = widget;
        new_widget.id = Uuid::new_v4().to_string();
        dashboard.widgets.push(new_widget.clone());
        dashboard.updated_at = Utc::now();

        Ok(dashboard.clone())
    }

    pub fn remove_widget(&self, dashboard_id: &str, widget_id: &str) -> Result<Dashboard, String> {
        let mut dashboards = self.dashboards.write();
        let dashboard = dashboards.get_mut(dashboard_id).ok_or_else(|| "Dashboard not found".to_string())?;

        dashboard.widgets.retain(|w| w.id != widget_id);
        dashboard.layout.widget_positions.remove(widget_id);
        dashboard.updated_at = Utc::now();

        Ok(dashboard.clone())
    }

    pub fn update_widget(&self, dashboard_id: &str, widget: Widget) -> Result<Widget, String> {
        let mut dashboards = self.dashboards.write();
        let dashboard = dashboards.get_mut(dashboard_id).ok_or_else(|| "Dashboard not found".to_string())?;

        let idx = dashboard.widgets.iter().position(|w| w.id == widget.id)
            .ok_or_else(|| "Widget not found".to_string())?;

        dashboard.widgets[idx] = widget.clone();
        dashboard.updated_at = Utc::now();

        Ok(widget)
    }

    pub fn set_default_dashboard(&self, id: &str) -> Result<(), String> {
        let mut dashboards = self.dashboards.write();

        for dashboard in dashboards.values_mut() {
            dashboard.is_default = dashboard.id == id;
        }

        Ok(())
    }

    pub fn get_default_dashboard(&self) -> Option<Dashboard> {
        let dashboards = self.dashboards.read();
        dashboards.values().find(|d| d.is_default).cloned()
    }

    pub fn create_from_template(&self, template_id: &str, name: &str, created_by: &str) -> Result<Dashboard, String> {
        let templates = self.templates.read();
        let template = templates.get(template_id).ok_or_else(|| "Template not found".to_string())?;

        let mut dashboard = template.dashboard.clone();
        dashboard.id = Uuid::new_v4().to_string();
        dashboard.name = name.to_string();
        dashboard.is_default = false;
        dashboard.created_at = Utc::now();
        dashboard.updated_at = Utc::now();
        dashboard.created_by = created_by.to_string();

        for widget in &mut dashboard.widgets {
            widget.id = Uuid::new_v4().to_string();
        }

        let mut dashboards = self.dashboards.write();
        dashboards.insert(dashboard.id.clone(), dashboard.clone());

        Ok(dashboard)
    }

    pub fn register_template(&self, template: DashboardTemplate) {
        let mut templates = self.templates.write();
        templates.insert(template.id.clone(), template);
    }

    pub fn get_all_templates(&self) -> Vec<DashboardTemplate> {
        let templates = self.templates.read();
        templates.values().cloned().collect()
    }

    pub fn record_snapshot(&self, snapshot: DashboardSnapshot) {
        let mut snapshots = self.snapshots.write();
        if snapshots.len() >= self.max_snapshots {
            snapshots.pop_front();
        }
        snapshots.push_back(snapshot);
    }

    pub fn get_snapshot_history(&self, dashboard_id: &str, limit: usize) -> Vec<DashboardSnapshot> {
        let snapshots = self.snapshots.read();
        snapshots
            .iter()
            .filter(|s| s.dashboard_id == dashboard_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn create_overview_template() -> DashboardTemplate {
        let cpu_widget = Widget {
            id: "cpu_chart".to_string(),
            widget_type: WidgetType::LineChart,
            title: "CPU Usage".to_string(),
            metrics: vec![WidgetMetric {
                metric_type: MetricType::CpuUsage,
                label: "CPU %".to_string(),
                color: Some("#4CAF50".to_string()),
                aggregation: Some("avg".to_string()),
            }],
            visualization: VisualizationConfig {
                visualization_type: WidgetType::LineChart,
                show_legend: true,
                show_grid: true,
                show_axis_labels: true,
                color_scheme: Some("default".to_string()),
                thresholds: Some(vec![
                    VisualizationThreshold {
                        value: 80.0,
                        color: "#FFC107".to_string(),
                        label: "Warning".to_string(),
                    },
                    VisualizationThreshold {
                        value: 95.0,
                        color: "#F44336".to_string(),
                        label: "Critical".to_string(),
                    },
                ]),
                time_range_secs: 300,
                animations_enabled: true,
            },
            size: WidgetSize {
                width: 6,
                height: 4,
                min_width: 4,
                min_height: 3,
            },
            refresh_interval_secs: 5,
            alert_config: Some(WidgetAlertConfig {
                enabled: true,
                warning_threshold: Some(80.0),
                critical_threshold: Some(95.0),
                comparison_operator: "greater_than".to_string(),
            }),
        };

        let memory_widget = Widget {
            id: "memory_chart".to_string(),
            widget_type: WidgetType::AreaChart,
            title: "Memory Usage".to_string(),
            metrics: vec![WidgetMetric {
                metric_type: MetricType::MemoryPercent,
                label: "Memory %".to_string(),
                color: Some("#2196F3".to_string()),
                aggregation: Some("avg".to_string()),
            }],
            visualization: VisualizationConfig {
                visualization_type: WidgetType::AreaChart,
                show_legend: true,
                show_grid: true,
                show_axis_labels: true,
                color_scheme: Some("default".to_string()),
                thresholds: Some(vec![
                    VisualizationThreshold {
                        value: 80.0,
                        color: "#FFC107".to_string(),
                        label: "Warning".to_string(),
                    },
                    VisualizationThreshold {
                        value: 95.0,
                        color: "#F44336".to_string(),
                        label: "Critical".to_string(),
                    },
                ]),
                time_range_secs: 300,
                animations_enabled: true,
            },
            size: WidgetSize {
                width: 6,
                height: 4,
                min_width: 4,
                min_height: 3,
            },
            refresh_interval_secs: 5,
            alert_config: Some(WidgetAlertConfig {
                enabled: true,
                warning_threshold: Some(80.0),
                critical_threshold: Some(95.0),
                comparison_operator: "greater_than".to_string(),
            }),
        };

        let stats_widget = Widget {
            id: "stats".to_string(),
            widget_type: WidgetType::StatCard,
            title: "Quick Stats".to_string(),
            metrics: vec![
                WidgetMetric {
                    metric_type: MetricType::CpuUsage,
                    label: "CPU".to_string(),
                    color: Some("#4CAF50".to_string()),
                    aggregation: Some("avg".to_string()),
                },
                WidgetMetric {
                    metric_type: MetricType::MemoryPercent,
                    label: "Memory".to_string(),
                    color: Some("#2196F3".to_string()),
                    aggregation: Some("avg".to_string()),
                },
                WidgetMetric {
                    metric_type: MetricType::Tps,
                    label: "TPS".to_string(),
                    color: Some("#9C27B0".to_string()),
                    aggregation: Some("avg".to_string()),
                },
                WidgetMetric {
                    metric_type: MetricType::PlayerCount,
                    label: "Players".to_string(),
                    color: Some("#FF9800".to_string()),
                    aggregation: Some("max".to_string()),
                },
            ],
            visualization: VisualizationConfig {
                visualization_type: WidgetType::StatCard,
                show_legend: false,
                show_grid: false,
                show_axis_labels: false,
                color_scheme: None,
                thresholds: None,
                time_range_secs: 60,
                animations_enabled: false,
            },
            size: WidgetSize {
                width: 12,
                height: 2,
                min_width: 6,
                min_height: 2,
            },
            refresh_interval_secs: 5,
            alert_config: None,
        };

        let alerts_widget = Widget {
            id: "alerts".to_string(),
            widget_type: WidgetType::AlertList,
            title: "Active Alerts".to_string(),
            metrics: vec![],
            visualization: VisualizationConfig {
                visualization_type: WidgetType::AlertList,
                show_legend: false,
                show_grid: false,
                show_axis_labels: false,
                color_scheme: None,
                thresholds: None,
                time_range_secs: 3600,
                animations_enabled: true,
            },
            size: WidgetSize {
                width: 6,
                height: 4,
                min_width: 4,
                min_height: 3,
            },
            refresh_interval_secs: 10,
            alert_config: None,
        };

        let network_widget = Widget {
            id: "network".to_string(),
            widget_type: WidgetType::LineChart,
            title: "Network Traffic".to_string(),
            metrics: vec![
                WidgetMetric {
                    metric_type: MetricType::NetworkRx,
                    label: "Download".to_string(),
                    color: Some("#4CAF50".to_string()),
                    aggregation: Some("sum".to_string()),
                },
                WidgetMetric {
                    metric_type: MetricType::NetworkTx,
                    label: "Upload".to_string(),
                    color: Some("#F44336".to_string()),
                    aggregation: Some("sum".to_string()),
                },
            ],
            visualization: VisualizationConfig {
                visualization_type: WidgetType::LineChart,
                show_legend: true,
                show_grid: true,
                show_axis_labels: true,
                color_scheme: Some("default".to_string()),
                thresholds: None,
                time_range_secs: 300,
                animations_enabled: true,
            },
            size: WidgetSize {
                width: 6,
                height: 4,
                min_width: 4,
                min_height: 3,
            },
            refresh_interval_secs: 5,
            alert_config: None,
        };

        let mut positions = HashMap::new();
        positions.insert("stats".to_string(), WidgetPosition { x: 0, y: 0, width: 12, height: 2 });
        positions.insert("cpu_chart".to_string(), WidgetPosition { x: 0, y: 2, width: 6, height: 4 });
        positions.insert("memory_chart".to_string(), WidgetPosition { x: 6, y: 2, width: 6, height: 4 });
        positions.insert("network".to_string(), WidgetPosition { x: 0, y: 6, width: 6, height: 4 });
        positions.insert("alerts".to_string(), WidgetPosition { x: 6, y: 6, width: 6, height: 4 });

        let dashboard = Dashboard {
            id: "overview".to_string(),
            name: "Overview".to_string(),
            description: "Server overview dashboard with key metrics".to_string(),
            widgets: vec![stats_widget, cpu_widget, memory_widget, network_widget, alerts_widget],
            layout: DashboardLayout {
                columns: 12,
                rows: 11,
                widget_positions: positions,
            },
            refresh_interval_secs: 5,
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "system".to_string(),
        };

        DashboardTemplate {
            id: "overview".to_string(),
            name: "Overview Dashboard".to_string(),
            description: "Standard server overview with CPU, memory, network, and alerts".to_string(),
            category: DashboardCategory::Overview,
            dashboard,
        }
    }

    pub fn init_default_templates(&self) {
        self.register_template(Self::create_overview_template());

        let perf_widget = Widget {
            id: "perf".to_string(),
            widget_type: WidgetType::LineChart,
            title: "Performance Metrics".to_string(),
            metrics: vec![
                WidgetMetric {
                    metric_type: MetricType::Tps,
                    label: "TPS".to_string(),
                    color: Some("#4CAF50".to_string()),
                    aggregation: None,
                },
                WidgetMetric {
                    metric_type: MetricType::CpuUsage,
                    label: "CPU".to_string(),
                    color: Some("#2196F3".to_string()),
                    aggregation: None,
                },
            ],
            visualization: VisualizationConfig {
                visualization_type: WidgetType::LineChart,
                show_legend: true,
                show_grid: true,
                show_axis_labels: true,
                color_scheme: None,
                thresholds: None,
                time_range_secs: 600,
                animations_enabled: true,
            },
            size: WidgetSize {
                width: 12,
                height: 6,
                min_width: 6,
                min_height: 4,
            },
            refresh_interval_secs: 5,
            alert_config: None,
        };

        let perf_dashboard = Dashboard {
            id: "performance".to_string(),
            name: "Performance".to_string(),
            description: "Detailed performance metrics".to_string(),
            widgets: vec![perf_widget],
            layout: DashboardLayout {
                columns: 12,
                rows: 6,
                widget_positions: HashMap::new(),
            },
            refresh_interval_secs: 5,
            is_default: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "system".to_string(),
        };

        self.register_template(DashboardTemplate {
            id: "performance".to_string(),
            name: "Performance Dashboard".to_string(),
            description: "Focus on server performance metrics".to_string(),
            category: DashboardCategory::Performance,
            dashboard: perf_dashboard,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_manager_creation() {
        let manager = DashboardManager::with_default();
        assert_eq!(manager.max_snapshots, 100);
    }

    #[test]
    fn test_create_dashboard() {
        let manager = DashboardManager::with_default();
        let dashboard = manager.create_dashboard("Test", "Test dashboard", "admin");

        assert_eq!(dashboard.name, "Test");
        assert_eq!(dashboard.created_by, "admin");
        assert!(!dashboard.is_default);
    }

    #[test]
    fn test_add_widget() {
        let manager = DashboardManager::with_default();
        let dashboard = manager.create_dashboard("Test", "Test", "admin");

        let widget = Widget {
            id: "new".to_string(),
            widget_type: WidgetType::StatCard,
            title: "Test Widget".to_string(),
            metrics: vec![],
            visualization: VisualizationConfig {
                visualization_type: WidgetType::StatCard,
                show_legend: false,
                show_grid: false,
                show_axis_labels: false,
                color_scheme: None,
                thresholds: None,
                time_range_secs: 60,
                animations_enabled: false,
            },
            size: WidgetSize {
                width: 4,
                height: 2,
                min_width: 2,
                min_height: 2,
            },
            refresh_interval_secs: 5,
            alert_config: None,
        };

        let result = manager.add_widget(&dashboard.id, widget);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().widgets.len(), 1);
    }

    #[test]
    fn test_set_default_dashboard() {
        let manager = DashboardManager::with_default();
        let d1 = manager.create_dashboard("D1", "First", "admin");
        let d2 = manager.create_dashboard("D2", "Second", "admin");

        manager.set_default_dashboard(&d1.id).unwrap();
        let default = manager.get_default_dashboard();
        assert_eq!(default.unwrap().name, "D1");

        manager.set_default_dashboard(&d2.id).unwrap();
        let default = manager.get_default_dashboard();
        assert_eq!(default.unwrap().name, "D2");
    }

    #[test]
    fn test_create_from_template() {
        let manager = DashboardManager::with_default();
        manager.init_default_templates();

        let dashboard = manager.create_from_template("overview", "My Overview", "admin");
        assert!(dashboard.is_ok());

        let created = dashboard.unwrap();
        assert_eq!(created.name, "My Overview");
        assert!(!created.widgets.is_empty());
    }

    #[test]
    fn test_overview_template_creation() {
        let template = DashboardManager::create_overview_template();
        assert_eq!(template.id, "overview");
        assert!(!template.dashboard.widgets.is_empty());
    }
}

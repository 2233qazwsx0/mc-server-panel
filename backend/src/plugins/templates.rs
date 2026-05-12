use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::plugins::types::*;

pub struct TemplateManager {
    templates_dir: std::path::PathBuf,
    templates: HashMap<String, PluginTemplate>,
}

impl TemplateManager {
    pub fn new(templates_dir: std::path::PathBuf) -> Self {
        Self {
            templates_dir,
            templates: HashMap::new(),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.templates_dir).await?;
        self.load_templates().await?;
        Ok(())
    }

    async fn load_templates(&mut self) -> Result<()> {
        let mut entries = tokio::fs::read_dir(&self.templates_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(template) = serde_json::from_str::<PluginTemplate>(&content) {
                        self.templates.insert(template.id.clone(), template);
                    }
                }
            }
        }

        if self.templates.is_empty() {
            self.create_default_templates().await?;
        }

        Ok(())
    }

    async fn create_default_templates(&mut self) -> Result<()> {
        let defaults = vec![
            self.create_essentials_template()?,
            self.create_permissions_template()?,
            self.create_worldedit_template()?,
        ];

        for template in defaults {
            self.save_template(&template).await?;
            self.templates.insert(template.id.clone(), template);
        }

        Ok(())
    }

    fn create_essentials_template(&self) -> Result<PluginTemplate> {
        Ok(PluginTemplate {
            id: Uuid::new_v4().to_string(),
            name: "EssentialsX Configuration".to_string(),
            plugin_name: "EssentialsX".to_string(),
            version: "2.x".to_string(),
            template_name: "default".to_string(),
            config_template: json!({
                " Essentials": {
                    "enabled": true,
                    "locale": "en",
                    "debug": false
                },
                "socialspy": {
                    "enabled": false,
                    "socialSpyCosts": 100.0
                },
                "nickname-length": 16,
                "max-player-name-length": 16,
                "change-player-name": false,
                "allow-duplicate-names": false,
                "zero-session-size": "kick",
                "use-color-codes": false,
                "force-color-codes": false,
                "allow-italic": false
            }),
            variables: vec![
                TemplateVariable {
                    name: "locale".to_string(),
                    var_type: "string".to_string(),
                    default_value: "en".to_string(),
                    description: "Language locale".to_string(),
                    required: false,
                    options: Some(vec![
                        "en".to_string(),
                        "zh_CN".to_string(),
                        "zh_TW".to_string(),
                    ]),
                },
                TemplateVariable {
                    name: "debug".to_string(),
                    var_type: "boolean".to_string(),
                    default_value: "false".to_string(),
                    description: "Enable debug mode".to_string(),
                    required: false,
                    options: None,
                },
            ],
            description: "Standard EssentialsX configuration with common settings".to_string(),
            created_at: Utc::now(),
            usage_count: 0,
        })
    }

    fn create_permissions_template(&self) -> Result<PluginTemplate> {
        Ok(PluginTemplate {
            id: Uuid::new_v4().to_string(),
            name: "LuckPerms Basic Setup".to_string(),
            plugin_name: "LuckPerms".to_string(),
            version: "5.x".to_string(),
            template_name: "basic-setup".to_string(),
            config_template: json!({
                "storage": {
                    "type": "sqlite",
                    "dataSourceProperties": {}
                },
                "sync": {
                    "enabled": true,
                    "delay": 60
                },
                "cache": {
                    "expiry": 300
                },
                "workarounds": {
                    "enabled": true
                },
                "broadcast": {
                    "permission-not-found": false
                },
                "priorities": {
                    "inheritance": true,
                    "weight": true
                }
            }),
            variables: vec![
                TemplateVariable {
                    name: "storage_type".to_string(),
                    var_type: "select".to_string(),
                    default_value: "sqlite".to_string(),
                    description: "Database type for storing permissions".to_string(),
                    required: true,
                    options: Some(vec![
                        "sqlite".to_string(),
                        "mysql".to_string(),
                        "mariadb".to_string(),
                        "postgresql".to_string(),
                        "h2".to_string(),
                    ]),
                },
            ],
            description: "Basic LuckPerms setup with SQLite storage".to_string(),
            created_at: Utc::now(),
            usage_count: 0,
        })
    }

    fn create_worldedit_template(&self) -> Result<PluginTemplate> {
        Ok(PluginTemplate {
            id: Uuid::new_v4().to_string(),
            name: "WorldEdit Performance".to_string(),
            plugin_name: "WorldEdit".to_string(),
            version: "7.x".to_string(),
            template_name: "performance".to_string(),
            config_template: json!({
                "region-selection": {
                    "max-block-count": -1,
                    "max-poly-size": 30000,
                    "max-brush-radius": 20,
                    "allow-cuboid-mode": true,
                    "allow-glob-selection": true
                },
                "history": {
                    "size": 25,
                    "disk-size": 100
                },
                "clipboard": {
                    "use-disk": false,
                    "compression-level": 7
                },
                "logging": {
                    "log-commands": false,
                    "log-durations": true,
                    "record-history": true
                }
            }),
            variables: vec![],
            description: "Optimized WorldEdit configuration for better performance".to_string(),
            created_at: Utc::now(),
            usage_count: 0,
        })
    }

    pub async fn create_template(
        &mut self,
        name: String,
        plugin_name: String,
        version: String,
        config: Value,
        variables: Vec<TemplateVariable>,
        description: String,
    ) -> Result<PluginTemplate> {
        let template = PluginTemplate {
            id: Uuid::new_v4().to_string(),
            name,
            plugin_name,
            version,
            template_name: "custom".to_string(),
            config_template: config,
            variables,
            description,
            created_at: Utc::now(),
            usage_count: 0,
        };

        self.save_template(&template).await?;
        self.templates.insert(template.id.clone(), template.clone());

        Ok(template)
    }

    pub async fn get_templates(&self, plugin_name: Option<&str>) -> Vec<PluginTemplate> {
        match plugin_name {
            Some(name) => self
                .templates
                .values()
                .filter(|t| t.plugin_name.to_lowercase() == name.to_lowercase())
                .cloned()
                .collect(),
            None => self.templates.values().cloned().collect(),
        }
    }

    pub async fn get_template(&self, template_id: &str) -> Option<PluginTemplate> {
        self.templates.get(template_id).cloned()
    }

    pub async fn apply_template(
        &self,
        template_id: &str,
        variables: HashMap<String, String>,
    ) -> Result<Value> {
        let template = self
            .templates
            .get(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template not found"))?;

        let mut config = template.config_template.clone();

        for var in &template.variables {
            if let Some(value) = variables.get(&var.name) {
                config = self.set_nested_value(&var.name, value, config);
            }
        }

        Ok(config)
    }

    fn set_nested_value(&self, path: &str, value: &str, mut config: Value) -> Value {
        let parts: Vec<&str> = path.split('.').collect();
        
        if parts.len() == 1 {
            if let Value::Object(ref mut map) = config {
                let v = match value {
                    "true" | "false" => Value::Bool(value.parse().unwrap_or(false)),
                    v if v.parse::<f64>().is_ok() => Value::Number(v.parse().unwrap()),
                    _ => Value::String(value.to_string()),
                };
                map.insert(parts[0].to_string(), v);
            }
        } else {
            if let Value::Object(ref mut map) = config {
                if let Some(nested) = map.get_mut(parts[0]) {
                    let remaining_path = parts[1..].join(".");
                    *nested = self.set_nested_value(&remaining_path, value, nested.clone());
                }
            }
        }

        config
    }

    pub async fn delete_template(&mut self, template_id: &str) -> Result<()> {
        if let Some(template) = self.templates.remove(template_id) {
            let path = self.templates_dir.join(format!("{}.json", template_id));
            if path.exists() {
                tokio::fs::remove_file(path).await?;
            }
        }
        Ok(())
    }

    async fn save_template(&self, template: &PluginTemplate) -> Result<()> {
        let path = self.templates_dir.join(format!("{}.json", template.id));
        let content = serde_json::to_string_pretty(template)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub async fn increment_usage(&mut self, template_id: &str) -> Result<()> {
        if let Some(template) = self.templates.get_mut(template_id) {
            template.usage_count += 1;
            let path = self.templates_dir.join(format!("{}.json", template_id));
            let content = serde_json::to_string_pretty(template)?;
            tokio::fs::write(path, content).await?;
        }
        Ok(())
    }

    pub fn get_popular_templates(&self, limit: usize) -> Vec<PluginTemplate> {
        let mut templates: Vec<_> = self.templates.values().cloned().collect();
        templates.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        templates.into_iter().take(limit).collect()
    }
}

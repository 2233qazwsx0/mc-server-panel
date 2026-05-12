use std::collections::HashMap;
use anyhow::Result;

use crate::plugins::types::*;

pub struct CompatibilityAnalyzer {
    server_version: String,
    api_version: String,
    known_issues: HashMap<String, Vec<String>>,
    compatibility_cache: HashMap<String, f64>,
}

impl CompatibilityAnalyzer {
    pub fn new(server_version: String, api_version: String) -> Self {
        let mut known_issues = HashMap::new();
        known_issues.insert(
            "worldedit".to_string(),
            vec![
                "Version 7.x requires Paper 1.14+".to_string(),
                "Some expressions may not work on Spigot".to_string(),
            ],
        );
        known_issues.insert(
            "worldguard".to_string(),
            vec![
                "Requires WorldEdit to be installed".to_string(),
                "Region flags may conflict with core plugins".to_string(),
            ],
        );

        Self {
            server_version,
            api_version,
            known_issues,
            compatibility_cache: HashMap::new(),
        }
    }

    pub fn analyze_compatibility(&mut self, plugin: &Plugin) -> PluginCompatibility {
        if let Some(&cached) = self.compatibility_cache.get(&plugin.plugin_id.to_string()) {
            return PluginCompatibility {
                compatibility_score: cached,
                server_version: self.server_version.clone(),
                api_version: self.api_version.clone(),
                known_issues: self
                    .known_issues
                    .get(&plugin.name.to_lowercase())
                    .cloned()
                    .unwrap_or_default(),
                verified: self.check_verified(plugin),
            };
        }

        let score = self.calculate_compatibility_score(plugin);
        self.compatibility_cache
            .insert(plugin.plugin_id.to_string(), score);

        PluginCompatibility {
            compatibility_score: score,
            server_version: self.server_version.clone(),
            api_version: self.api_version.clone(),
            known_issues: self
                .known_issues
                .get(&plugin.name.to_lowercase())
                .cloned()
                .unwrap_or_default(),
            verified: self.check_verified(plugin),
        }
    }

    fn calculate_compatibility_score(&self, plugin: &Plugin) -> f64 {
        let mut score = 100.0;

        if !plugin
            .supported_versions
            .contains(&self.server_version)
        {
            score -= 40.0;
        }

        for supported in &plugin.supported_versions {
            let supported_major = self.extract_major_minor(supported);
            let server_major = self.extract_major_minor(&self.server_version);

            if supported_major != server_major {
                score -= 20.0;
            }
        }

        score -= self.check_version_gaps(&plugin.supported_versions);

        score -= self.check_old_api_version(&plugin.api_version);

        if plugin.description.as_ref().map(|d| d.is_empty()).unwrap_or(true) {
            score -= 5.0;
        }

        if plugin.stats.average_rating < 3.0 {
            score -= 10.0;
        }

        if self.check_verified(plugin) {
            score += 5.0;
        }

        score.max(0.0).min(100.0)
    }

    fn extract_major_minor(&self, version: &str) -> String {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            format!("{}.{}", parts[0], parts[1])
        } else {
            version.to_string()
        }
    }

    fn check_version_gaps(&self, supported_versions: &[String]) -> f64 {
        if supported_versions.is_empty() {
            return 30.0;
        }

        let mut gaps = 0.0;
        for version in supported_versions {
            if version.contains("alpha") || version.contains("beta") {
                gaps += 10.0;
            }
        }

        gaps
    }

    fn check_old_api_version(&self, api_version: &str) -> f64 {
        let old_apis = vec!["1.8", "1.9", "1.10", "1.11", "1.12"];
        
        for old in &old_apis {
            if api_version.starts_with(old) {
                return 25.0;
            }
        }

        0.0
    }

    fn check_verified(&self, plugin: &Plugin) -> bool {
        let verified_authors = vec![
            "sk89q".to_string(),
            "wizjany".to_string(),
            "luck".to_string(),
            "mdc".to_string(),
        ];

        plugin
            .authors
            .iter()
            .any(|a| verified_authors.contains(a))
    }

    pub fn get_compatibility_rating(&self, score: f64) -> String {
        if score >= 90.0 {
            "Excellent".to_string()
        } else if score >= 75.0 {
            "Good".to_string()
        } else if score >= 50.0 {
            "Fair".to_string()
        } else if score >= 25.0 {
            "Poor".to_string()
        } else {
            "Incompatible".to_string()
        }
    }

    pub fn batch_analyze(&mut self, plugins: &[Plugin]) -> Vec<PluginCompatibility> {
        plugins
            .iter()
            .map(|p| self.analyze_compatibility(p))
            .collect()
    }

    pub fn find_compatible_alternatives(
        &self,
        plugin: &Plugin,
        all_plugins: &[Plugin],
    ) -> Vec<Plugin> {
        let target_score = self.analyze_compatibility(plugin).compatibility_score;

        all_plugins
            .iter()
            .filter(|p| {
                p.category == plugin.category
                    && p.name != plugin.name
                    && self
                        .analyze_compatibility(p)
                        .compatibility_score
                        > target_score
            })
            .cloned()
            .collect()
    }

    pub fn update_server_version(&mut self, version: String, api_version: String) {
        self.server_version = version;
        self.api_version = api_version;
        self.compatibility_cache.clear();
    }
}

use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::plugins::types::*;

pub struct DependencyResolver {
    installed_plugins: HashMap<String, InstalledPlugin>,
    known_dependencies: HashMap<String, Vec<PluginDependency>>,
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            installed_plugins: HashMap::new(),
            known_dependencies: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: InstalledPlugin) {
        self.installed_plugins.insert(plugin.id.clone(), plugin.clone());
        self.known_dependencies.insert(plugin.name.clone(), plugin.dependencies);
    }

    pub fn unregister_plugin(&mut self, plugin_id: &str) -> Option<InstalledPlugin> {
        if let Some(plugin) = self.installed_plugins.remove(plugin_id) {
            self.known_dependencies.remove(&plugin.name);
            Some(plugin)
        } else {
            None
        }
    }

    pub fn check_conflicts(&self, new_plugin: &Plugin) -> Vec<DependencyConflict> {
        let mut conflicts = Vec::new();

        for (plugin_id, installed) in &self.installed_plugins {
            if let Some(conflict) = self.check_version_conflict(&installed, new_plugin) {
                conflicts.push(conflict);
            }

            if let Some(dep_conflicts) = self.check_dependency_conflicts(&installed, new_plugin) {
                conflicts.extend(dep_conflicts);
            }

            if let Some(soft_conflict) = self.check_api_conflicts(&installed, new_plugin) {
                conflicts.push(soft_conflict);
            }

            if let Some(cmd_conflict) = self.check_command_conflicts(&installed, new_plugin) {
                conflicts.push(cmd_conflict);
            }
        }

        conflicts.extend(self.check_known_conflicts(new_plugin));

        conflicts
    }

    fn check_version_conflict(
        &self,
        installed: &InstalledPlugin,
        new_plugin: &Plugin,
    ) -> Option<DependencyConflict> {
        if installed.name.to_lowercase() == new_plugin.name.to_lowercase() {
            let installed_major = self.extract_major_version(&installed.version);
            let new_major = self.extract_major_version(&new_plugin.version);

            if installed_major != new_major {
                return Some(DependencyConflict {
                    plugin_a: installed.name.clone(),
                    plugin_b: new_plugin.name.clone(),
                    conflict_type: ConflictType::Version,
                    description: format!(
                        "Version mismatch: installed {} but trying to install {}",
                        installed.version, new_plugin.version
                    ),
                    severity: ConflictSeverity::Critical,
                    resolution: format!(
                        "Either keep version {} or uninstall current version first",
                        installed.version
                    ),
                });
            }
        }
        None
    }

    fn check_dependency_conflicts(
        &self,
        installed: &InstalledPlugin,
        new_plugin: &Plugin,
    ) -> Option<Vec<DependencyConflict>> {
        let mut conflicts = Vec::new();

        if let Some(deps) = self.known_dependencies.get(&installed.name) {
            for dep in deps {
                if dep.name.to_lowercase() == new_plugin.name.to_lowercase()
                    && !dep.version_match
                {
                    conflicts.push(DependencyConflict {
                        plugin_a: installed.name.clone(),
                        plugin_b: new_plugin.name.clone(),
                        conflict_type: ConflictType::HardDependency,
                        description: format!(
                            "Hard dependency conflict: {} requires {} version {} but found {}",
                            installed.name, new_plugin.name, dep.version, new_plugin.version
                        ),
                        severity: if dep.required {
                            ConflictSeverity::Critical
                        } else {
                            ConflictSeverity::Medium
                        },
                        resolution: format!(
                            "Install a compatible version of {} (required: {})",
                            new_plugin.name, dep.version
                        ),
                    });
                }
            }
        }

        if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        }
    }

    fn check_api_conflicts(
        &self,
        installed: &InstalledPlugin,
        new_plugin: &Plugin,
    ) -> Option<DependencyConflict> {
        let installed_apis: HashSet<_> = installed
            .supported_versions
            .iter()
            .cloned()
            .collect();

        let new_apis: HashSet<_> = new_plugin
            .supported_versions
            .iter()
            .cloned()
            .collect();

        let intersection: HashSet<_> = installed_apis.intersection(&new_apis).collect();

        if intersection.is_empty() {
            return Some(DependencyConflict {
                plugin_a: installed.name.clone(),
                plugin_b: new_plugin.name.clone(),
                conflict_type: ConflictType::ApiIncompatibility,
                description: format!(
                    "No common API versions: {} supports {:?} but {} supports {:?}",
                    installed.name, installed_apis, new_plugin.name, new_apis
                ),
                severity: ConflictSeverity::High,
                resolution: format!(
                    "One of these plugins may not work correctly. Consider alternatives."
                ),
            });
        }

        None
    }

    fn check_command_conflicts(
        &self,
        installed: &InstalledPlugin,
        new_plugin: &Plugin,
    ) -> Option<DependencyConflict> {
        let known_conflicts = self.get_known_command_conflicts();
        
        let installed_lower = installed.name.to_lowercase();
        let new_lower = new_plugin.name.to_lowercase();

        if let Some(commands) = known_conflicts.get(&installed_lower) {
            if commands.contains(&new_lower.as_str()) {
                return Some(DependencyConflict {
                    plugin_a: installed.name.clone(),
                    plugin_b: new_plugin.name.clone(),
                    conflict_type: ConflictType::CommandConflict,
                    description: format!(
                        "{} and {} are known to have command conflicts",
                        installed.name, new_plugin.name
                    ),
                    severity: ConflictSeverity::Medium,
                    resolution: "Use plugin prefixes for commands or disable one plugin".to_string(),
                });
            }
        }

        None
    }

    fn check_known_conflicts(&self, new_plugin: &Plugin) -> Vec<DependencyConflict> {
        let known_groups = vec![
            vec!["worldedit", "fawe", "fastasyncworldedit"],
            vec!["essentialsx", "essentials"],
            vec!["luckperms", "permissionsmanager"],
        ];

        let mut conflicts = Vec::new();
        let new_lower = new_plugin.name.to_lowercase();

        for group in &known_groups {
            if group.iter().any(|p| p == &new_lower.as_str()) {
                for installed in self.installed_plugins.values() {
                    let installed_lower = installed.name.to_lowercase();
                    if group.iter().any(|p| p == &installed_lower.as_str())
                        && installed_lower != new_lower
                    {
                        conflicts.push(DependencyConflict {
                            plugin_a: installed.name.clone(),
                            plugin_b: new_plugin.name.clone(),
                            conflict_type: ConflictType::ResourceConflict,
                            description: format!(
                                "These plugins provide similar functionality: {} vs {}",
                                installed.name, new_plugin.name
                            ),
                            severity: ConflictSeverity::Medium,
                            resolution: "Choose one plugin that best fits your needs".to_string(),
                        });
                    }
                }
            }
        }

        conflicts
    }

    fn get_known_command_conflicts(&self) -> HashMap<String, Vec<&'static str>> {
        let mut conflicts = HashMap::new();
        conflicts.insert("worldguard".to_string(), vec!["regionizer", "protectionlib"]);
        conflicts.insert("coreprotect".to_string(), vec!["logblock", "prism"]);
        conflicts.insert("vault".to_string(), vec!["permissionsex"]);
        conflicts
    }

    fn extract_major_version(&self, version: &str) -> &str {
        version.split('.').next().unwrap_or(version)
    }

    pub fn resolve_dependencies(&self, plugin: &Plugin) -> Vec<PluginDependency> {
        let mut dependencies = Vec::new();
        
        let known_deps = self.get_common_dependencies(&plugin.name.to_lowercase());
        
        for dep_name in known_deps {
            let installed = self.find_plugin_by_name(dep_name);
            
            dependencies.push(PluginDependency {
                name: dep_name.to_string(),
                version: "*".to_string(),
                required: true,
                installed: installed.is_some(),
                version_match: true,
            });
        }

        dependencies
    }

    fn get_common_dependencies(&self, plugin_name: &str) -> Vec<&'static str> {
        let dependency_map = HashMap::from([
            ("worldedit", vec!["craftbook"]),
            ("worldguard", vec!["worldedit"]),
            ("vault", vec!["permissionsex", "luckperms", "groupmanager"]),
            ("essentialsx", vec!["vault"]),
            ("luckperms", vec!["vault"]),
            ("placeholderapi", vec![]),
            ("skript", vec![]),
            (" Citizens", vec!["skript"]),
        ]);

        dependency_map
            .get(plugin_name)
            .cloned()
            .unwrap_or_default()
    }

    fn find_plugin_by_name(&self, name: &str) -> Option<&InstalledPlugin> {
        self.installed_plugins
            .values()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
    }

    pub fn get_all_conflicts(&self) -> Vec<DependencyConflict> {
        let mut all_conflicts = Vec::new();
        let plugins: Vec<_> = self.installed_plugins.values().collect();

        for i in 0..plugins.len() {
            for j in (i + 1)..plugins.len() {
                let plugin_a = plugins[i];
                let plugin_b = plugins[j];

                if let Some(conflict) = self.check_plugin_pair_conflict(plugin_a, plugin_b) {
                    all_conflicts.push(conflict);
                }
            }
        }

        all_conflicts
    }

    fn check_plugin_pair_conflict(
        &self,
        plugin_a: &InstalledPlugin,
        plugin_b: &InstalledPlugin,
    ) -> Option<DependencyConflict> {
        if let Some(dep_a) = self.known_dependencies.get(&plugin_a.name) {
            for dep in dep_a {
                if dep.name.to_lowercase() == plugin_b.name.to_lowercase() && !dep.version_match {
                    return Some(DependencyConflict {
                        plugin_a: plugin_a.name.clone(),
                        plugin_b: plugin_b.name.clone(),
                        conflict_type: ConflictType::HardDependency,
                        description: format!(
                            "Version mismatch between {} and {}",
                            plugin_a.name, plugin_b.name
                        ),
                        severity: ConflictSeverity::High,
                        resolution: "Update one of the plugins to a compatible version".to_string(),
                    });
                }
            }
        }

        None
    }

    pub fn get_compatibility_matrix(&self) -> Vec<Vec<String>> {
        let plugins: Vec<_> = self.installed_plugins
            .values()
            .map(|p| p.name.clone())
            .collect();

        let mut matrix = Vec::new();
        for plugin_a in &plugins {
            let mut row = Vec::new();
            for plugin_b in &plugins {
                if plugin_a == plugin_b {
                    row.push("SELF".to_string());
                } else if self.has_conflict(plugin_a, plugin_b) {
                    row.push("CONFLICT".to_string());
                } else {
                    row.push("OK".to_string());
                }
            }
            matrix.push(row);
        }

        matrix
    }

    fn has_conflict(&self, plugin_a: &str, plugin_b: &str) -> bool {
        for conflict in self.get_all_conflicts() {
            if (conflict.plugin_a == plugin_a && conflict.plugin_b == plugin_b)
                || (conflict.plugin_a == plugin_b && conflict.plugin_b == plugin_a)
            {
                return true;
            }
        }
        false
    }
}

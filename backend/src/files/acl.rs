use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use uuid::Uuid;

pub struct AclService {
    server_root: PathBuf,
    acl_file: PathBuf,
    rules: RwLock<HashMap<String, AclRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclRule {
    pub id: String,
    pub path: String,
    pub principal: AclPrincipal,
    pub permissions: Vec<String>,
    pub recursive: bool,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclPrincipal {
    pub kind: PrincipalKind,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrincipalKind {
    User,
    Group,
    Role,
}

impl AclService {
    pub fn new(server_root: PathBuf) -> Self {
        let acl_file = server_root.join(".acl.json");
        let service = Self {
            server_root,
            acl_file,
            rules: RwLock::new(HashMap::new()),
        };
        service.load_rules().ok();
        service
    }

    fn load_rules(&self) -> Result<()> {
        if self.acl_file.exists() {
            let content = fs::read_to_string(&self.acl_file)?;
            let rules: HashMap<String, AclRule> = serde_json::from_str(&content)?;
            let mut guard = self.rules.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            *guard = rules;
        }
        Ok(())
    }

    fn save_rules(&self) -> Result<()> {
        let guard = self.rules.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let content = serde_json::to_string_pretty(&*guard)?;
        fs::write(&self.acl_file, content)?;
        Ok(())
    }

    pub fn add_rule(&self, rule: AclRuleRequest) -> Result<AclRule> {
        let full_path = self.resolve_path(&rule.path)?;
        
        if !full_path.exists() {
            anyhow::bail!("Path does not exist: {}", rule.path);
        }

        let now = chrono::Utc::now().to_rfc3339();
        let acl_rule = AclRule {
            id: Uuid::new_v4().to_string(),
            path: rule.path,
            principal: rule.principal,
            permissions: rule.permissions,
            recursive: rule.recursive,
            created_at: now.clone(),
            modified_at: now,
        };

        {
            let mut guard = self.rules.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.insert(acl_rule.id.clone(), acl_rule.clone());
        }

        self.save_rules()?;
        Ok(acl_rule)
    }

    pub fn remove_rule(&self, rule_id: &str) -> Result<()> {
        {
            let mut guard = self.rules.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            if guard.remove(rule_id).is_none() {
                anyhow::bail!("Rule not found: {}", rule_id);
            }
        }
        self.save_rules()?;
        Ok(())
    }

    pub fn update_rule(&self, rule_id: &str, update: AclRuleUpdate) -> Result<AclRule> {
        let mut guard = self.rules.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let rule = guard.get_mut(rule_id)
            .ok_or_else(|| anyhow::anyhow!("Rule not found: {}", rule_id))?;

        if let Some(path) = update.path {
            let full_path = self.resolve_path(&path)?;
            if !full_path.exists() {
                anyhow::bail!("Path does not exist: {}", path);
            }
            rule.path = path;
        }

        if let Some(principal) = update.principal {
            rule.principal = principal;
        }

        if let Some(permissions) = update.permissions {
            rule.permissions = permissions;
        }

        if let Some(recursive) = update.recursive {
            rule.recursive = recursive;
        }

        rule.modified_at = chrono::Utc::now().to_rfc3339();
        let updated = rule.clone();
        drop(guard);
        self.save_rules()?;
        Ok(updated)
    }

    pub fn list_rules(&self, path_filter: Option<&str>) -> Result<Vec<AclRule>> {
        let guard = self.rules.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let rules: Vec<AclRule> = guard.values()
            .filter(|rule| {
                if let Some(filter) = path_filter {
                    rule.path.starts_with(filter) || rule.path == filter
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        Ok(rules)
    }

    pub fn get_rule(&self, rule_id: &str) -> Result<AclRule> {
        let guard = self.rules.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        guard.get(rule_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Rule not found: {}", rule_id))
    }

    pub fn check_permission(&self, path: &str, principal_id: &str, permission: &str) -> Result<bool> {
        let full_path = self.resolve_path(path)?;
        let guard = self.rules.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;

        let mut current_path: Option<PathBuf> = Some(full_path.clone());
        
        while let Some(p) = current_path {
            let path_str = p.to_string_lossy()
                .strip_prefix(&self.server_root.to_string_lossy())
                .unwrap_or(p.to_string_lossy().as_ref())
                .trim_start_matches('/')
                .to_string();

            for rule in guard.values().filter(|r| r.principal.id == principal_id) {
                let rule_path = rule.path.trim_start_matches('/');
                
                if rule.path == "/" || path_str == rule_path || path_str.starts_with(&format!("{}/", rule_path)) {
                    if rule.recursive || p == full_path {
                        if rule.permissions.contains(&permission.to_string()) {
                            return Ok(true);
                        }
                        if rule.permissions.contains(&"admin".to_string()) {
                            return Ok(true);
                        }
                    }
                }
            }

            current_path = p.parent().map(|p| p.to_path_buf());
        }

        Ok(false)
    }

    pub fn get_effective_permissions(&self, path: &str, principal_id: &str) -> Result<Vec<String>> {
        let full_path = self.resolve_path(path)?;
        let guard = self.rules.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        let mut permissions = Vec::new();
        let mut has_admin = false;

        let mut current_path: Option<PathBuf> = Some(full_path.clone());
        
        while let Some(p) = current_path {
            let path_str = p.to_string_lossy()
                .strip_prefix(&self.server_root.to_string_lossy())
                .unwrap_or(p.to_string_lossy().as_ref())
                .trim_start_matches('/')
                .to_string();

            for rule in guard.values().filter(|r| r.principal.id == principal_id) {
                let rule_path = rule.path.trim_start_matches('/');
                
                if rule.path == "/" || path_str == rule_path || path_str.starts_with(&format!("{}/", rule_path)) {
                    if rule.recursive || p == full_path {
                        for perm in &rule.permissions {
                            if !permissions.contains(perm) {
                                permissions.push(perm.clone());
                            }
                        }
                        if rule.permissions.contains(&"admin".to_string()) {
                            has_admin = true;
                        }
                    }
                }
            }

            current_path = p.parent().map(|p| p.to_path_buf());
        }

        if has_admin {
            permissions = vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
                "execute".to_string(),
                "admin".to_string(),
            ];
        }

        Ok(permissions)
    }

    pub fn inherit_permissions(&self, from_path: &str, to_path: &str, recursive: bool) -> Result<Vec<AclRule>> {
        let source_rules = self.list_rules(Some(from_path))?;
        let mut new_rules = Vec::new();

        for rule in source_rules {
            let new_path = rule.path.replace(from_path, to_path);
            
            let new_rule = self.add_rule(AclRuleRequest {
                path: new_path,
                principal: rule.principal,
                permissions: rule.permissions,
                recursive,
            })?;
            new_rules.push(new_rule);
        }

        Ok(new_rules)
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let requested_path = self.server_root.join(path);
        let canonical = requested_path.canonicalize()?;
        
        if !canonical.starts_with(&self.server_root) {
            anyhow::bail!("Path escape detected: {}", path);
        }

        Ok(canonical)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclRuleRequest {
    pub path: String,
    pub principal: AclPrincipal,
    pub permissions: Vec<String>,
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclRuleUpdate {
    pub path: Option<String>,
    pub principal: Option<AclPrincipal>,
    pub permissions: Option<Vec<String>>,
    pub recursive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclPermissionCheck {
    pub path: String,
    pub principal_id: String,
    pub permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclPermissionResult {
    pub allowed: bool,
    pub effective_permissions: Vec<String>,
    pub matched_rules: Vec<String>,
}

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct ValidatorService {
    server_root: PathBuf,
    schemas: HashMap<String, ValidationSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSchema {
    pub name: String,
    pub file_type: String,
    pub rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required(String),
    MaxLength(String, usize),
    MinLength(String, usize),
    Pattern(String, String),
    Range(String, f64, f64),
    Custom(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub file_path: String,
    pub file_type: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub code: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub code: String,
}

impl ValidatorService {
    pub fn new(server_root: PathBuf) -> Self {
        let mut service = Self {
            server_root,
            schemas: HashMap::new(),
        };
        service.load_default_schemas();
        service
    }

    fn load_default_schemas(&mut self) {
        self.schemas.insert(
            "spigot.yml".to_string(),
            ValidationSchema {
                name: "Spigot Configuration".to_string(),
                file_type: "yaml".to_string(),
                rules: vec![
                    ValidationRule::Pattern(
                        "spigot.settings".to_string(),
                        r"^\d+$".to_string(),
                    ),
                    ValidationRule::Range(
                        "spigot.settings.max-velocity".to_string(),
                        0.0,
                        100.0,
                    ),
                ],
            },
        );

        self.schemas.insert(
            "server.properties".to_string(),
            ValidationSchema {
                name: "Server Properties".to_string(),
                file_type: "properties".to_string(),
                rules: vec![
                    ValidationRule::Pattern(
                        "server-port".to_string(),
                        r"^\d{1,5}$".to_string(),
                    ),
                    ValidationRule::Pattern(
                        "max-players".to_string(),
                        r"^\d+$".to_string(),
                    ),
                ],
            },
        );

        self.schemas.insert(
            "bukkit.yml".to_string(),
            ValidationSchema {
                name: "Bukkit Configuration".to_string(),
                file_type: "yaml".to_string(),
                rules: vec![],
            },
        );

        self.schemas.insert(
            "paper.yml".to_string(),
            ValidationSchema {
                name: "Paper Configuration".to_string(),
                file_type: "yaml".to_string(),
                rules: vec![],
            },
        );

        self.schemas.insert(
            "ops.json".to_string(),
            ValidationSchema {
                name: "Operators List".to_string(),
                file_type: "json".to_string(),
                rules: vec![],
            },
        );

        self.schemas.insert(
            "whitelist.json".to_string(),
            ValidationSchema {
                name: "Whitelist".to_string(),
                file_type: "json".to_string(),
                rules: vec![],
            },
        );

        self.schemas.insert(
            "banned-players.json".to_string(),
            ValidationSchema {
                name: "Banned Players".to_string(),
                file_type: "json".to_string(),
                rules: vec![],
            },
        );

        self.schemas.insert(
            "banned-ips.json".to_string(),
            ValidationSchema {
                name: "Banned IPs".to_string(),
                file_type: "json".to_string(),
                rules: vec![],
            },
        );
    }

    pub fn validate_file(&self, path: &str) -> Result<ValidationResult> {
        let full_path = self.server_root.join(path);

        if !full_path.exists() {
            anyhow::bail!("File not found: {}", path);
        }

        let extension = full_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_name = full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_type = match extension.as_str() {
            "yml" | "yaml" => "yaml",
            "json" => "json",
            "properties" => "properties",
            "toml" => "toml",
            "xml" => "xml",
            _ => "unknown",
        };

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        if let Some(schema) = self.schemas.get(&file_name) {
            match schema.file_type.as_str() {
                "yaml" => {
                    self.validate_yaml_content(&full_path, &mut errors, &mut warnings, &mut suggestions)?;
                }
                "json" => {
                    self.validate_json_content(&full_path, &mut errors, &mut warnings, &mut suggestions)?;
                }
                "properties" => {
                    self.validate_properties_content(&full_path, &mut errors, &mut warnings, &mut suggestions)?;
                }
                _ => {}
            }
        } else {
            match file_type {
                "yaml" => {
                    self.validate_yaml_content(&full_path, &mut errors, &mut warnings, &mut suggestions)?;
                }
                "json" => {
                    self.validate_json_content(&full_path, &mut errors, &mut warnings, &mut suggestions)?;
                }
                "properties" => {
                    self.validate_properties_content(&full_path, &mut errors, &mut warnings, &mut suggestions)?;
                }
                _ => {}
            }
        }

        let valid = errors.is_empty();

        Ok(ValidationResult {
            valid,
            file_path: path.to_string(),
            file_type: file_type.to_string(),
            errors,
            warnings,
            suggestions,
        })
    }

    fn validate_yaml_content(&self, path: &PathBuf, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>, suggestions: &mut Vec<String>) -> Result<()> {
        let content = fs::read_to_string(path)?;

        match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(_) => {}
            Err(e) => {
                let line = e.location()
                    .map(|loc| loc.line())
                    .unwrap_or(1);

                errors.push(ValidationError {
                    line: Some(line),
                    column: e.location().map(|loc| loc.column()),
                    message: format!("YAML parsing error: {}", e),
                    code: "YAML001".to_string(),
                    severity: "error".to_string(),
                });
            }
        }

        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with('\t') {
                warnings.push(ValidationWarning {
                    line: Some(idx + 1),
                    column: None,
                    message: "YAML should use spaces for indentation, not tabs".to_string(),
                    code: "YAML101".to_string(),
                });
            }

            if line.ends_with(' ') && !trimmed.is_empty() {
                warnings.push(ValidationWarning {
                    line: Some(idx + 1),
                    column: None,
                    message: "Trailing whitespace detected".to_string(),
                    code: "YAML102".to_string(),
                });
            }

            if line.contains(":  ") && !line.contains("http") {
                warnings.push(ValidationWarning {
                    line: Some(idx + 1),
                    column: None,
                    message: "Possible unnecessary space after colon".to_string(),
                    code: "YAML103".to_string(),
                });
            }
        }

        Ok(())
    }

    fn validate_json_content(&self, path: &PathBuf, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>, suggestions: &mut Vec<String>) -> Result<()> {
        let content = fs::read_to_string(path)?;

        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(value) => {
                self.validate_json_structure(&value, path, errors, warnings, suggestions)?;
            }
            Err(e) => {
                let line = e.line();
                let column = e.column();

                errors.push(ValidationError {
                    line: Some(line),
                    column: Some(column),
                    message: format!("JSON parsing error: {}", e),
                    code: "JSON001".to_string(),
                    severity: "error".to_string(),
                });
            }
        }

        Ok(())
    }

    fn validate_json_structure(&self, value: &serde_json::Value, path: &PathBuf, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>, suggestions: &mut Vec<String>) -> Result<()> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        match file_name {
            "ops.json" | "whitelist.json" => {
                if let Some(arr) = value.as_array() {
                    for (idx, item) in arr.iter().enumerate() {
                        if !item.is_object() {
                            errors.push(ValidationError {
                                line: None,
                                column: None,
                                message: format!("Invalid entry at index {}: expected object", idx),
                                code: "JSON201".to_string(),
                                severity: "error".to_string(),
                            });
                        } else if file_name == "ops.json" {
                            if !item.get("uuid").is_some() && !item.get("name").is_some() {
                                errors.push(ValidationError {
                                    line: None,
                                    column: None,
                                    message: format!("Missing 'uuid' or 'name' at index {}", idx),
                                    code: "JSON202".to_string(),
                                    severity: "error".to_string(),
                                });
                            }
                        }
                    }
                } else {
                    errors.push(ValidationError {
                        line: None,
                        column: None,
                        message: "Root must be an array".to_string(),
                        code: "JSON203".to_string(),
                        severity: "error".to_string(),
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn validate_properties_content(&self, path: &PathBuf, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>, suggestions: &mut Vec<String>) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if !trimmed.contains('=') {
                warnings.push(ValidationWarning {
                    line: Some(idx + 1),
                    column: None,
                    message: "Property line should contain '=' separator".to_string(),
                    code: "PROP101".to_string(),
                });
            }

            if trimmed.starts_with(' ') {
                warnings.push(ValidationWarning {
                    line: Some(idx + 1),
                    column: None,
                    message: "Property keys should not start with whitespace".to_string(),
                    code: "PROP102".to_string(),
                });
            }

            if let Some(eq_idx) = trimmed.find('=') {
                let key = &trimmed[..eq_idx];
                let value = &trimmed[eq_idx + 1..];

                if key.is_empty() {
                    errors.push(ValidationError {
                        line: Some(idx + 1),
                        column: None,
                        message: "Empty property key".to_string(),
                        code: "PROP001".to_string(),
                        severity: "error".to_string(),
                    });
                }

                if key.contains(' ') && !key.starts_with('#') {
                    warnings.push(ValidationWarning {
                        line: Some(idx + 1),
                        column: None,
                        message: "Property keys should not contain spaces".to_string(),
                        code: "PROP103".to_string(),
                    });
                }

                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if file_name == "server.properties" {
                    self.validate_server_property(key, value, idx + 1, errors, warnings)?;
                }
            }
        }

        Ok(())
    }

    fn validate_server_property(&self, key: &str, value: &str, line: usize, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
        match key {
            "server-port" | "rcon.port" => {
                if let Ok(port) = value.parse::<u16>() {
                    if port == 0 {
                        errors.push(ValidationError {
                            line: Some(line),
                            column: None,
                            message: "Port cannot be 0".to_string(),
                            code: "PROP201".to_string(),
                            severity: "error".to_string(),
                        });
                    }
                } else {
                    errors.push(ValidationError {
                        line: Some(line),
                        column: None,
                        message: format!("Invalid port number: {}", value),
                        code: "PROP202".to_string(),
                        severity: "error".to_string(),
                    });
                }
            }
            "max-players" => {
                if let Ok(players) = value.parse::<u32>() {
                    if players > 1000 {
                        warnings.push(ValidationWarning {
                            line: Some(line),
                            column: None,
                            message: "Max players exceeds recommended limit (1000)".to_string(),
                            code: "PROP301".to_string(),
                        });
                    }
                } else {
                    errors.push(ValidationError {
                        line: Some(line),
                        column: None,
                        message: format!("Invalid player count: {}", value),
                        code: "PROP203".to_string(),
                        severity: "error".to_string(),
                    });
                }
            }
            "view-distance" => {
                if let Ok(distance) = value.parse::<u32>() {
                    if distance > 32 {
                        warnings.push(ValidationWarning {
                            line: Some(line),
                            column: None,
                            message: "View distance above 32 may cause performance issues".to_string(),
                            code: "PROP302".to_string(),
                        });
                    }
                }
            }
            "spawn-protection" => {
                if let Ok(protection) = value.parse::<u32>() {
                    if protection > 16 {
                        warnings.push(ValidationWarning {
                            line: Some(line),
                            column: None,
                            message: "Spawn protection above 16 may cause issues".to_string(),
                            code: "PROP303".to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    pub fn validate_json_with_schema(&self, path: &str, schema: &str) -> Result<ValidationResult> {
        let full_path = self.server_root.join(path);

        if !full_path.exists() {
            anyhow::bail!("File not found: {}", path);
        }

        let content = fs::read_to_string(&full_path)?;
        let json_value: serde_json::Value = serde_json::from_str(&content)?;

        let schema_value: serde_json::Value = serde_json::from_str(schema)?;

        let mut errors = Vec::new();
        self.validate_against_schema(&json_value, &schema_value, "", &mut errors);

        Ok(ValidationResult {
            valid: errors.is_empty(),
            file_path: path.to_string(),
            file_type: "json".to_string(),
            errors,
            warnings: vec![],
            suggestions: vec![],
        })
    }

    fn validate_against_schema(&self, value: &serde_json::Value, schema: &serde_json::Value, path: &str, errors: &mut Vec<ValidationError>) {
        if let Some(required) = schema.get("required") {
            if let Some(req_array) = required.as_array() {
                if let Some(obj) = value.as_object() {
                    for req in req_array {
                        if let Some(key) = req.as_str() {
                            if !obj.contains_key(key) {
                                errors.push(ValidationError {
                                    line: None,
                                    column: None,
                                    message: format!("Missing required property: {}", key),
                                    code: "SCHEMA001".to_string(),
                                    severity: "error".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        if let Some(type_value) = schema.get("type") {
            if let Some(expected_type) = type_value.as_str() {
                match expected_type {
                    "string" if !value.is_string() => {
                        errors.push(ValidationError {
                            line: None,
                            column: None,
                            message: format!("Expected string at {}", path),
                            code: "SCHEMA002".to_string(),
                            severity: "error".to_string(),
                        });
                    }
                    "number" if !value.is_number() => {
                        errors.push(ValidationError {
                            line: None,
                            column: None,
                            message: format!("Expected number at {}", path),
                            code: "SCHEMA003".to_string(),
                            severity: "error".to_string(),
                        });
                    }
                    "boolean" if !value.is_boolean() => {
                        errors.push(ValidationError {
                            line: None,
                            column: None,
                            message: format!("Expected boolean at {}", path),
                            code: "SCHEMA004".to_string(),
                            severity: "error".to_string(),
                        });
                    }
                    "array" if !value.is_array() => {
                        errors.push(ValidationError {
                            line: None,
                            column: None,
                            message: format!("Expected array at {}", path),
                            code: "SCHEMA005".to_string(),
                            severity: "error".to_string(),
                        });
                    }
                    "object" if !value.is_object() => {
                        errors.push(ValidationError {
                            line: None,
                            column: None,
                            message: format!("Expected object at {}", path),
                            code: "SCHEMA006".to_string(),
                            severity: "error".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }

        if let Some(obj) = value.as_object() {
            if let Some(properties) = schema.get("properties") {
                if let Some(props_obj) = properties.as_object() {
                    for (key, prop_schema) in props_obj {
                        if let Some(prop_value) = obj.get(key) {
                            self.validate_against_schema(prop_value, prop_schema, &format!("{}.{}", path, key), errors);
                        }
                    }
                }
            }
        }
    }

    pub fn get_supported_types(&self) -> Vec<SupportedFileType> {
        vec![
            SupportedFileType {
                extension: "yml".to_string(),
                extension2: "yaml".to_string(),
                mime_types: vec!["application/x-yaml".to_string(), "text/yaml".to_string()],
                schema_available: true,
                description: "YAML configuration files".to_string(),
            },
            SupportedFileType {
                extension: "json".to_string(),
                extension2: "".to_string(),
                mime_types: vec!["application/json".to_string()],
                schema_available: true,
                description: "JSON data files".to_string(),
            },
            SupportedFileType {
                extension: "properties".to_string(),
                extension2: "".to_string(),
                mime_types: vec!["text/plain".to_string()],
                schema_available: true,
                description: "Java properties files".to_string(),
            },
            SupportedFileType {
                extension: "toml".to_string(),
                extension2: "".to_string(),
                mime_types: vec!["application/toml".to_string()],
                schema_available: false,
                description: "TOML configuration files".to_string(),
            },
            SupportedFileType {
                extension: "xml".to_string(),
                extension2: "".to_string(),
                mime_types: vec!["application/xml".to_string(), "text/xml".to_string()],
                schema_available: false,
                description: "XML configuration files".to_string(),
            },
        ]
    }

    pub fn add_schema(&mut self, name: String, schema: ValidationSchema) {
        self.schemas.insert(name, schema);
    }

    pub fn get_schema(&self, name: &str) -> Option<&ValidationSchema> {
        self.schemas.get(name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedFileType {
    pub extension: String,
    pub extension2: String,
    pub mime_types: Vec<String>,
    pub schema_available: bool,
    pub description: String,
}

use axum::{
    extract::State,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;
use crate::developer::DeveloperState;
use crate::error::AppError;

#[derive(Serialize, ToSchema)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: HashMap<String, PathItem>,
    pub components: Components,
}

#[derive(Serialize, ToSchema)]
pub struct OpenApiInfo {
    pub title: String,
    pub description: String,
    pub version: String,
    pub contact: OpenApiContact,
}

#[derive(Serialize, ToSchema)]
pub struct OpenApiContact {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, ToSchema)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
}

#[derive(Serialize, ToSchema)]
pub struct Operation {
    pub summary: String,
    pub description: String,
    pub operation_id: String,
    pub responses: HashMap<String, ResponseObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
}

#[derive(Serialize, ToSchema)]
pub struct ResponseObject {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Serialize, ToSchema)]
pub struct MediaType {
    pub schema_: SchemaRef,
}

#[derive(Serialize, ToSchema)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    pub ref_: String,
}

#[derive(Serialize, ToSchema)]
pub struct RequestBody {
    pub description: String,
    pub required: bool,
    pub content: HashMap<String, MediaType>,
}

#[derive(Serialize, ToSchema)]
pub struct Parameter {
    pub name: String,
    pub in_: String,
    pub description: String,
    pub required: bool,
    pub schema_: SchemaRef,
}

#[derive(Serialize, ToSchema)]
pub struct Components {
    pub schemas: HashMap<String, Schema>,
    pub security_schemes: HashMap<String, SecurityScheme>,
}

#[derive(Serialize, ToSchema)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: Option<HashMap<String, Schema>>,
    pub required: Option<Vec<String>>,
}

#[derive(Serialize, ToSchema)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub description: Option<String>,
    pub name: Option<String>,
    pub location: Option<String>,
}

use std::collections::HashMap;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Minecraft Server Admin Panel API",
        description = "High-performance Minecraft server management panel with developer tools",
        version = "1.0.0",
        contact(
            name = "MC Admin Team",
            url = "https://github.com/mc-admin/panel"
        )
    ),
    paths(
        get_openapi_spec,
        get_paths,
        validate_schema
    ),
    components(
        schemas(
            crate::developer::OpenApiSpec,
            crate::developer::ServerStatus,
            crate::developer::CommandRequest,
            crate::developer::MetricsResponse,
            crate::developer::ApiResponse,
            crate::developer::RequestLogEntry,
            crate::developer::RateLimitInfo,
            crate::developer::EventSubscription,
            crate::developer::PluginHook,
            crate::developer::ProfilerSnapshot
        )
    ),
    tags(
        (name = "Server", description = "Server management endpoints"),
        (name = "Metrics", description = "Server metrics and monitoring"),
        (name = "RCON", description = "RCON remote console"),
        (name = "Developer", description = "Developer tools and APIs")
    )
)]
pub struct ApiDoc;

#[derive(Debug, Serialize, ToSchema)]
pub struct ServerStatus {
    pub online: bool,
    pub cpu: f64,
    pub memory: f64,
    pub tps: f64,
    pub players: u32,
    pub max_players: u32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CommandRequest {
    pub command: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MetricsResponse {
    pub cpu: f64,
    pub memory: f64,
    pub uptime_seconds: u64,
    pub timestamp: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
}

#[utoipa::path(
    get,
    path = "/api/docs/openapi.json",
    responses(
        (status = 200, description = "OpenAPI specification", body = OpenApiSpec)
    ),
    tag = "Developer"
)]
pub async fn get_openapi_spec() -> Response {
    let spec = ApiDoc::openapi();
    Json(spec).into_response()
}

#[utoipa::path(
    get,
    path = "/api/docs/paths",
    responses(
        (status = 200, description = "List of all API paths")
    ),
    tag = "Developer"
)]
pub async fn get_paths() -> Result<Json<Vec<String>>, AppError> {
    let paths = vec![
        "/health".to_string(),
        "/api/status".to_string(),
        "/api/start".to_string(),
        "/api/stop".to_string(),
        "/api/restart".to_string(),
        "/api/command".to_string(),
        "/api/logs".to_string(),
        "/api/metrics".to_string(),
        "/api/metrics/history".to_string(),
        "/api/rcon/connect".to_string(),
        "/api/rcon/disconnect".to_string(),
        "/api/rcon/stats".to_string(),
        "/api/rcon/players".to_string(),
        "/api/developer/docs".to_string(),
        "/api/developer/paths".to_string(),
        "/api/developer/schema".to_string(),
        "/api/developer/request-logs".to_string(),
        "/api/developer/rate-limit".to_string(),
        "/api/developer/events/subscribe".to_string(),
        "/api/developer/events/unsubscribe".to_string(),
        "/api/developer/plugins/hooks".to_string(),
        "/api/developer/plugins/reload".to_string(),
        "/api/developer/sdk/generate".to_string(),
        "/api/developer/profiler/snapshots".to_string(),
        "/api/developer/profiler/start".to_string(),
        "/api/developer/profiler/stop".to_string(),
        "/api/developer/webhook/validate".to_string(),
        "/api/developer/ws-debug/sessions".to_string(),
        "/api/developer/ws-debug/send".to_string(),
        "/api/developer/console/execute".to_string(),
        "/ws".to_string(),
    ];
    Ok(Json(paths))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SchemaValidationRequest {
    pub schema: serde_json::Value,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SchemaValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/developer/schema",
    request_body = SchemaValidationRequest,
    responses(
        (status = 200, description = "Schema validation result", body = SchemaValidationResponse)
    ),
    tag = "Developer"
)]
pub async fn validate_schema(
    Json(req): Json<SchemaValidationRequest>,
) -> Result<Json<SchemaValidationResponse>, AppError> {
    let mut errors = Vec::new();
    
    if let Some(obj) = req.schema.as_object() {
        if let Some(properties) = obj.get("properties") {
            if let Some(props) = properties.as_object() {
                for (key, schema) in props {
                    if let Some(required) = obj.get("required").and_then(|r| r.as_array()) {
                        let is_required = required.iter().any(|r| r == key);
                        if is_required {
                            if let Some(data_obj) = req.data.as_object() {
                                if !data_obj.contains_key(key) {
                                    errors.push(format!("Missing required field: {}", key));
                                }
                            }
                        }
                    }
                    
                    if let Some(schema_type) = schema.get("type") {
                        if let Some(data_val) = req.data.get(key) {
                            let expected_type = schema_type.as_str().unwrap_or("");
                            let actual_type = match data_val {
                                serde_json::Value::Null => "null",
                                serde_json::Value::Bool(_) => "boolean",
                                serde_json::Value::Number(_) => "number",
                                serde_json::Value::String(_) => "string",
                                serde_json::Value::Array(_) => "array",
                                serde_json::Value::Object(_) => "object",
                            };
                            if expected_type == "integer" && !matches!(data_val, serde_json::Value::Number(n) if n.is_i64()) {
                                errors.push(format!("Field '{}' expected integer, got {}", key, actual_type));
                            } else if expected_type == "number" && !matches!(data_val, serde_json::Value::Number(_)) {
                                errors.push(format!("Field '{}' expected number, got {}", key, actual_type));
                            } else if expected_type == "string" && !matches!(data_val, serde_json::Value::String(_)) {
                                errors.push(format!("Field '{}' expected string, got {}", key, actual_type));
                            } else if expected_type == "boolean" && !matches!(data_val, serde_json::Value::Bool(_)) {
                                errors.push(format!("Field '{}' expected boolean, got {}", key, actual_type));
                            } else if expected_type == "array" && !matches!(data_val, serde_json::Value::Array(_)) {
                                errors.push(format!("Field '{}' expected array, got {}", key, actual_type));
                            } else if expected_type == "object" && !matches!(data_val, serde_json::Value::Object(_)) {
                                errors.push(format!("Field '{}' expected object, got {}", key, actual_type));
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(Json(SchemaValidationResponse {
        valid: errors.is_empty(),
        errors,
    }))
}

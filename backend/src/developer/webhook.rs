use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sha1::{Sha1, Digest};
use hmac::{Hmac, Mac};
use chrono::Utc;

use crate::state::AppState;

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookValidateRequest {
    pub payload: String,
    pub signature: String,
    pub secret: String,
    pub timestamp: Option<String>,
    pub algorithm: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookValidateResponse {
    pub valid: bool,
    pub signature: String,
    pub computed_signature: String,
    pub algorithm: String,
    pub timestamp_valid: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookGenerateRequest {
    pub payload: String,
    pub secret: String,
    pub algorithm: Option<String>,
    pub timestamp: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookGenerateResponse {
    pub signature: String,
    pub timestamp: Option<i64>,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookTestRequest {
    pub url: String,
    pub payload: serde_json::Value,
    pub secret: String,
    pub method: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookTestResponse {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

pub fn compute_signature_sha1(secret: &str, payload: &str, timestamp: Option<&str>) -> String {
    let mut mac = HmacSha1::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    
    if let Some(ts) = timestamp {
        mac.update(ts.as_bytes());
        mac.update(b".");
    }
    mac.update(payload.as_bytes());
    
    let result = mac.finalize();
    hex::encode(result)
}

pub fn compute_signature_sha256(secret: &str, payload: &str, timestamp: Option<&str>) -> String {
    use sha2::{Sha256, Digest as Sha2Digest};
    
    let mut hasher = Sha256::new();
    
    if let Some(ts) = timestamp {
        hasher.update(ts.as_bytes());
        hasher.update(b".");
    }
    hasher.update(payload.as_bytes());
    
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn verify_signature(
    secret: &str,
    payload: &str,
    signature: &str,
    algorithm: &str,
    timestamp: Option<&str>,
) -> bool {
    let computed = match algorithm.to_lowercase().as_str() {
        "sha1" | "sha-1" => compute_signature_sha1(secret, payload, timestamp),
        "sha256" | "sha-256" => compute_signature_sha256(secret, payload, timestamp),
        _ => compute_signature_sha1(secret, payload, timestamp),
    };
    
    computed == signature
}

#[utoipa::path(
    post,
    path = "/api/developer/webhook/validate",
    request_body = WebhookValidateRequest,
    responses(
        (status = 200, description = "Validate webhook signature", body = WebhookValidateResponse)
    ),
    tag = "Developer"
)]
pub async fn validate_webhook(
    State(state): State<AppState>,
    Json(req): Json<WebhookValidateRequest>,
) -> Result<Json<WebhookValidateResponse>, crate::error::AppError> {
    let algorithm = req.algorithm.unwrap_or_else(|| "SHA1".to_string());
    let timestamp_valid = if let Some(ts) = &req.timestamp {
        if let Ok(ts_int) = ts.parse::<i64>() {
            let now = Utc::now().timestamp();
            (now - ts_int).abs() < 300
        } else {
            false
        }
    } else {
        true
    };
    
    let computed = match algorithm.to_lowercase().as_str() {
        "sha1" | "sha-1" => compute_signature_sha1(&req.secret, &req.payload, req.timestamp.as_deref()),
        "sha256" | "sha-256" => compute_signature_sha256(&req.secret, &req.payload, req.timestamp.as_deref()),
        _ => compute_signature_sha1(&req.secret, &req.payload, req.timestamp.as_deref()),
    };
    
    let valid = computed == req.signature && timestamp_valid;
    
    Ok(Json(WebhookValidateResponse {
        valid,
        signature: req.signature,
        computed_signature: computed,
        algorithm: algorithm.clone(),
        timestamp_valid,
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/webhook/generate",
    request_body = WebhookGenerateRequest,
    responses(
        (status = 200, description = "Generate webhook signature", body = WebhookGenerateResponse)
    ),
    tag = "Developer"
)]
pub async fn generate_webhook_signature(
    State(state): State<AppState>,
    Json(req): Json<WebhookGenerateRequest>,
) -> Result<Json<WebhookGenerateResponse>, crate::error::AppError> {
    let algorithm = req.algorithm.unwrap_or_else(|| "SHA1".to_string());
    let include_timestamp = req.timestamp.unwrap_or(true);
    
    let (timestamp, signature) = if include_timestamp {
        let ts = Utc::now().timestamp();
        let sig = match algorithm.to_lowercase().as_str() {
            "sha1" | "sha-1" => compute_signature_sha1(&req.secret, &req.payload, Some(&ts.to_string())),
            "sha256" | "sha-256" => compute_signature_sha256(&req.secret, &req.payload, Some(&ts.to_string())),
            _ => compute_signature_sha1(&req.secret, &req.payload, Some(&ts.to_string())),
        };
        (Some(ts), sig)
    } else {
        let sig = match algorithm.to_lowercase().as_str() {
            "sha1" | "sha-1" => compute_signature_sha1(&req.secret, &req.payload, None),
            "sha256" | "sha-256" => compute_signature_sha256(&req.secret, &req.payload, None),
            _ => compute_signature_sha1(&req.secret, &req.payload, None),
        };
        (None, sig)
    };
    
    let mut headers = std::collections::HashMap::new();
    headers.insert("X-Signature".to_string(), signature.clone());
    headers.insert("X-Algorithm".to_string(), algorithm.clone());
    if let Some(ts) = timestamp {
        headers.insert("X-Timestamp".to_string(), ts.to_string());
    }
    
    Ok(Json(WebhookGenerateResponse {
        signature,
        timestamp,
        headers,
    }))
}

#[utoipa::path(
    post,
    path = "/api/developer/webhook/test",
    request_body = WebhookTestRequest,
    responses(
        (status = 200, description = "Test webhook endpoint", body = WebhookTestResponse)
    ),
    tag = "Developer"
)]
pub async fn test_webhook(
    State(state): State<AppState>,
    Json(req): Json<WebhookTestRequest>,
) -> Result<Json<WebhookTestResponse>, crate::error::AppError> {
    let start = std::time::Instant::now();
    let method = req.method.unwrap_or_else(|| "POST".to_string());
    
    let payload_str = serde_json::to_string(&req.payload)
        .unwrap_or_else(|_| req.payload.to_string());
    
    let signature = compute_signature_sha1(&req.secret, &payload_str, None);
    
    let client = reqwest::Client::new();
    
    let result = match method.to_uppercase().as_str() {
        "POST" => {
            client.post(&req.url)
                .header("Content-Type", "application/json")
                .header("X-Signature", &signature)
                .header("X-Timestamp", Utc::now().timestamp().to_string())
                .body(payload_str)
                .send()
                .await
        },
        "PUT" => {
            client.put(&req.url)
                .header("Content-Type", "application/json")
                .header("X-Signature", &signature)
                .body(payload_str)
                .send()
                .await
        },
        _ => {
            return Ok(Json(WebhookTestResponse {
                success: false,
                status_code: None,
                response_body: None,
                error: Some(format!("Unsupported method: {}", method)),
                duration_ms: start.elapsed().as_millis() as u64,
            }));
        }
    };
    
    match result {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().await.ok();
            let success = resp.status().is_success();
            
            Ok(Json(WebhookTestResponse {
                success,
                status_code: Some(status),
                response_body: body,
                error: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }))
        }
        Err(e) => {
            Ok(Json(WebhookTestResponse {
                success: false,
                status_code: None,
                response_body: None,
                error: Some(e.to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/developer/webhook/algorithms",
    responses(
        (status = 200, description = "List supported signature algorithms")
    ),
    tag = "Developer"
)]
pub async fn list_algorithms() -> Result<impl IntoResponse, crate::error::AppError> {
    let algorithms = vec![
        serde_json::json!({
            "id": "sha1",
            "name": "SHA-1",
            "secure": false,
            "description": "Legacy algorithm, not recommended for production"
        }),
        serde_json::json!({
            "id": "sha256",
            "name": "SHA-256",
            "secure": true,
            "description": "Recommended algorithm for webhook signatures"
        }),
    ];
    
    Ok(Json(algorithms))
}

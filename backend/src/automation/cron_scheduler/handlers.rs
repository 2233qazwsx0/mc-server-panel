use axum::Json;

pub async fn list_jobs() -> Json<serde_json::Value> {
    Json(serde_json::json!({"jobs": []}))
}

pub async fn create_job() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn update_job() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn delete_job() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn trigger_job() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

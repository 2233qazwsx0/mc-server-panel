use axum::Json;

pub async fn create_backup() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn list_backups() -> Json<serde_json::Value> {
    Json(serde_json::json!({"backups": []}))
}

pub async fn get_backup() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn restore_backup() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn delete_backup() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

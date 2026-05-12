use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Server is already running")]
    ServerAlreadyRunning,

    #[error("Server is not running")]
    ServerNotRunning,

    #[error("Process error: {0}")]
    ProcessError(#[from] std::io::Error),

    #[error("RCON error: {0}")]
    RconError(String),

    #[error("RCON not connected")]
    RconNotConnected,

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Command not allowed: {0}")]
    CommandNotAllowed(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::ServerAlreadyRunning => (StatusCode::CONFLICT, self.to_string()),
            AppError::ServerNotRunning => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::RconNotConnected => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::ConnectionRefused(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::Timeout(_) => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            AppError::InvalidCommand(_) | AppError::CommandNotAllowed(_) => {
                (StatusCode::FORBIDDEN, self.to_string())
            }
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) | AppError::InsufficientFunds(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Io(_) | AppError::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

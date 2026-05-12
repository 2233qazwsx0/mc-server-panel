use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;
use crate::api::{handlers, ws};

pub fn create_app(state: AppState) -> Router {
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/api/status", get(handlers::get_server_status))
        .route("/api/start", post(handlers::start_server))
        .route("/api/stop", post(handlers::stop_server))
        .route("/api/restart", post(handlers::restart_server))
        .route("/api/command", post(handlers::send_command))
        .route("/api/logs", get(handlers::get_logs))
        .route("/api/metrics", get(handlers::get_metrics))
        .route("/api/metrics/history", get(handlers::get_metrics_history))
        .route("/api/rcon/connect", post(handlers::connect_rcon))
        .route("/api/rcon/disconnect", post(handlers::disconnect_rcon))
        .route("/api/rcon/stats", get(handlers::get_rcon_stats))
        .route("/api/rcon/players", get(handlers::get_player_list))
        .route("/ws", get(ws::ws_handler))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

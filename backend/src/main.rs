pub mod config;
pub mod error;
pub mod core;
pub mod monitor;
pub mod players;
pub mod state;
pub mod api;
pub mod automation;

use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::Config;
use crate::state::AppState;
use crate::players::handlers::create_players_state;
use crate::players::routes::create_players_router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = Config::load_or_default("config.toml")?;

    let app_state = AppState::new(config.clone());
    let players_state = create_players_state();

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let players_app = create_players_router(app_state, players_state)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.api.host, config.api.port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Minecraft Admin Panel - Players Module (M4) starting on http://{}", addr);
    info!("Players API available at http://{}/api/players/*", addr);

    axum::serve(listener, players_app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down gracefully...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down gracefully...");
        }
    }
}

use axum::{extract::State, Json};
use serde::Serialize;
use std::result::Result as StdResult;

use crate::error::AppError;
use crate::state::AppState;

pub type Result<T> = StdResult<T, AppError>;

#[axum::debug_handler]
pub async fn connect_rcon(
    State(state): State<AppState>,
) -> Result<Json<()>> {
    if !state.process_manager.is_running().await {
        return Err(AppError::ServerNotRunning);
    }

    state.rcon_client.connect().await
        .map_err(|e| AppError::RconError(e.to_string()))?;

    Ok(Json(()))
}

pub async fn disconnect_rcon(
    State(state): State<AppState>,
) -> Result<Json<()>> {
    state.rcon_client.disconnect().await;
    Ok(Json(()))
}

#[derive(Debug, Serialize)]
pub struct RconStatsResponse {
    pub connected: bool,
    pub tps: Option<f64>,
    pub mspt: Option<f64>,
    pub online_players: usize,
    pub max_players: u32,
}

pub async fn get_rcon_stats(
    State(state): State<AppState>,
) -> Result<Json<RconStatsResponse>> {
    let connected = state.rcon_client.is_connected().await;

    if !connected {
        return Ok(Json(RconStatsResponse {
            connected: false,
            tps: None,
            mspt: None,
            online_players: 0,
            max_players: 20,
        }));
    }

    let stats = state.rcon_client.get_cached_stats().await;

    Ok(Json(RconStatsResponse {
        connected,
        tps: stats.tps,
        mspt: stats.mspt,
        online_players: stats.online_players.len(),
        max_players: stats.max_players,
    }))
}

#[derive(Debug, Serialize)]
pub struct PlayerListResponse {
    pub players: Vec<crate::core::rcon_client::PlayerInfo>,
    pub total: usize,
}

pub async fn get_player_list(
    State(state): State<AppState>,
) -> Result<Json<PlayerListResponse>> {
    if !state.rcon_client.is_connected().await {
        return Err(AppError::RconNotConnected);
    }

    let players = state.rcon_client.get_player_list().await
        .map_err(|e| AppError::RconError(e.to_string()))?;

    let total = players.len();

    Ok(Json(PlayerListResponse { players, total }))
}

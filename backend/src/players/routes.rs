use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::state::AppState;
use crate::players::handlers::{self, PlayersState};

pub fn create_players_router(
    state: AppState,
    players_state: PlayersState,
) -> Router {
    Router::new()
        .route("/api/players/map", get(handlers::get_player_map))
        .route("/api/players/:player_name/inventory", get(handlers::get_player_inventory))
        .route("/api/players/:player_name/inventory", put(handlers::update_player_inventory))
        .route("/api/players/ops", get(handlers::get_op_list))
        .route("/api/players/ops/:player_name", post(handlers::grant_op))
        .route("/api/players/ops/:player_name", delete(handlers::revoke_op))
        .route("/api/players/bans", get(handlers::get_ban_list))
        .route("/api/players/bans", post(handlers::ban_player))
        .route("/api/players/bans/:player_name", delete(handlers::unban_player))
        .route("/api/players/bans/sync/:server_id", post(handlers::sync_bans_to_server))
        .route("/api/players/actions", get(handlers::get_player_actions))
        .route("/api/players/:player_name/stats", get(handlers::get_player_stats))
        .route("/api/players/:player_name/backup/:backup_type", post(handlers::create_player_backup))
        .route("/api/players/backups", get(handlers::get_player_backups))
        .route("/api/players/:player_name/backup/:backup_id", post(handlers::restore_player_backup))
        .route("/api/players/permissions/groups", get(handlers::get_permission_groups))
        .route("/api/players/permissions/groups", post(handlers::create_permission_group))
        .route("/api/players/permissions/groups/:group_id", put(handlers::update_permission_group))
        .route("/api/players/permissions/groups/:group_id", delete(handlers::delete_permission_group))
        .route("/api/players/permissions/groups/:group_id/permissions", post(handlers::add_permission_to_group))
        .route("/api/players/permissions/groups/:group_id/permissions/:permission", delete(handlers::remove_permission_from_group))
        .route("/api/players/chat", get(handlers::get_chat_history))
        .route("/api/players/chat/search", get(handlers::search_chat))
        .route("/api/players/warnings", get(handlers::get_warnings))
        .route("/api/players/warnings", post(handlers::issue_warning))
        .route("/api/players/warnings/:warning_id", delete(handlers::revoke_warning))
        .route("/api/players/economy", get(handlers::get_player_economy))
        .route("/api/players/economy/leaderboard", get(handlers::get_economy_leaderboard))
        .route("/api/players/economy/give", post(handlers::give_money))
        .route("/api/players/economy/take", post(handlers::take_money))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
        .with_state(players_state)
}

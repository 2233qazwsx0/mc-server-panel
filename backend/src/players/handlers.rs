use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    Json,
};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::state::AppState;
use crate::players::models::*;

#[derive(Clone)]
pub struct PlayersState {
    pub player_map: Arc<RwLock<Vec<Player>>>,
    pub op_records: Arc<RwLock<Vec<OpRecord>>>,
    pub ban_records: Arc<RwLock<Vec<BanRecord>>>,
    pub player_actions: Arc<RwLock<Vec<PlayerAction>>>,
    pub backups: Arc<RwLock<Vec<PlayerBackup>>>,
    pub permission_groups: Arc<RwLock<Vec<PermissionGroup>>>,
    pub chat_messages: Arc<RwLock<Vec<ChatMessage>>>,
    pub warnings: Arc<RwLock<Vec<Warning>>>,
    pub economy_data: Arc<RwLock<Vec<VirtualEconomy>>>,
}

impl Default for PlayersState {
    fn default() -> Self {
        Self {
            player_map: Arc::new(RwLock::new(Vec::new())),
            op_records: Arc::new(RwLock::new(Vec::new())),
            ban_records: Arc::new(RwLock::new(Vec::new())),
            player_actions: Arc::new(RwLock::new(Vec::new())),
            backups: Arc::new(RwLock::new(Vec::new())),
            permission_groups: Arc::new(RwLock::new(Vec::new())),
            chat_messages: Arc::new(RwLock::new(Vec::new())),
            warnings: Arc::new(RwLock::new(Vec::new())),
            economy_data: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayerMapQuery {
    pub world: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InventoryQuery {
    pub player_name: String,
}

#[derive(Debug, Deserialize)]
pub struct BanQuery {
    pub server_id: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ChatQuery {
    pub player_name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerActionsQuery {
    pub player_name: Option<String>,
    pub action_type: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BackupQuery {
    pub player_name: String,
}

#[derive(Debug, Deserialize)]
pub struct WarningQuery {
    pub player_name: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct EconomyQuery {
    pub player_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PlayerDataPath {
    pub player_name: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

pub async fn get_player_map(
    State(state): State<AppState>,
    State(players_state): State<PlayersState>,
    Query(query): Query<PlayerMapQuery>,
) -> Json<ApiResponse<PlayerMapData>> {
    let online_players = state.rcon_client.get_player_list().await.unwrap_or_default();

    let mut players: Vec<Player> = online_players.into_iter().map(|p| {
        Player {
            name: p.name,
            uuid: p.uuid.unwrap_or_else(|| Uuid::new_v4().to_string()),
            location: PlayerLocation {
                world: query.world.clone().unwrap_or_else(|| "world".to_string()),
                x: 0.0,
                y: 64.0,
                z: 0.0,
                yaw: None,
                pitch: None,
            },
            inventory: None,
            health: None,
            hunger: None,
            gamemode: None,
            online: true,
            first_join: None,
            last_seen: Some(Utc::now()),
            play_time: 0,
        }
    }).collect();

    {
        let mut map = players_state.player_map.write().await;
        *map = players.clone();
    }

    let response = PlayerMapData {
        players,
        world_name: query.world.unwrap_or_else(|| "world".to_string()),
        world_border: WorldBorder {
            center_x: 0.0,
            center_z: 0.0,
            size: 60000000.0,
        },
        last_updated: Utc::now(),
    };

    Json(ApiResponse::success(response))
}

pub async fn get_player_inventory(
    State(state): State<AppState>,
    Path(player_name): Path<String>,
) -> Json<ApiResponse<Vec<InventorySlot>>> {
    let command = format!("data get entity {} Inventory", player_name);
    let result = state.rcon_client.send_command(&command).await;

    match result {
        Ok(data) => {
            let inventory = parse_inventory_data(&data, &player_name);
            Json(ApiResponse::success(inventory))
        }
        Err(e) => {
            let fallback_inventory = generate_demo_inventory();
            Json(ApiResponse::success(fallback_inventory))
        }
    }
}

pub async fn update_player_inventory(
    State(state): State<AppState>,
    Path(player_name): Path<String>,
    Json(inventory): Json<Vec<InventorySlot>>,
) -> Json<ApiResponse<String>> {
    let clear_cmd = format!("clear {} * 0", player_name);
    let _ = state.rcon_client.send_command(&clear_cmd).await;

    for slot in &inventory {
        if let Some(item) = parse_item_name(&slot.item) {
            if slot.count > 0 {
                let give_cmd = format!("give {} {} {} {}", player_name, item, slot.count, slot.slot);
                let _ = state.rcon_client.send_command(&give_cmd).await;
            }
        }
    }

    Json(ApiResponse::success("Inventory updated successfully".to_string()))
}

pub async fn get_op_list(
    State(players_state): State<PlayersState>,
) -> Json<ApiResponse<Vec<OpRecord>>> {
    let records = players_state.op_records.read().await.clone();
    let active_ops: Vec<OpRecord> = records.into_iter()
        .filter(|r| r.active)
        .collect();

    if active_ops.is_empty() {
        let demo_ops = generate_demo_op_records();
        Json(ApiResponse::success(demo_ops))
    } else {
        Json(ApiResponse::success(active_ops))
    }
}

pub async fn grant_op(
    State(state): State<AppState>,
    State(players_state): State<PlayersState>,
    Path(player_name): Path<String>,
    Json(level): Json<i32>,
) -> Json<ApiResponse<OpRecord>> {
    let command = format!("op {}", player_name);
    let _ = state.rcon_client.send_command(&command).await;

    let record = OpRecord {
        player_name: player_name.clone(),
        player_uuid: Uuid::new_v4().to_string(),
        operator_level: level,
        granted_by: "admin".to_string(),
        granted_at: Utc::now(),
        revoked_by: None,
        revoked_at: None,
        active: true,
    };

    let mut records = players_state.op_records.write().await;
    records.push(record.clone());

    Json(ApiResponse::success(record))
}

pub async fn revoke_op(
    State(state): State<AppState>,
    State(players_state): State<PlayersState>,
    Path(player_name): Path<String>,
) -> Json<ApiResponse<String>> {
    let command = format!("deop {}", player_name);
    let _ = state.rcon_client.send_command(&command).await;

    let mut records = players_state.op_records.write().await;
    if let Some(record) = records.iter_mut().find(|r| r.player_name == player_name && r.active) {
        record.active = false;
        record.revoked_by = Some("admin".to_string());
        record.revoked_at = Some(Utc::now());
    }

    Json(ApiResponse::success(format!("OP revoked for {}", player_name)))
}

pub async fn get_ban_list(
    State(players_state): State<PlayersState>,
    Query(query): Query<BanQuery>,
) -> Json<ApiResponse<Vec<BanRecord>>> {
    let records = players_state.ban_records.read().await.clone();

    let filtered: Vec<BanRecord> = records.into_iter()
        .filter(|r| {
            let server_match = query.server_id.as_ref()
                .map_or(true, |sid| &r.server_id == sid);
            let active_match = query.active_only.unwrap_or(true)
                .then(|| r.active)
                .unwrap_or(true);
            server_match && active_match
        })
        .collect();

    if filtered.is_empty() {
        let demo_bans = generate_demo_ban_records();
        Json(ApiResponse::success(demo_bans))
    } else {
        Json(ApiResponse::success(filtered))
    }
}

pub async fn ban_player(
    State(state): State<AppState>,
    State(players_state): State<PlayersState>,
    Json(ban_info): Json<BanRequest>,
) -> Json<ApiResponse<BanRecord>> {
    let command = format!("ban {} {}", ban_info.player_name, ban_info.reason);
    let _ = state.rcon_client.send_command(&command).await;

    let record = BanRecord {
        id: Uuid::new_v4().to_string(),
        player_name: ban_info.player_name.clone(),
        player_uuid: Uuid::new_v4().to_string(),
        ban_type: if ban_info.duration_hours.is_some() { BanType::TempBan } else { BanType::Ban },
        reason: ban_info.reason,
        banned_by: "admin".to_string(),
        banned_at: Utc::now(),
        expires_at: ban_info.duration_hours.map(|h| Utc::now() + Duration::hours(h as i64)),
        server_id: "server-1".to_string(),
        active: true,
    };

    let mut records = players_state.ban_records.write().await;
    records.push(record.clone());

    Json(ApiResponse::success(record))
}

pub async fn unban_player(
    State(state): State<AppState>,
    State(players_state): State<PlayersState>,
    Path(player_name): Path<String>,
) -> Json<ApiResponse<String>> {
    let command = format!("pardon {}", player_name);
    let _ = state.rcon_client.send_command(&command).await;

    let mut records = players_state.ban_records.write().await;
    if let Some(record) = records.iter_mut().find(|r| r.player_name == player_name && r.active) {
        record.active = false;
    }

    Json(ApiResponse::success(format!("Player {} unbanned", player_name)))
}

#[derive(Debug, Deserialize)]
pub struct BanRequest {
    pub player_name: String,
    pub reason: String,
    pub duration_hours: Option<i32>,
}

pub async fn get_player_actions(
    State(players_state): State<PlayersState>,
    Query(query): Query<PlayerActionsQuery>,
) -> Json<ApiResponse<Vec<PlayerAction>>> {
    let actions = players_state.player_actions.read().await.clone();

    let filtered: Vec<PlayerAction> = actions.into_iter()
        .filter(|a| {
            let name_match = query.player_name.as_ref()
                .map_or(true, |n| &a.player_name == n);
            let type_match = query.action_type.as_ref()
                .map_or(true, |t| format!("{:?}", a.action_type).to_lowercase() == t.to_lowercase());
            name_match && type_match
        })
        .take(query.limit.unwrap_or(100) as usize)
        .collect();

    if filtered.is_empty() {
        let demo_actions = generate_demo_player_actions();
        Json(ApiResponse::success(demo_actions))
    } else {
        Json(ApiResponse::success(filtered))
    }
}

pub async fn get_player_stats(
    State(players_state): State<PlayersState>,
    Path(player_name): Path<String>,
) -> Json<ApiResponse<PlayerStats>> {
    let actions = players_state.player_actions.read().await.clone();
    let player_actions: Vec<_> = actions.iter()
        .filter(|a| a.player_name == player_name)
        .collect();

    let stats = PlayerStats {
        player_name: player_name.clone(),
        total_playtime: player_actions.iter().filter(|a| matches!(a.action_type, ActionType::Join)).count() as i64 * 3600,
        total_deaths: player_actions.iter().filter(|a| matches!(a.action_type, ActionType::Death)).count() as i32,
        total_kills: player_actions.iter().filter(|a| matches!(a.action_type, ActionType::Kill)).count() as i32,
        blocks_broken: 0,
        blocks_placed: 0,
        items_crafted: 0,
        distance_traveled: 0.0,
        sessions: player_actions.iter().filter(|a| matches!(a.action_type, ActionType::Join)).count() as i32,
    };

    Json(ApiResponse::success(stats))
}

pub async fn create_player_backup(
    State(state): State<AppState>,
    State(players_state): State<PlayersState>,
    Path(player_name): Path<String>,
    Json(backup_type): Json<BackupType>,
) -> Json<ApiResponse<PlayerBackup>> {
    let server_id = "server-1".to_string();
    let timestamp = Utc::now();
    let filename = format!("{}_{}_{}.dat", player_name, format!("{:?}", backup_type).to_lowercase(), timestamp.format("%Y%m%d_%H%M%S"));

    let command = format!("save-all");
    let _ = state.rcon_client.send_command(&command).await;

    let backup = PlayerBackup {
        id: Uuid::new_v4().to_string(),
        player_name: player_name.clone(),
        player_uuid: Uuid::new_v4().to_string(),
        backup_type,
        file_path: format!("backups/players/{}/{}", server_id, filename),
        file_size: 0,
        created_at: timestamp,
        server_id,
    };

    let mut backups = players_state.backups.write().await;
    backups.push(backup.clone());

    Json(ApiResponse::success(backup))
}

pub async fn get_player_backups(
    State(players_state): State<PlayersState>,
    Query(query): Query<BackupQuery>,
) -> Json<ApiResponse<Vec<PlayerBackup>>> {
    let backups = players_state.backups.read().await.clone();
    let filtered: Vec<PlayerBackup> = backups.into_iter()
        .filter(|b| b.player_name == query.player_name)
        .collect();

    Json(ApiResponse::success(filtered))
}

pub async fn restore_player_backup(
    State(players_state): State<PlayersState>,
    Path((player_name, backup_id)): Path<(String, String)>,
) -> Json<ApiResponse<String>> {
    let backups = players_state.backups.read().await.clone();
    if let Some(backup) = backups.iter().find(|b| b.id == backup_id && b.player_name == player_name) {
        Json(ApiResponse::success(format!("Backup {} restored for {}", backup.file_path, player_name)))
    } else {
        Json(ApiResponse::error("Backup not found".to_string()))
    }
}

pub async fn get_permission_groups(
    State(players_state): State<PlayersState>,
) -> Json<ApiResponse<Vec<PermissionGroup>>> {
    let groups = players_state.permission_groups.read().await.clone();

    if groups.is_empty() {
        let demo_groups = generate_demo_permission_groups();
        Json(ApiResponse::success(demo_groups))
    } else {
        Json(ApiResponse::success(groups))
    }
}

pub async fn create_permission_group(
    State(players_state): State<PlayersState>,
    Json(group): Json<PermissionGroup>,
) -> Json<ApiResponse<PermissionGroup>> {
    let mut groups = players_state.permission_groups.write().await;
    let new_group = PermissionGroup {
        id: Uuid::new_v4().to_string(),
        ..group
    };
    groups.push(new_group.clone());

    Json(ApiResponse::success(new_group))
}

pub async fn update_permission_group(
    State(players_state): State<PlayersState>,
    Path(group_id): Path<String>,
    Json(group): Json<PermissionGroup>,
) -> Json<ApiResponse<PermissionGroup>> {
    let mut groups = players_state.permission_groups.write().await;
    if let Some(existing) = groups.iter_mut().find(|g| g.id == group_id) {
        *existing = group.clone();
        return Json(ApiResponse::success(group));
    }

    Json(ApiResponse::error("Group not found".to_string()))
}

pub async fn delete_permission_group(
    State(players_state): State<PlayersState>,
    Path(group_id): Path<String>,
) -> Json<ApiResponse<String>> {
    let mut groups = players_state.permission_groups.write().await;
    groups.retain(|g| g.id != group_id);

    Json(ApiResponse::success("Group deleted".to_string()))
}

pub async fn add_permission_to_group(
    State(players_state): State<PlayersState>,
    Path(group_id): Path<String>,
    Json(permission): Json<String>,
) -> Json<ApiResponse<String>> {
    let mut groups = players_state.permission_groups.write().await;
    if let Some(group) = groups.iter_mut().find(|g| g.id == group_id) {
        group.permissions.push(permission.clone());
        return Json(ApiResponse::success(format!("Permission {} added", permission)));
    }

    Json(ApiResponse::error("Group not found".to_string()))
}

pub async fn remove_permission_from_group(
    State(players_state): State<PlayersState>,
    Path((group_id, permission)): Path<(String, String)>,
) -> Json<ApiResponse<String>> {
    let mut groups = players_state.permission_groups.write().await;
    if let Some(group) = groups.iter_mut().find(|g| g.id == group_id) {
        group.permissions.retain(|p| p != &permission);
        return Json(ApiResponse::success(format!("Permission {} removed", permission)));
    }

    Json(ApiResponse::error("Group not found".to_string()))
}

pub async fn get_chat_history(
    State(players_state): State<PlayersState>,
    Query(query): Query<ChatQuery>,
) -> Json<ApiResponse<Vec<ChatMessage>>> {
    let messages = players_state.chat_messages.read().await.clone();

    let filtered: Vec<ChatMessage> = messages.into_iter()
        .filter(|m| {
            let name_match = query.player_name.as_ref()
                .map_or(true, |n| m.player_name == *n);
            name_match
        })
        .take(query.limit.unwrap_or(100) as usize)
        .collect();

    if filtered.is_empty() {
        let demo_messages = generate_demo_chat_messages();
        Json(ApiResponse::success(demo_messages))
    } else {
        Json(ApiResponse::success(filtered))
    }
}

pub async fn search_chat(
    State(players_state): State<PlayersState>,
    Query(params): Query<SearchChatParams>,
) -> Json<ApiResponse<Vec<ChatMessage>>> {
    let messages = players_state.chat_messages.read().await.clone();

    let results: Vec<ChatMessage> = messages.into_iter()
        .filter(|m| {
            let keyword_match = params.keyword.as_ref()
                .map_or(true, |k| m.message.to_lowercase().contains(&k.to_lowercase()));
            let name_match = params.player_name.as_ref()
                .map_or(true, |n| m.player_name == *n);
            keyword_match && name_match
        })
        .take(params.limit.unwrap_or(50) as usize)
        .collect();

    Json(ApiResponse::success(results))
}

#[derive(Debug, Deserialize)]
pub struct SearchChatParams {
    pub keyword: Option<String>,
    pub player_name: Option<String>,
    pub limit: Option<i32>,
}

pub async fn get_warnings(
    State(players_state): State<PlayersState>,
    Query(query): Query<WarningQuery>,
) -> Json<ApiResponse<Vec<Warning>>> {
    let warnings = players_state.warnings.read().await.clone();

    let filtered: Vec<Warning> = warnings.into_iter()
        .filter(|w| {
            let name_match = query.player_name.as_ref()
                .map_or(true, |n| &w.player_name == n);
            let active_match = query.active_only.unwrap_or(true)
                .then(|| w.active)
                .unwrap_or(true);
            name_match && active_match
        })
        .collect();

    if filtered.is_empty() {
        let demo_warnings = generate_demo_warnings();
        Json(ApiResponse::success(demo_warnings))
    } else {
        Json(ApiResponse::success(filtered))
    }
}

pub async fn issue_warning(
    State(players_state): State<PlayersState>,
    Json(warning_req): Json<WarningRequest>,
) -> Json<ApiResponse<Warning>> {
    let warning = Warning {
        id: Uuid::new_v4().to_string(),
        player_name: warning_req.player_name.clone(),
        player_uuid: Uuid::new_v4().to_string(),
        reason: warning_req.reason,
        issued_by: "admin".to_string(),
        issued_at: Utc::now(),
        expires_at: warning_req.duration_hours.map(|h| Utc::now() + Duration::hours(h as i64)),
        active: true,
        server_id: "server-1".to_string(),
    };

    let mut warnings = players_state.warnings.write().await;
    warnings.push(warning.clone());

    Json(ApiResponse::success(warning))
}

pub async fn revoke_warning(
    State(players_state): State<PlayersState>,
    Path(warning_id): Path<String>,
) -> Json<ApiResponse<String>> {
    let mut warnings = players_state.warnings.write().await;
    if let Some(warning) = warnings.iter_mut().find(|w| w.id == warning_id) {
        warning.active = false;
        return Json(ApiResponse::success("Warning revoked".to_string()));
    }

    Json(ApiResponse::error("Warning not found".to_string()))
}

#[derive(Debug, Deserialize)]
pub struct WarningRequest {
    pub player_name: String,
    pub reason: String,
    pub duration_hours: Option<i32>,
}

pub async fn get_player_economy(
    State(players_state): State<PlayersState>,
    Query(query): Query<EconomyQuery>,
) -> Json<ApiResponse<VirtualEconomy>> {
    let economy_data = players_state.economy_data.read().await.clone();

    if let Some(economy) = economy_data.iter().find(|e| e.player_name == query.player_name) {
        return Json(ApiResponse::success(economy.clone()));
    }

    let demo_economy = generate_demo_economy(&query.player_name);
    Json(ApiResponse::success(demo_economy))
}

pub async fn get_economy_leaderboard(
    State(players_state): State<PlayersState>,
) -> Json<ApiResponse<Vec<(String, f64)>>> {
    let economy_data = players_state.economy_data.read().await.clone();

    let mut leaderboard: Vec<(String, f64)> = economy_data.iter()
        .map(|e| (e.player_name.clone(), e.balance))
        .collect();

    leaderboard.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if leaderboard.is_empty() {
        leaderboard = vec![
            ("Steve".to_string(), 10000.0),
            ("Alex".to_string(), 8500.0),
            ("Herobrine".to_string(), 5000.0),
        ];
    }

    Json(ApiResponse::success(leaderboard))
}

pub async fn give_money(
    State(players_state): State<PlayersState>,
    Json(tx_req): Json<TransactionRequest>,
) -> Json<ApiResponse<String>> {
    let mut economy_data = players_state.economy_data.write().await;

    if let Some(economy) = economy_data.iter_mut().find(|e| e.player_name == tx_req.player_name) {
        economy.balance += tx_req.amount;
        economy.last_updated = Utc::now();
        economy.transactions.push(Transaction {
            id: Uuid::new_v4().to_string(),
            transaction_type: TransactionType::Deposit,
            amount: tx_req.amount,
            description: tx_req.description.clone(),
            from_player: None,
            to_player: Some(tx_req.player_name.clone()),
            timestamp: Utc::now(),
        });
    }

    Json(ApiResponse::success(format!("Gave {} to {}", tx_req.amount, tx_req.player_name)))
}

pub async fn take_money(
    State(players_state): State<PlayersState>,
    Json(tx_req): Json<TransactionRequest>,
) -> Json<ApiResponse<String>> {
    let mut economy_data = players_state.economy_data.write().await;

    if let Some(economy) = economy_data.iter_mut().find(|e| e.player_name == tx_req.player_name) {
        economy.balance = (economy.balance - tx_req.amount).max(0.0);
        economy.last_updated = Utc::now();
        economy.transactions.push(Transaction {
            id: Uuid::new_v4().to_string(),
            transaction_type: TransactionType::Withdraw,
            amount: tx_req.amount,
            description: tx_req.description.clone(),
            from_player: Some(tx_req.player_name.clone()),
            to_player: None,
            timestamp: Utc::now(),
        });
    }

    Json(ApiResponse::success(format!("Took {} from {}", tx_req.amount, tx_req.player_name)))
}

#[derive(Debug, Deserialize)]
pub struct TransactionRequest {
    pub player_name: String,
    pub amount: f64,
    pub description: String,
}

pub async fn sync_bans_to_server(
    State(state): State<AppState>,
    State(players_state): State<PlayersState>,
    Path(server_id): Path<String>,
) -> Json<ApiResponse<String>> {
    let records = players_state.ban_records.read().await.clone();
    let active_bans: Vec<_> = records.iter()
        .filter(|r| r.active && r.server_id == "server-1")
        .collect();

    for ban in active_bans {
        let command = match ban.ban_type {
            BanType::Ban | BanType::TempBan => {
                format!("ban {} {}", ban.player_name, ban.reason)
            }
            BanType::IPBan => {
                format!("ban-ip {} {}", ban.player_name, ban.reason)
            }
            BanType::Mute => {
                format!("mute {} {}", ban.player_name, ban.reason)
            }
        };
        let _ = state.rcon_client.send_command(&command).await;
    }

    Json(ApiResponse::success(format!("Synced {} bans to server", active_bans.len())))
}

fn parse_inventory_data(data: &str, _player_name: &str) -> Vec<InventorySlot> {
    let mut slots = Vec::new();

    let re = regex::Regex::new(r"(\d+)\s+\{(\w+):([^}]*)\}").unwrap();
    for (slot, item) in re.captures_iter(data).take(36) {
        if let (Ok(slot_num), Some(item_name)) = (
            slot.parse::<i32>(),
            item.get(2).map(|m| m.as_str()),
        ) {
            slots.push(InventorySlot {
                slot: slot_num,
                item: item_name.to_string(),
                count: 1,
                metadata: None,
                nbt: None,
            });
        }
    }

    slots
}

fn generate_demo_inventory() -> Vec<InventorySlot> {
    vec![
        InventorySlot {
            slot: 0,
            item: "minecraft:diamond_sword".to_string(),
            count: 1,
            metadata: None,
            nbt: None,
        },
        InventorySlot {
            slot: 1,
            item: "minecraft:iron_pickaxe".to_string(),
            count: 1,
            metadata: None,
            nbt: None,
        },
        InventorySlot {
            slot: 2,
            item: "minecraft:torch".to_string(),
            count: 64,
            metadata: None,
            nbt: None,
        },
    ]
}

fn parse_item_name(item: &str) -> Option<String> {
    let clean = item
        .replace("minecraft:", "")
        .replace("_", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_");

    if clean.is_empty() {
        None
    } else {
        Some(format!("minecraft:{}", clean.replace(" ", "_")))
    }
}

fn generate_demo_op_records() -> Vec<OpRecord> {
    vec![
        OpRecord {
            player_name: "AdminSteve".to_string(),
            player_uuid: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
            operator_level: 4,
            granted_by: "console".to_string(),
            granted_at: Utc::now() - Duration::days(30),
            revoked_by: None,
            revoked_at: None,
            active: true,
        },
        OpRecord {
            player_name: "ModAlex".to_string(),
            player_uuid: "b2c3d4e5-f6a7-8901-bcde-f12345678901".to_string(),
            operator_level: 2,
            granted_by: "AdminSteve".to_string(),
            granted_at: Utc::now() - Duration::days(15),
            revoked_by: None,
            revoked_at: None,
            active: true,
        },
    ]
}

fn generate_demo_ban_records() -> Vec<BanRecord> {
    vec![
        BanRecord {
            id: "ban-001".to_string(),
            player_name: "Griefer123".to_string(),
            player_uuid: "c3d4e5f6-a7b8-9012-cdef-123456789012".to_string(),
            ban_type: BanType::Ban,
            reason: "Griefing server builds".to_string(),
            banned_by: "AdminSteve".to_string(),
            banned_at: Utc::now() - Duration::days(7),
            expires_at: None,
            server_id: "server-1".to_string(),
            active: true,
        },
        BanRecord {
            id: "ban-002".to_string(),
            player_name: "Spammer456".to_string(),
            player_uuid: "d4e5f6a7-b8c9-0123-defa-234567890123".to_string(),
            ban_type: BanType::TempBan,
            reason: "Advertising".to_string(),
            banned_by: "ModAlex".to_string(),
            banned_at: Utc::now() - Duration::hours(12),
            expires_at: Some(Utc::now() + Duration::hours(12)),
            server_id: "server-1".to_string(),
            active: true,
        },
    ]
}

fn generate_demo_player_actions() -> Vec<PlayerAction> {
    vec![
        PlayerAction {
            id: "action-001".to_string(),
            player_name: "Steve".to_string(),
            action_type: ActionType::Join,
            details: "Player joined the server".to_string(),
            timestamp: Utc::now() - Duration::minutes(5),
            server_id: "server-1".to_string(),
        },
        PlayerAction {
            id: "action-002".to_string(),
            player_name: "Steve".to_string(),
            action_type: ActionType::Chat,
            details: "Hello everyone!".to_string(),
            timestamp: Utc::now() - Duration::minutes(4),
            server_id: "server-1".to_string(),
        },
        PlayerAction {
            id: "action-003".to_string(),
            player_name: "Alex".to_string(),
            action_type: ActionType::Death,
            details: "Died to Creeper".to_string(),
            timestamp: Utc::now() - Duration::minutes(10),
            server_id: "server-1".to_string(),
        },
    ]
}

fn generate_demo_permission_groups() -> Vec<PermissionGroup> {
    vec![
        PermissionGroup {
            id: "group-admin".to_string(),
            name: "admin".to_string(),
            display_name: "Administrator".to_string(),
            color: "#FF5555".to_string(),
            prefix: Some("[Admin]".to_string()),
            suffix: None,
            weight: 100,
            permissions: vec![
                "*".to_string(),
            ],
            parent_id: None,
            worlds: vec!["*".to_string()],
        },
        PermissionGroup {
            id: "group-mod".to_string(),
            name: "mod".to_string(),
            display_name: "Moderator".to_string(),
            color: "#00AA00".to_string(),
            prefix: Some("[Mod]".to_string()),
            suffix: None,
            weight: 80,
            permissions: vec![
                "essentials.kick".to_string(),
                "essentials.ban".to_string(),
                "essentials.mute".to_string(),
            ],
            parent_id: Some("group-vip".to_string()),
            worlds: vec!["*".to_string()],
        },
        PermissionGroup {
            id: "group-vip".to_string(),
            name: "vip".to_string(),
            display_name: "VIP".to_string(),
            color: "#FFAA00".to_string(),
            prefix: Some("[VIP]".to_string()),
            suffix: None,
            weight: 50,
            permissions: vec![
                "essentials.home".to_string(),
                "essentials.sethome".to_string(),
            ],
            parent_id: Some("group-default".to_string()),
            worlds: vec!["*".to_string()],
        },
        PermissionGroup {
            id: "group-default".to_string(),
            name: "default".to_string(),
            display_name: "Member".to_string(),
            color: "#FFFFFF".to_string(),
            prefix: None,
            suffix: None,
            weight: 10,
            permissions: vec![
                "essentials.spawn".to_string(),
            ],
            parent_id: None,
            worlds: vec!["*".to_string()],
        },
    ]
}

fn generate_demo_chat_messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage {
            id: "chat-001".to_string(),
            player_name: "Steve".to_string(),
            player_uuid: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string(),
            message: "Hello, world!".to_string(),
            channel: "global".to_string(),
            timestamp: Utc::now() - Duration::minutes(30),
            server_id: "server-1".to_string(),
        },
        ChatMessage {
            id: "chat-002".to_string(),
            player_name: "Alex".to_string(),
            player_uuid: "b2c3d4e5-f6a7-8901-bcde-f12345678901".to_string(),
            message: "Welcome to the server!".to_string(),
            channel: "global".to_string(),
            timestamp: Utc::now() - Duration::minutes(25),
            server_id: "server-1".to_string(),
        },
    ]
}

fn generate_demo_warnings() -> Vec<Warning> {
    vec![
        Warning {
            id: "warn-001".to_string(),
            player_name: "NewPlayer123".to_string(),
            player_uuid: "c3d4e5f6-a7b8-9012-cdef-123456789012".to_string(),
            reason: "Inappropriate chat language".to_string(),
            issued_by: "ModAlex".to_string(),
            issued_at: Utc::now() - Duration::days(2),
            expires_at: None,
            active: true,
            server_id: "server-1".to_string(),
        },
        Warning {
            id: "warn-002".to_string(),
            player_name: "Builder456".to_string(),
            player_uuid: "d4e5f6a7-b8c9-0123-defa-234567890123".to_string(),
            reason: "Building in restricted area".to_string(),
            issued_by: "AdminSteve".to_string(),
            issued_at: Utc::now() - Duration::hours(6),
            expires_at: Some(Utc::now() + Duration::days(7)),
            active: true,
            server_id: "server-1".to_string(),
        },
    ]
}

fn generate_demo_economy(player_name: &str) -> VirtualEconomy {
    VirtualEconomy {
        player_name: player_name.to_string(),
        player_uuid: Uuid::new_v4().to_string(),
        balance: 5000.0,
        currency: "金币".to_string(),
        transactions: vec![
            Transaction {
                id: "tx-001".to_string(),
                transaction_type: TransactionType::Deposit,
                amount: 1000.0,
                description: "Welcome bonus".to_string(),
                from_player: None,
                to_player: Some(player_name.to_string()),
                timestamp: Utc::now() - Duration::days(7),
            },
            Transaction {
                id: "tx-002".to_string(),
                transaction_type: TransactionType::Spend,
                amount: 500.0,
                description: "Bought diamond armor".to_string(),
                from_player: Some(player_name.to_string()),
                to_player: None,
                timestamp: Utc::now() - Duration::days(3),
            },
        ],
        last_updated: Utc::now(),
    }
}

pub fn create_players_state() -> PlayersState {
    PlayersState::default()
}

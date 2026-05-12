use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub uuid: String,
    pub location: PlayerLocation,
    pub inventory: Option<Vec<InventorySlot>>,
    pub health: Option<i32>,
    pub hunger: Option<i32>,
    pub gamemode: Option<String>,
    pub online: bool,
    pub first_join: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub play_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLocation {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub slot: i32,
    pub item: String,
    pub count: i32,
    pub metadata: Option<i32>,
    pub nbt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMapData {
    pub players: Vec<Player>,
    pub world_name: String,
    pub world_border: WorldBorder,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBorder {
    pub center_x: f64,
    pub center_z: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpRecord {
    pub player_name: String,
    pub player_uuid: String,
    pub operator_level: i32,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    pub revoked_by: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanRecord {
    pub id: String,
    pub player_name: String,
    pub player_uuid: String,
    pub ban_type: BanType,
    pub reason: String,
    pub banned_by: String,
    pub banned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub server_id: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BanType {
    Ban,
    TempBan,
    IPBan,
    Mute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub id: String,
    pub player_name: String,
    pub action_type: ActionType,
    pub details: String,
    pub timestamp: DateTime<Utc>,
    pub server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Join,
    Leave,
    Chat,
    Command,
    Death,
    Kill,
    Trade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBackup {
    pub id: String,
    pub player_name: String,
    pub player_uuid: String,
    pub backup_type: BackupType,
    pub file_path: String,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
    pub server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Inventory,
    Stats,
    Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub color: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub weight: i32,
    pub permissions: Vec<String>,
    pub parent_id: Option<String>,
    pub worlds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub player_name: String,
    pub player_uuid: String,
    pub message: String,
    pub channel: String,
    pub timestamp: DateTime<Utc>,
    pub server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub id: String,
    pub player_name: String,
    pub player_uuid: String,
    pub reason: String,
    pub issued_by: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
    pub server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualEconomy {
    pub player_name: String,
    pub player_uuid: String,
    pub balance: f64,
    pub currency: String,
    pub transactions: Vec<Transaction>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub description: String,
    pub from_player: Option<String>,
    pub to_player: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdraw,
    Transfer,
    Earn,
    Spend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub player_name: String,
    pub total_playtime: i64,
    pub total_deaths: i32,
    pub total_kills: i32,
    pub blocks_broken: i32,
    pub blocks_placed: i32,
    pub items_crafted: i32,
    pub distance_traveled: f64,
    pub sessions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSearchQuery {
    pub player_name: Option<String>,
    pub action_type: Option<ActionType>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

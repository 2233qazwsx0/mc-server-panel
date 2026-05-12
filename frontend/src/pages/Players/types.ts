export interface Player {
  name: string;
  uuid: string;
  location: PlayerLocation;
  inventory: InventorySlot[] | null;
  health: number | null;
  hunger: number | null;
  gamemode: string | null;
  online: boolean;
  first_join: string | null;
  last_seen: string | null;
  play_time: number;
}

export interface PlayerLocation {
  world: string;
  x: number;
  y: number;
  z: number;
  yaw: number | null;
  pitch: number | null;
}

export interface InventorySlot {
  slot: number;
  item: string;
  count: number;
  metadata: number | null;
  nbt: string | null;
}

export interface PlayerMapData {
  players: Player[];
  world_name: string;
  world_border: WorldBorder;
  last_updated: string;
}

export interface WorldBorder {
  center_x: number;
  center_z: number;
  size: number;
}

export interface OpRecord {
  player_name: string;
  player_uuid: string;
  operator_level: number;
  granted_by: string;
  granted_at: string;
  revoked_by: string | null;
  revoked_at: string | null;
  active: boolean;
}

export interface BanRecord {
  id: string;
  player_name: string;
  player_uuid: string;
  ban_type: 'Ban' | 'TempBan' | 'IPBan' | 'Mute';
  reason: string;
  banned_by: string;
  banned_at: string;
  expires_at: string | null;
  server_id: string;
  active: boolean;
}

export interface PlayerAction {
  id: string;
  player_name: string;
  action_type: ActionType;
  details: string;
  timestamp: string;
  server_id: string;
}

export type ActionType = 'Join' | 'Leave' | 'Chat' | 'Command' | 'Death' | 'Kill' | 'Trade';

export interface PlayerBackup {
  id: string;
  player_name: string;
  player_uuid: string;
  backup_type: BackupType;
  file_path: string;
  file_size: number;
  created_at: string;
  server_id: string;
}

export type BackupType = 'Full' | 'Inventory' | 'Stats' | 'Location';

export interface PermissionGroup {
  id: string;
  name: string;
  display_name: string;
  color: string;
  prefix: string | null;
  suffix: string | null;
  weight: number;
  permissions: string[];
  parent_id: string | null;
  worlds: string[];
}

export interface ChatMessage {
  id: string;
  player_name: string;
  player_uuid: string;
  message: string;
  channel: string;
  timestamp: string;
  server_id: string;
}

export interface Warning {
  id: string;
  player_name: string;
  player_uuid: string;
  reason: string;
  issued_by: string;
  issued_at: string;
  expires_at: string | null;
  active: boolean;
  server_id: string;
}

export interface VirtualEconomy {
  player_name: string;
  player_uuid: string;
  balance: number;
  currency: string;
  transactions: Transaction[];
  last_updated: string;
}

export interface Transaction {
  id: string;
  transaction_type: TransactionType;
  amount: number;
  description: string;
  from_player: string | null;
  to_player: string | null;
  timestamp: string;
}

export type TransactionType = 'Deposit' | 'Withdraw' | 'Transfer' | 'Earn' | 'Spend';

export interface PlayerStats {
  player_name: string;
  total_playtime: number;
  total_deaths: number;
  total_kills: number;
  blocks_broken: number;
  blocks_placed: number;
  items_crafted: number;
  distance_traveled: number;
  sessions: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

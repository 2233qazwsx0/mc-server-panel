import { ApiResponse, PlayerMapData, InventorySlot, OpRecord, BanRecord, PlayerAction, PlayerBackup, PermissionGroup, ChatMessage, Warning, VirtualEconomy, PlayerStats } from './types';

const API_BASE = '/api';

async function fetchApi<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${url}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  const result: ApiResponse<T> = await response.json();

  if (!result.success) {
    throw new Error(result.error || 'API request failed');
  }

  return result.data as T;
}

export const playerApi = {
  getPlayerMap: (world?: string) =>
    fetchApi<PlayerMapData>(`/players/map${world ? `?world=${world}` : ''}`),

  getInventory: (playerName: string) =>
    fetchApi<InventorySlot[]>(`/players/${encodeURIComponent(playerName)}/inventory`),

  updateInventory: (playerName: string, inventory: InventorySlot[]) =>
    fetchApi<string>(`/players/${encodeURIComponent(playerName)}/inventory`, {
      method: 'PUT',
      body: JSON.stringify(inventory),
    }),

  getOpList: () => fetchApi<OpRecord[]>('/players/ops'),

  grantOp: (playerName: string, level: number) =>
    fetchApi<OpRecord>(`/players/ops/${encodeURIComponent(playerName)}`, {
      method: 'POST',
      body: JSON.stringify(level),
    }),

  revokeOp: (playerName: string) =>
    fetchApi<string>(`/players/ops/${encodeURIComponent(playerName)}`, {
      method: 'DELETE',
    }),

  getBanList: (serverId?: string, activeOnly?: boolean) => {
    const params = new URLSearchParams();
    if (serverId) params.append('server_id', serverId);
    if (activeOnly !== undefined) params.append('active_only', String(activeOnly));
    return fetchApi<BanRecord[]>(`/players/bans?${params}`);
  },

  banPlayer: (playerName: string, reason: string, durationHours?: number) =>
    fetchApi<BanRecord>('/players/bans', {
      method: 'POST',
      body: JSON.stringify({ player_name: playerName, reason, duration_hours: durationHours }),
    }),

  unbanPlayer: (playerName: string) =>
    fetchApi<string>(`/players/bans/${encodeURIComponent(playerName)}`, {
      method: 'DELETE',
    }),

  syncBans: (serverId: string) =>
    fetchApi<string>(`/players/bans/sync/${encodeURIComponent(serverId)}`, {
      method: 'POST',
    }),

  getPlayerActions: (playerName?: string, actionType?: string, limit?: number) => {
    const params = new URLSearchParams();
    if (playerName) params.append('player_name', playerName);
    if (actionType) params.append('action_type', actionType);
    if (limit) params.append('limit', String(limit));
    return fetchApi<PlayerAction[]>(`/players/actions?${params}`);
  },

  getPlayerStats: (playerName: string) =>
    fetchApi<PlayerStats>(`/players/${encodeURIComponent(playerName)}/stats`),

  createBackup: (playerName: string, backupType: string) =>
    fetchApi<PlayerBackup>(`/players/${encodeURIComponent(playerName)}/backup/${backupType}`, {
      method: 'POST',
    }),

  getBackups: (playerName: string) =>
    fetchApi<PlayerBackup[]>(`/players/backups?player_name=${encodeURIComponent(playerName)}`),

  restoreBackup: (playerName: string, backupId: string) =>
    fetchApi<string>(`/players/${encodeURIComponent(playerName)}/backup/${backupId}`, {
      method: 'POST',
    }),

  getPermissionGroups: () => fetchApi<PermissionGroup[]>('/players/permissions/groups'),

  createPermissionGroup: (group: PermissionGroup) =>
    fetchApi<PermissionGroup>('/players/permissions/groups', {
      method: 'POST',
      body: JSON.stringify(group),
    }),

  updatePermissionGroup: (groupId: string, group: PermissionGroup) =>
    fetchApi<PermissionGroup>(`/players/permissions/groups/${groupId}`, {
      method: 'PUT',
      body: JSON.stringify(group),
    }),

  deletePermissionGroup: (groupId: string) =>
    fetchApi<string>(`/players/permissions/groups/${groupId}`, {
      method: 'DELETE',
    }),

  addPermission: (groupId: string, permission: string) =>
    fetchApi<string>(`/players/permissions/groups/${groupId}/permissions`, {
      method: 'POST',
      body: JSON.stringify(permission),
    }),

  removePermission: (groupId: string, permission: string) =>
    fetchApi<string>(`/players/permissions/groups/${groupId}/permissions/${encodeURIComponent(permission)}`, {
      method: 'DELETE',
    }),

  getChatHistory: (playerName?: string, limit?: number) => {
    const params = new URLSearchParams();
    if (playerName) params.append('player_name', playerName);
    if (limit) params.append('limit', String(limit));
    return fetchApi<ChatMessage[]>(`/players/chat?${params}`);
  },

  searchChat: (keyword?: string, playerName?: string, limit?: number) => {
    const params = new URLSearchParams();
    if (keyword) params.append('keyword', keyword);
    if (playerName) params.append('player_name', playerName);
    if (limit) params.append('limit', String(limit));
    return fetchApi<ChatMessage[]>(`/players/chat/search?${params}`);
  },

  getWarnings: (playerName?: string, activeOnly?: boolean) => {
    const params = new URLSearchParams();
    if (playerName) params.append('player_name', playerName);
    if (activeOnly !== undefined) params.append('active_only', String(activeOnly));
    return fetchApi<Warning[]>(`/players/warnings?${params}`);
  },

  issueWarning: (playerName: string, reason: string, durationHours?: number) =>
    fetchApi<Warning>('/players/warnings', {
      method: 'POST',
      body: JSON.stringify({ player_name: playerName, reason, duration_hours: durationHours }),
    }),

  revokeWarning: (warningId: string) =>
    fetchApi<string>(`/players/warnings/${warningId}`, {
      method: 'DELETE',
    }),

  getEconomy: (playerName: string) =>
    fetchApi<VirtualEconomy>(`/players/economy?player_name=${encodeURIComponent(playerName)}`),

  getEconomyLeaderboard: () => fetchApi<[string, number][]>('/players/economy/leaderboard'),

  giveMoney: (playerName: string, amount: number, description: string) =>
    fetchApi<string>('/players/economy/give', {
      method: 'POST',
      body: JSON.stringify({ player_name: playerName, amount, description }),
    }),

  takeMoney: (playerName: string, amount: number, description: string) =>
    fetchApi<string>('/players/economy/take', {
      method: 'POST',
      body: JSON.stringify({ player_name: playerName, amount, description }),
    }),
};

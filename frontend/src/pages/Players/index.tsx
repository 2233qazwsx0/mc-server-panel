import { useState, useEffect, useCallback } from 'react';
import {
  Users,
  Backpack,
  Shield,
  Ban,
  BarChart3,
  Database,
  UserCog,
  MessageSquare,
  AlertTriangle,
  Coins,
  RefreshCw,
  Search,
  Plus,
  Trash2,
  Edit3,
  ChevronRight,
  Clock,
  MapPin,
  X,
} from 'lucide-react';
import { clsx } from 'clsx';
import { playerApi } from './api';
import {
  Player,
  PlayerMapData,
  InventorySlot,
  OpRecord,
  BanRecord,
  PlayerAction,
  PermissionGroup,
  ChatMessage,
  Warning,
  VirtualEconomy,
  PlayerBackup,
} from './types';

type TabType = 'map' | 'inventory' | 'ops' | 'bans' | 'analytics' | 'backup' | 'permissions' | 'chat' | 'warnings' | 'economy';

interface Tab {
  id: TabType;
  label: string;
  icon: typeof Users;
  priority: 'P0' | 'P1' | 'P2';
}

const tabs: Tab[] = [
  { id: 'map', label: '实时玩家地图', icon: MapPin, priority: 'P0' },
  { id: 'inventory', label: '背包查看与编辑', icon: Backpack, priority: 'P1' },
  { id: 'ops', label: 'OP权限审计', icon: Shield, priority: 'P0' },
  { id: 'bans', label: 'Ban名单同步', icon: Ban, priority: 'P1' },
  { id: 'analytics', label: '玩家行为分析', icon: BarChart3, priority: 'P2' },
  { id: 'backup', label: '玩家数据备份', icon: Database, priority: 'P0' },
  { id: 'permissions', label: '权限组可视化', icon: UserCog, priority: 'P1' },
  { id: 'chat', label: '聊天记录检索', icon: MessageSquare, priority: 'P0' },
  { id: 'warnings', label: '玩家警告系统', icon: AlertTriangle, priority: 'P1' },
  { id: 'economy', label: '虚拟经济查询', icon: Coins, priority: 'P1' },
];

export function Players() {
  const [activeTab, setActiveTab] = useState<TabType>('map');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="flex flex-col h-full">
      <header className="bg-nether-800 border-b border-nether-600 p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Users className="w-6 h-6 text-mc-green" />
            <h1 className="text-xl font-display font-bold gradient-text-primary">玩家与权限中心</h1>
          </div>
          <button
            onClick={() => setError(null)}
            className="btn btn-secondary"
          >
            <RefreshCw className="w-4 h-4 mr-2" />
            刷新
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-hidden flex flex-col">
        <nav className="bg-nether-800 border-b border-nether-600 px-4 overflow-x-auto">
          <div className="flex gap-1 py-2 min-w-max">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={clsx(
                  'flex items-center gap-2 px-4 py-2 rounded-lg font-mono text-sm transition-all duration-200',
                  activeTab === tab.id
                    ? 'bg-mc-green/20 text-mc-green border border-mc-green/50'
                    : 'text-text-secondary hover:text-text-primary hover:bg-nether-700'
                )}
              >
                <tab.icon className="w-4 h-4" />
                <span>{tab.label}</span>
                <span className={clsx(
                  'text-xs px-1.5 py-0.5 rounded',
                  tab.priority === 'P0' && 'bg-mc-green/20 text-mc-green',
                  tab.priority === 'P1' && 'bg-yellow-500/20 text-yellow-400',
                  tab.priority === 'P2' && 'bg-gray-500/20 text-gray-400'
                )}>
                  {tab.priority}
                </span>
              </button>
            ))}
          </div>
        </nav>

        <main className="flex-1 overflow-auto p-6 bg-nether-900">
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <RefreshCw className="w-8 h-8 text-mc-green animate-spin" />
            </div>
          ) : error ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center">
                <p className="text-red-400 mb-4">{error}</p>
                <button onClick={() => setError(null)} className="btn btn-secondary">
                  重试
                </button>
              </div>
            </div>
          ) : (
            <TabContent activeTab={activeTab} />
          )}
        </main>
      </div>
    </div>
  );
}

function TabContent({ activeTab }: { activeTab: TabType }) {
  switch (activeTab) {
    case 'map':
      return <PlayerMapTab />;
    case 'inventory':
      return <InventoryTab />;
    case 'ops':
      return <OpAuditTab />;
    case 'bans':
      return <BanListTab />;
    case 'analytics':
      return <AnalyticsTab />;
    case 'backup':
      return <BackupTab />;
    case 'permissions':
      return <PermissionsTab />;
    case 'chat':
      return <ChatTab />;
    case 'warnings':
      return <WarningsTab />;
    case 'economy':
      return <EconomyTab />;
    default:
      return <div>功能开发中...</div>;
  }
}

function PlayerMapTab() {
  const [mapData, setMapData] = useState<PlayerMapData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadMapData();
  }, []);

  const loadMapData = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getPlayerMap();
      setMapData(data);
    } catch (e) {
      console.error('Failed to load map data:', e);
    }
    setLoading(false);
  };

  if (loading) {
    return <div className="flex items-center justify-center h-64"><RefreshCw className="w-8 h-8 animate-spin" /></div>;
  }

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
          <h2 className="text-lg font-display font-bold mb-4 flex items-center gap-2">
            <MapPin className="w-5 h-5 text-mc-green" />
            世界边界
          </h2>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-text-muted text-sm">世界名称</p>
              <p className="text-text-primary font-mono">{mapData?.world_name || 'world'}</p>
            </div>
            <div>
              <p className="text-text-muted text-sm">边界大小</p>
              <p className="text-text-primary font-mono">{mapData?.world_border.size.toLocaleString() || '60000000'}</p>
            </div>
            <div>
              <p className="text-text-muted text-sm">中心 X</p>
              <p className="text-text-primary font-mono">{mapData?.world_border.center_x || 0}</p>
            </div>
            <div>
              <p className="text-text-muted text-sm">中心 Z</p>
              <p className="text-text-primary font-mono">{mapData?.world_border.center_z || 0}</p>
            </div>
          </div>
        </div>

        <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
          <h2 className="text-lg font-display font-bold mb-4 flex items-center gap-2">
            <Users className="w-5 h-5 text-mc-green" />
            在线玩家 ({mapData?.players.length || 0})
          </h2>
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {mapData?.players.map((player) => (
              <div
                key={player.name}
                className="flex items-center justify-between p-3 bg-nether-700 rounded-lg"
              >
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 bg-mc-green/20 rounded-full flex items-center justify-center">
                    <span className="text-mc-green font-bold">{player.name[0].toUpperCase()}</span>
                  </div>
                  <div>
                    <p className="text-text-primary font-mono">{player.name}</p>
                    <p className="text-text-muted text-xs">{player.gamemode || 'survival'}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-text-muted text-xs">
                    {player.location.x.toFixed(0)}, {player.location.y.toFixed(0)}, {player.location.z.toFixed(0)}
                  </p>
                </div>
              </div>
            ))}
            {(!mapData?.players || mapData.players.length === 0) && (
              <p className="text-text-muted text-center py-4">暂无在线玩家</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function InventoryTab() {
  const [players, setPlayers] = useState<Player[]>([]);
  const [selectedPlayer, setSelectedPlayer] = useState<string>('');
  const [inventory, setInventory] = useState<InventorySlot[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadPlayers();
  }, []);

  useEffect(() => {
    if (selectedPlayer) {
      loadInventory(selectedPlayer);
    }
  }, [selectedPlayer]);

  const loadPlayers = async () => {
    try {
      const data = await playerApi.getPlayerMap();
      setPlayers(data.players);
      if (data.players.length > 0) {
        setSelectedPlayer(data.players[0].name);
      }
    } catch (e) {
      console.error('Failed to load players:', e);
    }
  };

  const loadInventory = async (playerName: string) => {
    setLoading(true);
    try {
      const data = await playerApi.getInventory(playerName);
      setInventory(data);
    } catch (e) {
      console.error('Failed to load inventory:', e);
    }
    setLoading(false);
  };

  return (
    <div className="space-y-6">
      <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
        <h2 className="text-lg font-display font-bold mb-4 flex items-center gap-2">
          <Backpack className="w-5 h-5 text-mc-green" />
          背包查看与编辑
        </h2>

        <div className="flex gap-4 mb-6">
          <select
            value={selectedPlayer}
            onChange={(e) => setSelectedPlayer(e.target.value)}
            className="input flex-1"
          >
            <option value="">选择玩家</option>
            {players.map((p) => (
              <option key={p.name} value={p.name}>{p.name}</option>
            ))}
          </select>
          <button
            onClick={() => selectedPlayer && loadInventory(selectedPlayer)}
            className="btn btn-secondary"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
        ) : (
          <div className="grid grid-cols-9 gap-2">
            {Array.from({ length: 36 }).map((_, i) => {
              const slot = inventory.find((s) => s.slot === i);
              return (
                <div
                  key={i}
                  className="aspect-square bg-nether-700 rounded border border-nether-600 flex items-center justify-center cursor-pointer hover:bg-nether-600 transition-colors"
                  title={slot ? `${slot.item} x${slot.count}` : `Slot ${i}`}
                >
                  {slot ? (
                    <div className="text-center">
                      <div className="text-xs text-mc-green font-bold">{slot.count > 1 ? slot.count : ''}</div>
                    </div>
                  ) : (
                    <span className="text-text-muted text-xs">{i}</span>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function OpAuditTab() {
  const [opRecords, setOpRecords] = useState<OpRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [showGrantModal, setShowGrantModal] = useState(false);
  const [targetPlayer, setTargetPlayer] = useState('');

  useEffect(() => {
    loadOpRecords();
  }, []);

  const loadOpRecords = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getOpList();
      setOpRecords(data);
    } catch (e) {
      console.error('Failed to load OP records:', e);
    }
    setLoading(false);
  };

  const handleGrantOp = async () => {
    if (!targetPlayer) return;
    try {
      await playerApi.grantOp(targetPlayer, 4);
      setShowGrantModal(false);
      setTargetPlayer('');
      loadOpRecords();
    } catch (e) {
      console.error('Failed to grant OP:', e);
    }
  };

  const handleRevokeOp = async (playerName: string) => {
    try {
      await playerApi.revokeOp(playerName);
      loadOpRecords();
    } catch (e) {
      console.error('Failed to revoke OP:', e);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-display font-bold flex items-center gap-2">
            <Shield className="w-5 h-5 text-mc-green" />
            OP 权限审计
          </h2>
          <button onClick={() => setShowGrantModal(true)} className="btn btn-primary">
            <Plus className="w-4 h-4 mr-2" />
            授予 OP
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
        ) : (
          <div className="space-y-3">
            {opRecords.map((record) => (
              <div
                key={record.player_uuid}
                className="flex items-center justify-between p-4 bg-nether-700 rounded-lg border border-nether-600"
              >
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 bg-red-500/20 rounded-full flex items-center justify-center">
                    <Shield className="w-5 h-5 text-red-400" />
                  </div>
                  <div>
                    <p className="text-text-primary font-mono font-bold">{record.player_name}</p>
                    <p className="text-text-muted text-sm">
                      等级: {record.operator_level} | 授权者: {record.granted_by}
                    </p>
                    <p className="text-text-muted text-xs">
                      {new Date(record.granted_at).toLocaleString('zh-CN')}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span className={clsx(
                    'px-2 py-1 rounded text-xs font-mono',
                    record.active ? 'bg-mc-green/20 text-mc-green' : 'bg-gray-500/20 text-gray-400'
                  )}>
                    {record.active ? '活跃' : '已撤销'}
                  </span>
                  {record.active && (
                    <button
                      onClick={() => handleRevokeOp(record.player_name)}
                      className="btn btn-danger"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {showGrantModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-nether-800 rounded-lg border border-nether-600 p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-display font-bold">授予 OP 权限</h3>
              <button onClick={() => setShowGrantModal(false)}>
                <X className="w-5 h-5" />
              </button>
            </div>
            <input
              type="text"
              value={targetPlayer}
              onChange={(e) => setTargetPlayer(e.target.value)}
              placeholder="玩家名称"
              className="input w-full mb-4"
            />
            <div className="flex gap-2 justify-end">
              <button onClick={() => setShowGrantModal(false)} className="btn btn-secondary">
                取消
              </button>
              <button onClick={handleGrantOp} className="btn btn-primary">
                确认
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function BanListTab() {
  const [banRecords, setBanRecords] = useState<BanRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [showBanModal, setShowBanModal] = useState(false);
  const [banInfo, setBanInfo] = useState({ playerName: '', reason: '', duration: '' });

  useEffect(() => {
    loadBanRecords();
  }, []);

  const loadBanRecords = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getBanList(undefined, true);
      setBanRecords(data);
    } catch (e) {
      console.error('Failed to load ban records:', e);
    }
    setLoading(false);
  };

  const handleBan = async () => {
    if (!banInfo.playerName || !banInfo.reason) return;
    try {
      await playerApi.banPlayer(
        banInfo.playerName,
        banInfo.reason,
        banInfo.duration ? parseInt(banInfo.duration) : undefined
      );
      setShowBanModal(false);
      setBanInfo({ playerName: '', reason: '', duration: '' });
      loadBanRecords();
    } catch (e) {
      console.error('Failed to ban player:', e);
    }
  };

  const handleUnban = async (playerName: string) => {
    try {
      await playerApi.unbanPlayer(playerName);
      loadBanRecords();
    } catch (e) {
      console.error('Failed to unban player:', e);
    }
  };

  const handleSync = async () => {
    try {
      await playerApi.syncBans('server-1');
      alert('Ban 名单已同步到服务器');
    } catch (e) {
      console.error('Failed to sync bans:', e);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-display font-bold flex items-center gap-2">
            <Ban className="w-5 h-5 text-red-400" />
            Ban 名单管理
          </h2>
          <div className="flex gap-2">
            <button onClick={handleSync} className="btn btn-secondary">
              <RefreshCw className="w-4 h-4 mr-2" />
              同步到服务器
            </button>
            <button onClick={() => setShowBanModal(true)} className="btn btn-danger">
              <Plus className="w-4 h-4 mr-2" />
              添加封禁
            </button>
          </div>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
        ) : (
          <div className="space-y-3">
            {banRecords.map((record) => (
              <div
                key={record.id}
                className="flex items-center justify-between p-4 bg-nether-700 rounded-lg border border-nether-600"
              >
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 bg-red-500/20 rounded-full flex items-center justify-center">
                    <Ban className="w-5 h-5 text-red-400" />
                  </div>
                  <div>
                    <p className="text-text-primary font-mono font-bold">{record.player_name}</p>
                    <p className="text-text-muted text-sm">{record.reason}</p>
                    <p className="text-text-muted text-xs">
                      封禁者: {record.banned_by} | {new Date(record.banned_at).toLocaleString('zh-CN')}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span className={clsx(
                    'px-2 py-1 rounded text-xs font-mono',
                    record.ban_type === 'TempBan' ? 'bg-yellow-500/20 text-yellow-400' :
                    record.ban_type === 'Ban' ? 'bg-red-500/20 text-red-400' : 'bg-gray-500/20 text-gray-400'
                  )}>
                    {record.ban_type === 'TempBan' ? '临时' : record.ban_type === 'Ban' ? '永久' : record.ban_type}
                  </span>
                  {record.expires_at && (
                    <span className="text-text-muted text-xs">
                      至 {new Date(record.expires_at).toLocaleString('zh-CN')}
                    </span>
                  )}
                  <button onClick={() => handleUnban(record.player_name)} className="btn btn-secondary">
                      解封
                  </button>
                </div>
              </div>
            ))}
            {banRecords.length === 0 && (
              <p className="text-text-muted text-center py-8">暂无封禁记录</p>
            )}
          </div>
        )}
      </div>

      {showBanModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-nether-800 rounded-lg border border-nether-600 p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-display font-bold">添加封禁</h3>
              <button onClick={() => setShowBanModal(false)}>
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <input
                type="text"
                value={banInfo.playerName}
                onChange={(e) => setBanInfo({ ...banInfo, playerName: e.target.value })}
                placeholder="玩家名称"
                className="input w-full"
              />
              <input
                type="text"
                value={banInfo.reason}
                onChange={(e) => setBanInfo({ ...banInfo, reason: e.target.value })}
                placeholder="封禁原因"
                className="input w-full"
              />
              <input
                type="text"
                value={banInfo.duration}
                onChange={(e) => setBanInfo({ ...banInfo, duration: e.target.value })}
                placeholder="临时封禁时长（小时，留空为永久）"
                className="input w-full"
              />
            </div>
            <div className="flex gap-2 justify-end mt-6">
              <button onClick={() => setShowBanModal(false)} className="btn btn-secondary">
                取消
              </button>
              <button onClick={handleBan} className="btn btn-danger">
                确认封禁
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function AnalyticsTab() {
  const [actions, setActions] = useState<PlayerAction[]>([]);
  const [loading, setLoading] = useState(true);
  const [playerFilter, setPlayerFilter] = useState('');

  useEffect(() => {
    loadActions();
  }, []);

  const loadActions = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getPlayerActions(playerFilter || undefined, undefined, 50);
      setActions(data);
    } catch (e) {
      console.error('Failed to load actions:', e);
    }
    setLoading(false);
  };

  return (
    <div className="space-y-6">
      <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-display font-bold flex items-center gap-2">
            <BarChart3 className="w-5 h-5 text-mc-green" />
            玩家行为分析
          </h2>
          <div className="flex gap-2">
            <input
              type="text"
              value={playerFilter}
              onChange={(e) => setPlayerFilter(e.target.value)}
              placeholder="筛选玩家"
              className="input"
            />
            <button onClick={loadActions} className="btn btn-secondary">
              <Search className="w-4 h-4" />
            </button>
          </div>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
        ) : (
          <div className="space-y-2">
            {actions.map((action) => (
              <div
                key={action.id}
                className="flex items-center justify-between p-3 bg-nether-700 rounded-lg"
              >
                <div className="flex items-center gap-3">
                  <span className={clsx(
                    'w-2 h-2 rounded-full',
                    action.action_type === 'Join' && 'bg-mc-green',
                    action.action_type === 'Leave' && 'bg-gray-400',
                    action.action_type === 'Chat' && 'bg-blue-400',
                    action.action_type === 'Death' && 'bg-red-400',
                    action.action_type === 'Kill' && 'bg-orange-400',
                  )} />
                  <span className="text-text-primary font-mono">{action.player_name}</span>
                  <span className="text-text-muted text-sm">{action.details}</span>
                </div>
                <span className="text-text-muted text-xs">
                  {new Date(action.timestamp).toLocaleString('zh-CN')}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function BackupTab() {
  const [backups, setBackups] = useState<PlayerBackup[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedPlayer, setSelectedPlayer] = useState('');
  const [backupType, setBackupType] = useState<'Full' | 'Inventory' | 'Stats' | 'Location'>('Full');

  useEffect(() => {
    if (selectedPlayer) {
      loadBackups();
    }
  }, [selectedPlayer]);

  const loadBackups = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getBackups(selectedPlayer);
      setBackups(data);
    } catch (e) {
      console.error('Failed to load backups:', e);
    }
    setLoading(false);
  };

  const handleCreateBackup = async () => {
    if (!selectedPlayer) return;
    try {
      await playerApi.createBackup(selectedPlayer, backupType);
      loadBackups();
    } catch (e) {
      console.error('Failed to create backup:', e);
    }
  };

  const handleRestore = async (backupId: string) => {
    if (!selectedPlayer) return;
    try {
      await playerApi.restoreBackup(selectedPlayer, backupId);
      alert('备份已恢复');
    } catch (e) {
      console.error('Failed to restore backup:', e);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
        <h2 className="text-lg font-display font-bold mb-6 flex items-center gap-2">
          <Database className="w-5 h-5 text-mc-green" />
          玩家数据备份
        </h2>

        <div className="flex gap-4 mb-6">
          <input
            type="text"
            value={selectedPlayer}
            onChange={(e) => setSelectedPlayer(e.target.value)}
            placeholder="玩家名称"
            className="input flex-1"
          />
          <select
            value={backupType}
            onChange={(e) => setBackupType(e.target.value as any)}
            className="input"
          >
            <option value="Full">完整备份</option>
            <option value="Inventory">背包数据</option>
            <option value="Stats">统计信息</option>
            <option value="Location">位置数据</option>
          </select>
          <button onClick={handleCreateBackup} className="btn btn-primary">
            <Plus className="w-4 h-4 mr-2" />
            创建备份
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
        ) : (
          <div className="space-y-3">
            {backups.map((backup) => (
              <div
                key={backup.id}
                className="flex items-center justify-between p-4 bg-nether-700 rounded-lg border border-nether-600"
              >
                <div className="flex items-center gap-4">
                  <Database className="w-5 h-5 text-mc-green" />
                  <div>
                    <p className="text-text-primary font-mono">{backup.player_name}</p>
                    <p className="text-text-muted text-sm">
                      类型: {backup.backup_type} | 大小: {(backup.file_size / 1024).toFixed(2)} KB
                    </p>
                    <p className="text-text-muted text-xs">
                      {new Date(backup.created_at).toLocaleString('zh-CN')}
                    </p>
                  </div>
                </div>
                <button onClick={() => handleRestore(backup.id)} className="btn btn-secondary">
                  恢复
                </button>
              </div>
            ))}
            {backups.length === 0 && selectedPlayer && (
              <p className="text-text-muted text-center py-8">暂无备份记录</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function PermissionsTab() {
  const [groups, setGroups] = useState<PermissionGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedGroup, setSelectedGroup] = useState<PermissionGroup | null>(null);
  const [newPermission, setNewPermission] = useState('');

  useEffect(() => {
    loadGroups();
  }, []);

  const loadGroups = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getPermissionGroups();
      setGroups(data);
    } catch (e) {
      console.error('Failed to load groups:', e);
    }
    setLoading(false);
  };

  const handleAddPermission = async () => {
    if (!selectedGroup || !newPermission) return;
    try {
      await playerApi.addPermission(selectedGroup.id, newPermission);
      setNewPermission('');
      loadGroups();
    } catch (e) {
      console.error('Failed to add permission:', e);
    }
  };

  const handleRemovePermission = async (permission: string) => {
    if (!selectedGroup) return;
    try {
      await playerApi.removePermission(selectedGroup.id, permission);
      loadGroups();
    } catch (e) {
      console.error('Failed to remove permission:', e);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-display font-bold flex items-center gap-2">
            <UserCog className="w-5 h-5 text-mc-green" />
            权限组可视化编辑
          </h2>
          <button onClick={() => setShowCreateModal(true)} className="btn btn-primary">
            <Plus className="w-4 h-4 mr-2" />
            创建组
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {groups.map((group) => (
              <div
                key={group.id}
                className={clsx(
                  'bg-nether-700 rounded-lg border p-4 cursor-pointer transition-all',
                  selectedGroup?.id === group.id ? 'border-mc-green' : 'border-nether-600'
                )}
                onClick={() => setSelectedGroup(group)}
              >
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-3">
                    <div
                      className="w-8 h-8 rounded"
                      style={{ backgroundColor: group.color + '40', borderColor: group.color }}
                    >
                      <span className="flex items-center justify-center h-full font-bold" style={{ color: group.color }}>
                        {group.name[0].toUpperCase()}
                      </span>
                    </div>
                    <div>
                      <p className="text-text-primary font-bold">
                        {group.prefix && <span>{group.prefix} </span>}
                        {group.display_name}
                      </p>
                      <p className="text-text-muted text-xs">权重: {group.weight}</p>
                    </div>
                  </div>
                  <ChevronRight className="w-5 h-5 text-text-muted" />
                </div>
                <div className="flex flex-wrap gap-1">
                  {group.permissions.slice(0, 5).map((perm) => (
                    <span
                      key={perm}
                      className="px-2 py-0.5 bg-nether-600 rounded text-xs font-mono text-text-secondary"
                    >
                      {perm}
                    </span>
                  ))}
                  {group.permissions.length > 5 && (
                    <span className="px-2 py-0.5 text-text-muted text-xs">
                      +{group.permissions.length - 5} 更多
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {selectedGroup && (
        <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
          <h3 className="text-lg font-display font-bold mb-4">权限详情: {selectedGroup.display_name}</h3>
          <div className="flex gap-2 mb-4">
            <input
              type="text"
              value={newPermission}
              onChange={(e) => setNewPermission(e.target.value)}
              placeholder="添加权限节点"
              className="input flex-1"
            />
            <button onClick={handleAddPermission} className="btn btn-primary">
              <Plus className="w-4 h-4" />
            </button>
          </div>
          <div className="flex flex-wrap gap-2">
            {selectedGroup.permissions.map((perm) => (
              <span
                key={perm}
                className="flex items-center gap-1 px-3 py-1 bg-nether-700 rounded text-sm font-mono"
              >
                {perm}
                <button onClick={() => handleRemovePermission(perm)} className="text-red-400 hover:text-red-300">
                  <X className="w-4 h-4" />
                </button>
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function ChatTab() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchKeyword, setSearchKeyword] = useState('');
  const [searchPlayer, setSearchPlayer] = useState('');

  useEffect(() => {
    loadMessages();
  }, []);

  const loadMessages = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getChatHistory(undefined, 100);
      setMessages(data);
    } catch (e) {
      console.error('Failed to load messages:', e);
    }
    setLoading(false);
  };

  const handleSearch = async () => {
    setLoading(true);
    try {
      const data = await playerApi.searchChat(searchKeyword || undefined, searchPlayer || undefined, 100);
      setMessages(data);
    } catch (e) {
      console.error('Failed to search:', e);
    }
    setLoading(false);
  };

  return (
    <div className="space-y-6">
      <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
        <h2 className="text-lg font-display font-bold mb-6 flex items-center gap-2">
          <MessageSquare className="w-5 h-5 text-mc-green" />
          聊天记录检索
        </h2>

        <div className="flex gap-4 mb-6">
          <input
            type="text"
            value={searchKeyword}
            onChange={(e) => setSearchKeyword(e.target.value)}
            placeholder="搜索关键词"
            className="input flex-1"
          />
          <input
            type="text"
            value={searchPlayer}
            onChange={(e) => setSearchPlayer(e.target.value)}
            placeholder="玩家名称"
            className="input"
          />
          <button onClick={handleSearch} className="btn btn-primary">
            <Search className="w-4 h-4" />
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
        ) : (
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {messages.map((msg) => (
              <div key={msg.id} className="flex gap-3 p-3 bg-nether-700 rounded-lg">
                <div className="w-8 h-8 bg-blue-500/20 rounded-full flex items-center justify-center">
                  <span className="text-blue-400 font-bold text-sm">{msg.player_name[0].toUpperCase()}</span>
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-text-primary font-mono font-bold">{msg.player_name}</span>
                    <span className="text-text-muted text-xs">
                      {new Date(msg.timestamp).toLocaleString('zh-CN')}
                    </span>
                  </div>
                  <p className="text-text-secondary">{msg.message}</p>
                </div>
              </div>
            ))}
            {messages.length === 0 && (
              <p className="text-text-muted text-center py-8">暂无聊天记录</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function WarningsTab() {
  const [warnings, setWarnings] = useState<Warning[]>([]);
  const [loading, setLoading] = useState(true);
  const [showWarningModal, setShowWarningModal] = useState(false);
  const [warningInfo, setWarningInfo] = useState({ playerName: '', reason: '' });

  useEffect(() => {
    loadWarnings();
  }, []);

  const loadWarnings = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getWarnings(undefined, true);
      setWarnings(data);
    } catch (e) {
      console.error('Failed to load warnings:', e);
    }
    setLoading(false);
  };

  const handleIssueWarning = async () => {
    if (!warningInfo.playerName || !warningInfo.reason) return;
    try {
      await playerApi.issueWarning(warningInfo.playerName, warningInfo.reason);
      setShowWarningModal(false);
      setWarningInfo({ playerName: '', reason: '' });
      loadWarnings();
    } catch (e) {
      console.error('Failed to issue warning:', e);
    }
  };

  const handleRevokeWarning = async (warningId: string) => {
    try {
      await playerApi.revokeWarning(warningId);
      loadWarnings();
    } catch (e) {
      console.error('Failed to revoke warning:', e);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-display font-bold flex items-center gap-2">
            <AlertTriangle className="w-5 h-5 text-yellow-400" />
            玩家警告系统
          </h2>
          <button onClick={() => setShowWarningModal(true)} className="btn btn-primary">
            <Plus className="w-4 h-4 mr-2" />
            发出警告
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
        ) : (
          <div className="space-y-3">
            {warnings.map((warning) => (
              <div
                key={warning.id}
                className="flex items-center justify-between p-4 bg-nether-700 rounded-lg border border-yellow-500/30"
              >
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 bg-yellow-500/20 rounded-full flex items-center justify-center">
                    <AlertTriangle className="w-5 h-5 text-yellow-400" />
                  </div>
                  <div>
                    <p className="text-text-primary font-mono font-bold">{warning.player_name}</p>
                    <p className="text-text-muted text-sm">{warning.reason}</p>
                    <p className="text-text-muted text-xs">
                      警告者: {warning.issued_by} | {new Date(warning.issued_at).toLocaleString('zh-CN')}
                    </p>
                  </div>
                </div>
                <button
                  onClick={() => handleRevokeWarning(warning.id)}
                  className="btn btn-secondary"
                >
                  撤销
                </button>
              </div>
            ))}
            {warnings.length === 0 && (
              <p className="text-text-muted text-center py-8">暂无警告记录</p>
            )}
          </div>
        )}
      </div>

      {showWarningModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-nether-800 rounded-lg border border-nether-600 p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-display font-bold">发出警告</h3>
              <button onClick={() => setShowWarningModal(false)}>
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <input
                type="text"
                value={warningInfo.playerName}
                onChange={(e) => setWarningInfo({ ...warningInfo, playerName: e.target.value })}
                placeholder="玩家名称"
                className="input w-full"
              />
              <textarea
                value={warningInfo.reason}
                onChange={(e) => setWarningInfo({ ...warningInfo, reason: e.target.value })}
                placeholder="警告原因"
                className="input w-full"
                rows={3}
              />
            </div>
            <div className="flex gap-2 justify-end mt-6">
              <button onClick={() => setShowWarningModal(false)} className="btn btn-secondary">
                取消
              </button>
              <button onClick={handleIssueWarning} className="btn btn-primary">
                确认
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function EconomyTab() {
  const [economy, setEconomy] = useState<VirtualEconomy | null>(null);
  const [leaderboard, setLeaderboard] = useState<[string, number][]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedPlayer, setSelectedPlayer] = useState('');
  const [showMoneyModal, setShowMoneyModal] = useState(false);
  const [moneyAction, setMoneyAction] = useState<'give' | 'take'>('give');
  const [moneyInfo, setMoneyInfo] = useState({ amount: '', description: '' });

  useEffect(() => {
    loadLeaderboard();
  }, []);

  const loadLeaderboard = async () => {
    setLoading(true);
    try {
      const data = await playerApi.getEconomyLeaderboard();
      setLeaderboard(data);
    } catch (e) {
      console.error('Failed to load leaderboard:', e);
    }
    setLoading(false);
  };

  const loadEconomy = async () => {
    if (!selectedPlayer) return;
    setLoading(true);
    try {
      const data = await playerApi.getEconomy(selectedPlayer);
      setEconomy(data);
    } catch (e) {
      console.error('Failed to load economy:', e);
    }
    setLoading(false);
  };

  const handleMoneyAction = async () => {
    if (!selectedPlayer || !moneyInfo.amount) return;
    try {
      if (moneyAction === 'give') {
        await playerApi.giveMoney(selectedPlayer, parseFloat(moneyInfo.amount), moneyInfo.description);
      } else {
        await playerApi.takeMoney(selectedPlayer, parseFloat(moneyInfo.amount), moneyInfo.description);
      }
      setShowMoneyModal(false);
      setMoneyInfo({ amount: '', description: '' });
      loadEconomy();
    } catch (e) {
      console.error('Failed to perform money action:', e);
    }
  };

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
          <h2 className="text-lg font-display font-bold mb-6 flex items-center gap-2">
            <Coins className="w-5 h-5 text-yellow-400" />
            经济排行
          </h2>

          {loading ? (
            <div className="flex items-center justify-center h-48"><RefreshCw className="w-8 h-8 animate-spin" /></div>
          ) : (
            <div className="space-y-3">
              {leaderboard.map(([name, balance], index) => (
                <div
                  key={name}
                  className="flex items-center justify-between p-3 bg-nether-700 rounded-lg"
                >
                  <div className="flex items-center gap-3">
                    <span className={clsx(
                      'w-8 h-8 rounded-full flex items-center justify-center font-bold',
                      index === 0 && 'bg-yellow-500/20 text-yellow-400',
                      index === 1 && 'bg-gray-300/20 text-gray-300',
                      index === 2 && 'bg-amber-600/20 text-amber-600',
                      index > 2 && 'bg-nether-600 text-text-muted'
                    )}>
                      {index + 1}
                    </span>
                    <span className="text-text-primary font-mono">{name}</span>
                  </div>
                  <span className="text-yellow-400 font-mono font-bold">
                    {balance.toLocaleString()} 金币
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="bg-nether-800 rounded-lg border border-nether-600 p-6">
          <h2 className="text-lg font-display font-bold mb-6 flex items-center gap-2">
            <Coins className="w-5 h-5 text-yellow-400" />
            玩家经济详情
          </h2>

          <div className="flex gap-2 mb-6">
            <input
              type="text"
              value={selectedPlayer}
              onChange={(e) => setSelectedPlayer(e.target.value)}
              placeholder="输入玩家名称"
              className="input flex-1"
            />
            <button onClick={loadEconomy} className="btn btn-secondary">
              <Search className="w-4 h-4" />
            </button>
          </div>

          {economy && (
            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 bg-nether-700 rounded-lg">
                <span className="text-text-muted">余额</span>
                <span className="text-yellow-400 font-mono text-xl font-bold">
                  {economy.balance.toLocaleString()} {economy.currency}
                </span>
              </div>

              <div className="flex gap-2">
                <button
                  onClick={() => { setMoneyAction('give'); setShowMoneyModal(true); }}
                  className="btn btn-primary flex-1"
                >
                  <Plus className="w-4 h-4 mr-2" />
                  给钱
                </button>
                <button
                  onClick={() => { setMoneyAction('take'); setShowMoneyModal(true); }}
                  className="btn btn-secondary flex-1"
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  取钱
                </button>
              </div>

              <div className="mt-4">
                <h3 className="text-sm font-bold text-text-muted mb-2">最近交易</h3>
                <div className="space-y-2 max-h-40 overflow-y-auto">
                  {economy.transactions.map((tx) => (
                    <div key={tx.id} className="flex items-center justify-between p-2 bg-nether-700 rounded text-sm">
                      <span className="text-text-secondary">{tx.description}</span>
                      <span className={clsx(
                        'font-mono',
                        tx.transaction_type === 'Deposit' || tx.transaction_type === 'Earn' ? 'text-mc-green' :
                        'text-red-400'
                      )}>
                        {tx.transaction_type === 'Deposit' || tx.transaction_type === 'Earn' ? '+' : '-'}
                        {tx.amount.toLocaleString()}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {showMoneyModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-nether-800 rounded-lg border border-nether-600 p-6 w-full max-w-md">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-display font-bold">
                {moneyAction === 'give' ? '给钱' : '取钱'}
              </h3>
              <button onClick={() => setShowMoneyModal(false)}>
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <input
                type="number"
                value={moneyInfo.amount}
                onChange={(e) => setMoneyInfo({ ...moneyInfo, amount: e.target.value })}
                placeholder="金额"
                className="input w-full"
              />
              <input
                type="text"
                value={moneyInfo.description}
                onChange={(e) => setMoneyInfo({ ...moneyInfo, description: e.target.value })}
                placeholder="描述"
                className="input w-full"
              />
            </div>
            <div className="flex gap-2 justify-end mt-6">
              <button onClick={() => setShowMoneyModal(false)} className="btn btn-secondary">
                取消
              </button>
              <button onClick={handleMoneyAction} className="btn btn-primary">
                确认
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

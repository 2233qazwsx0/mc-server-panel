import React, { useState, useEffect } from 'react';
import { Database, Download, Upload, Key, Settings, BarChart3, Archive, RefreshCw, CheckCircle2, XCircle, AlertTriangle, Server, HardDrive } from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';
import { useTranslation } from 'react-i18next';

interface DbStatus {
  db_type: string;
  url: string;
  connected: boolean;
}

interface PlayerStats {
  id: number;
  player_uuid: string;
  player_name: string;
  play_time_seconds: number;
  blocks_placed: number;
  blocks_broken: number;
  deaths: number;
  kills: number;
}

interface EconomyAccount {
  id: number;
  player_uuid: string;
  balance: number;
  total_earned: number;
  total_spent: number;
}

interface Backup {
  id: number;
  backup_name: string;
  backup_path: string;
  backup_type: string;
  file_size_bytes: number;
  checksum: string;
  status: string;
  created_at: string;
}

interface ApiKey {
  id: number;
  key_name: string;
  permissions: string[];
  rate_limit: number;
  is_active: boolean;
  expires_at: string | null;
  last_used: string | null;
  created_at: string;
}

type TabType = 'overview' | 'players' | 'economy' | 'api-keys' | 'backup' | 'optimization' | 'performance' | 'archive' | 'sync';

const DatabaseDashboard: React.FC = () => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<TabType>('overview');
  const [dbStatus, setDbStatus] = useState<DbStatus | null>(null);
  const [players, setPlayers] = useState<PlayerStats[]>([]);
  const [economyAccounts, setEconomyAccounts] = useState<EconomyAccount[]>([]);
  const [backups, setBackups] = useState<Backup[]>([]);
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isOptimizing, setIsOptimizing] = useState(false);

  useEffect(() => {
    fetchDbStatus();
    fetchPlayers();
    fetchEconomyAccounts();
    fetchBackups();
    fetchApiKeys();
  }, []);

  const fetchDbStatus = async () => {
    try {
      const response = await fetch('/api/db/status');
      if (response.ok) {
        const data = await response.json();
        setDbStatus(data);
      }
    } catch (err) {
      setError('Failed to fetch database status');
    }
  };

  const fetchPlayers = async () => {
    try {
      const response = await fetch('/api/db/player-stats');
      if (response.ok) {
        const data = await response.json();
        setPlayers(data);
      }
    } catch (err) {
      console.error('Failed to fetch players:', err);
    }
  };

  const fetchEconomyAccounts = async () => {
    try {
      const response = await fetch('/api/db/economy');
      if (response.ok) {
        const data = await response.json();
        setEconomyAccounts(data);
      }
    } catch (err) {
      console.error('Failed to fetch economy accounts:', err);
    }
  };

  const fetchBackups = async () => {
    try {
      const response = await fetch('/api/db/backup');
      if (response.ok) {
        const data = await response.json();
        setBackups(data);
      }
    } catch (err) {
      console.error('Failed to fetch backups:', err);
    }
  };

  const fetchApiKeys = async () => {
    try {
      const response = await fetch('/api/db/api-keys');
      if (response.ok) {
        const data = await response.json();
        setApiKeys(data);
      }
    } catch (err) {
      console.error('Failed to fetch API keys:', err);
    }
  };

  const handleOptimize = async () => {
    setIsOptimizing(true);
    try {
      const response = await fetch('/api/db/optimize', { method: 'POST' });
      if (response.ok) {
        alert('Database optimization completed successfully!');
      }
    } catch (err) {
      setError('Optimization failed');
    } finally {
      setIsOptimizing(false);
    }
  };

  const handleBackup = async () => {
    try {
      const response = await fetch('/api/db/backup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: null, backup_type: 'Full' }),
      });
      if (response.ok) {
        alert('Backup created successfully!');
        fetchBackups();
      }
    } catch (err) {
      setError('Backup failed');
    }
  };

  const handleExport = async (tableName: string) => {
    try {
      const response = await fetch(`/api/db/export?table_name=${tableName}`);
      if (response.ok) {
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${tableName}_export.csv`;
        a.click();
      }
    } catch (err) {
      setError('Export failed');
    }
  };

  const tabs = [
    { id: 'overview', label: t('Overview'), icon: Database },
    { id: 'players', label: t('Players'), icon: Server },
    { id: 'economy', label: t('Economy'), icon: HardDrive },
    { id: 'api-keys', label: t('API Keys'), icon: Key },
    { id: 'backup', label: t('Backup'), icon: Download },
    { id: 'optimization', label: t('Optimization'), icon: Settings },
    { id: 'performance', label: t('Performance'), icon: BarChart3 },
    { id: 'archive', label: t('Archive'), icon: Archive },
    { id: 'sync', label: t('Sync'), icon: RefreshCw },
  ];

  const totalPlayTime = players.reduce((acc, p) => acc + p.play_time_seconds, 0);
  const totalBalance = economyAccounts.reduce((acc, e) => acc + e.balance, 0);
  const activeApiKeys = apiKeys.filter(k => k.is_active).length;

  const renderOverview = () => (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl p-6 text-white shadow-lg">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-blue-100">{t('Total Players')}</p>
              <p className="text-3xl font-bold mt-2">{players.length}</p>
            </div>
            <Server className="w-12 h-12 opacity-50" />
          </div>
        </div>

        <div className="bg-gradient-to-br from-green-500 to-green-600 rounded-xl p-6 text-white shadow-lg">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-green-100">{t('Total Balance')}</p>
              <p className="text-3xl font-bold mt-2">{totalBalance.toFixed(2)}</p>
            </div>
            <HardDrive className="w-12 h-12 opacity-50" />
          </div>
        </div>

        <div className="bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl p-6 text-white shadow-lg">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-purple-100">{t('Play Time (hours)')}</p>
              <p className="text-3xl font-bold mt-2">{Math.round(totalPlayTime / 3600)}</p>
            </div>
            <RefreshCw className="w-12 h-12 opacity-50" />
          </div>
        </div>

        <div className="bg-gradient-to-br from-orange-500 to-orange-600 rounded-xl p-6 text-white shadow-lg">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-orange-100">{t('Active API Keys')}</p>
              <p className="text-3xl font-bold mt-2">{activeApiKeys}</p>
            </div>
            <Key className="w-12 h-12 opacity-50" />
          </div>
        </div>
      </div>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl p-6 shadow-lg">
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Database className="w-5 h-5" />
          {t('Database Status')}
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="flex items-center gap-3">
            {dbStatus?.connected ? (
              <CheckCircle2 className="w-6 h-6 text-green-500" />
            ) : (
              <XCircle className="w-6 h-6 text-red-500" />
            )}
            <div>
              <p className="text-sm text-gray-500">{t('Connection Status')}</p>
              <p className="font-medium">{dbStatus?.connected ? t('Connected') : t('Disconnected')}</p>
            </div>
          </div>
          <div>
            <p className="text-sm text-gray-500">{t('Database Type')}</p>
            <p className="font-medium">{dbStatus?.db_type || 'Unknown'}</p>
          </div>
          <div>
            <p className="text-sm text-gray-500">{t('URL')}</p>
            <p className="font-medium text-sm">{dbStatus?.url || 'N/A'}</p>
          </div>
        </div>
      </div>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl p-6 shadow-lg">
        <h3 className="text-lg font-semibold mb-4">{t('Recent Backups')}</h3>
        <div className="space-y-3">
          {backups.slice(0, 5).map((backup) => (
            <div key={backup.id} className="flex items-center justify-between p-3 bg-dark-200 dark:bg-dark-700 rounded-lg">
              <div className="flex items-center gap-3">
                <Download className="w-5 h-5 text-blue-500" />
                <div>
                  <p className="font-medium">{backup.backup_name}</p>
                  <p className="text-sm text-gray-500">{new Date(backup.created_at).toLocaleString()}</p>
                </div>
              </div>
              <span className="text-sm text-gray-500">{(backup.file_size_bytes / 1024 / 1024).toFixed(2)} MB</span>
            </div>
          ))}
          {backups.length === 0 && (
            <p className="text-gray-500 text-center py-4">{t('No backups available')}</p>
          )}
        </div>
      </div>
    </div>
  );

  const renderPlayers = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">{t('Player Statistics')}</h2>
        <button
          onClick={() => handleExport('player_stats')}
          className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
        >
          <Download className="w-4 h-4" />
          {t('Export CSV')}
        </button>
      </div>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl shadow-lg overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-dark-200 dark:bg-dark-700">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{t('Player')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{t('Play Time')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{t('Blocks')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{t('Deaths')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{t('Kills')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-dark-200 dark:divide-dark-700">
              {players.map((player) => (
                <tr key={player.id} className="hover:bg-dark-200 dark:hover:bg-dark-700 transition">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center">
                      <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-500 rounded-full flex items-center justify-center text-white font-bold">
                        {player.player_name.charAt(0).toUpperCase()}
                      </div>
                      <div className="ml-4">
                        <p className="font-medium">{player.player_name}</p>
                        <p className="text-sm text-gray-500">{player.player_uuid.slice(0, 8)}...</p>
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    {Math.round(player.play_time_seconds / 3600)}h {Math.round((player.play_time_seconds % 3600) / 60)}m
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="text-green-500">+{player.blocks_placed}</span> / <span className="text-red-500">-{player.blocks_broken}</span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">{player.deaths}</td>
                  <td className="px-6 py-4 whitespace-nowrap">{player.kills}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );

  const renderEconomy = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">{t('Economy System')}</h2>
        <button
          onClick={() => handleExport('economy')}
          className="flex items-center gap-2 px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition"
        >
          <Download className="w-4 h-4" />
          {t('Export CSV')}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-green-500/10 border border-green-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Total Money')}</p>
          <p className="text-2xl font-bold text-green-500 mt-2">{totalBalance.toFixed(2)}</p>
        </div>
        <div className="bg-blue-500/10 border border-blue-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Total Accounts')}</p>
          <p className="text-2xl font-bold text-blue-500 mt-2">{economyAccounts.length}</p>
        </div>
        <div className="bg-purple-500/10 border border-purple-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Average Balance')}</p>
          <p className="text-2xl font-bold text-purple-500 mt-2">
            {economyAccounts.length > 0 ? (totalBalance / economyAccounts.length).toFixed(2) : '0.00'}
          </p>
        </div>
      </div>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl shadow-lg overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-dark-200 dark:bg-dark-700">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Player')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Balance')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Total Earned')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Total Spent')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-dark-200 dark:divide-dark-700">
              {economyAccounts.map((account) => (
                <tr key={account.id} className="hover:bg-dark-200 dark:hover:bg-dark-700 transition">
                  <td className="px-6 py-4 whitespace-nowrap font-medium">{account.player_uuid.slice(0, 8)}...</td>
                  <td className="px-6 py-4 whitespace-nowrap text-green-500 font-medium">{account.balance.toFixed(2)}</td>
                  <td className="px-6 py-4 whitespace-nowrap text-blue-500">{account.total_earned.toFixed(2)}</td>
                  <td className="px-6 py-4 whitespace-nowrap text-red-500">{account.total_spent.toFixed(2)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );

  const renderApiKeys = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">{t('API Key Management')}</h2>
        <button className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition">
          <Key className="w-4 h-4" />
          {t('Create Key')}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-blue-500/10 border border-blue-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Total Keys')}</p>
          <p className="text-2xl font-bold text-blue-500 mt-2">{apiKeys.length}</p>
        </div>
        <div className="bg-green-500/10 border border-green-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Active Keys')}</p>
          <p className="text-2xl font-bold text-green-500 mt-2">{activeApiKeys}</p>
        </div>
        <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Expired Keys')}</p>
          <p className="text-2xl font-bold text-red-500 mt-2">{apiKeys.length - activeApiKeys}</p>
        </div>
      </div>

      <div className="space-y-3">
        {apiKeys.map((key) => (
          <div key={key.id} className="bg-dark-100 dark:bg-dark-800 rounded-xl p-4 shadow-lg">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Key className={`w-5 h-5 ${key.is_active ? 'text-green-500' : 'text-red-500'}`} />
                <div>
                  <p className="font-medium">{key.key_name}</p>
                  <p className="text-sm text-gray-500">
                    {t('Created')}: {new Date(key.created_at).toLocaleDateString()}
                    {key.last_used && ` | ${t('Last used')}: ${new Date(key.last_used).toLocaleDateString()}`}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <span className={`px-3 py-1 rounded-full text-xs font-medium ${
                  key.is_active ? 'bg-green-500/20 text-green-500' : 'bg-red-500/20 text-red-500'
                }`}>
                  {key.is_active ? t('Active') : t('Inactive')}
                </span>
                <button className="p-2 hover:bg-dark-200 dark:hover:bg-dark-700 rounded-lg transition">
                  <Settings className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        ))}
        {apiKeys.length === 0 && (
          <p className="text-gray-500 text-center py-8">{t('No API keys available')}</p>
        )}
      </div>
    </div>
  );

  const renderBackup = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">{t('Backup & Restore')}</h2>
        <button
          onClick={handleBackup}
          className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition"
        >
          <Download className="w-4 h-4" />
          {t('Create Backup')}
        </button>
      </div>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl shadow-lg overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-dark-200 dark:bg-dark-700">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Backup Name')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Type')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Size')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Created')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Status')}</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{t('Actions')}</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-dark-200 dark:divide-dark-700">
              {backups.map((backup) => (
                <tr key={backup.id} className="hover:bg-dark-200 dark:hover:bg-dark-700 transition">
                  <td className="px-6 py-4 whitespace-nowrap font-medium">{backup.backup_name}</td>
                  <td className="px-6 py-4 whitespace-nowrap">{backup.backup_type}</td>
                  <td className="px-6 py-4 whitespace-nowrap">{(backup.file_size_bytes / 1024 / 1024).toFixed(2)} MB</td>
                  <td className="px-6 py-4 whitespace-nowrap">{new Date(backup.created_at).toLocaleString()}</td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                      backup.status === 'completed' ? 'bg-green-500/20 text-green-500' : 'bg-yellow-500/20 text-yellow-500'
                    }`}>
                      {backup.status}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex gap-2">
                      <button className="px-3 py-1 bg-green-500/20 text-green-500 rounded-lg hover:bg-green-500/30 transition text-sm">
                        {t('Restore')}
                      </button>
                      <button className="px-3 py-1 bg-red-500/20 text-red-500 rounded-lg hover:bg-red-500/30 transition text-sm">
                        {t('Delete')}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );

  const renderOptimization = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">{t('Database Optimization')}</h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="bg-dark-100 dark:bg-dark-800 rounded-xl p-6 shadow-lg">
          <h3 className="text-lg font-semibold mb-4">{t('Quick Actions')}</h3>
          <div className="space-y-3">
            <button
              onClick={handleOptimize}
              disabled={isOptimizing}
              className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition disabled:opacity-50"
            >
              <Settings className="w-5 h-5" />
              {isOptimizing ? t('Optimizing...') : t('Full Optimization')}
            </button>
            <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-green-500 text-white rounded-lg hover:bg-green-600 transition">
              <RefreshCw className="w-5 h-5" />
              {t('Run VACUUM')}
            </button>
            <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-purple-500 text-white rounded-lg hover:bg-purple-600 transition">
              <BarChart3 className="w-5 h-5" />
              {t('Run ANALYZE')}
            </button>
          </div>
        </div>

        <div className="bg-dark-100 dark:bg-dark-800 rounded-xl p-6 shadow-lg">
          <h3 className="text-lg font-semibold mb-4">{t('Scheduled Tasks')}</h3>
          <div className="space-y-3">
            <div className="flex items-center justify-between p-3 bg-dark-200 dark:bg-dark-700 rounded-lg">
              <div className="flex items-center gap-3">
                <RefreshCw className="w-5 h-5 text-blue-500" />
                <span>{t('Auto VACUUM')}</span>
              </div>
              <select className="px-3 py-1 bg-dark-300 dark:bg-dark-600 rounded-lg">
                <option value="24">{t('Every 24 hours')}</option>
                <option value="48">{t('Every 48 hours')}</option>
                <option value="168">{t('Weekly')}</option>
              </select>
            </div>
            <div className="flex items-center justify-between p-3 bg-dark-200 dark:bg-dark-700 rounded-lg">
              <div className="flex items-center gap-3">
                <BarChart3 className="w-5 h-5 text-green-500" />
                <span>{t('Auto ANALYZE')}</span>
              </div>
              <select className="px-3 py-1 bg-dark-300 dark:bg-dark-600 rounded-lg">
                <option value="6">{t('Every 6 hours')}</option>
                <option value="12">{t('Every 12 hours')}</option>
                <option value="24">{t('Every 24 hours')}</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  const renderPerformance = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">{t('Performance Analysis')}</h2>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-blue-500/10 border border-blue-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Total Queries')}</p>
          <p className="text-2xl font-bold text-blue-500 mt-2">1,234</p>
        </div>
        <div className="bg-green-500/10 border border-green-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Avg Query Time')}</p>
          <p className="text-2xl font-bold text-green-500 mt-2">12ms</p>
        </div>
        <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-6">
          <p className="text-sm text-gray-500">{t('Slow Queries')}</p>
          <p className="text-2xl font-bold text-red-500 mt-2">5</p>
        </div>
      </div>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl p-6 shadow-lg">
        <h3 className="text-lg font-semibold mb-4">{t('Query Performance Over Time')}</h3>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={[
            { time: '00:00', queries: 45, avgTime: 8 },
            { time: '04:00', queries: 32, avgTime: 12 },
            { time: '08:00', queries: 78, avgTime: 15 },
            { time: '12:00', queries: 95, avgTime: 10 },
            { time: '16:00', queries: 88, avgTime: 14 },
            { time: '20:00', queries: 65, avgTime: 11 },
          ]}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="time" />
            <YAxis />
            <Tooltip />
            <Line type="monotone" dataKey="queries" stroke="#3B82F6" strokeWidth={2} />
            <Line type="monotone" dataKey="avgTime" stroke="#10B981" strokeWidth={2} />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );

  const renderArchive = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">{t('Data Archiving')}</h2>
        <button className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition">
          <Archive className="w-4 h-4" />
          {t('Create Archive')}
        </button>
      </div>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl p-6 shadow-lg">
        <h3 className="text-lg font-semibold mb-4">{t('Archive Policies')}</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-dark-200 dark:bg-dark-700 rounded-lg">
            <div className="flex items-center gap-3">
              <Archive className="w-6 h-6 text-blue-500" />
              <div>
                <p className="font-medium">{t('Transactions Archive')}</p>
                <p className="text-sm text-gray-500">{t('Archive transactions older than 90 days')}</p>
              </div>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" className="sr-only peer" defaultChecked />
              <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-500"></div>
            </label>
          </div>

          <div className="flex items-center justify-between p-4 bg-dark-200 dark:bg-dark-700 rounded-lg">
            <div className="flex items-center gap-3">
              <Archive className="w-6 h-6 text-purple-500" />
              <div>
                <p className="font-medium">{t('Inactive Players Archive')}</p>
                <p className="text-sm text-gray-500">{t('Archive players inactive for 180 days')}</p>
              </div>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" className="sr-only peer" defaultChecked />
              <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-500"></div>
            </label>
          </div>
        </div>
      </div>
    </div>
  );

  const renderSync = () => (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">{t('Real-time Data Sync')}</h2>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl p-6 shadow-lg">
        <h3 className="text-lg font-semibold mb-4">{t('Sync Targets')}</h3>
        <div className="space-y-3">
          <div className="flex items-center justify-between p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
            <div className="flex items-center gap-3">
              <RefreshCw className="w-6 h-6 text-green-500" />
              <div>
                <p className="font-medium">{t('Player Stats Sync')}</p>
                <p className="text-sm text-gray-500">{t('Last synced')}: 2 minutes ago</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-5 h-5 text-green-500" />
              <span className="text-green-500">{t('Active')}</span>
            </div>
          </div>

          <div className="flex items-center justify-between p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
            <div className="flex items-center gap-3">
              <RefreshCw className="w-6 h-6 text-yellow-500" />
              <div>
                <p className="font-medium">{t('Economy Sync')}</p>
                <p className="text-sm text-gray-500">{t('Last synced')}: 15 minutes ago</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-yellow-500" />
              <span className="text-yellow-500">{t('Pending')}</span>
            </div>
          </div>
        </div>
      </div>

      <div className="bg-dark-100 dark:bg-dark-800 rounded-xl p-6 shadow-lg">
        <h3 className="text-lg font-semibold mb-4">{t('Sync History')}</h3>
        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 bg-dark-200 dark:bg-dark-700 rounded-lg">
            <div className="flex items-center gap-3">
              <CheckCircle2 className="w-5 h-5 text-green-500" />
              <span>Player Stats → External API</span>
            </div>
            <span className="text-sm text-gray-500">2 min ago</span>
          </div>
          <div className="flex items-center justify-between p-3 bg-dark-200 dark:bg-dark-700 rounded-lg">
            <div className="flex items-center gap-3">
              <CheckCircle2 className="w-5 h-5 text-green-500" />
              <span>Economy → Webhook</span>
            </div>
            <span className="text-sm text-gray-500">15 min ago</span>
          </div>
        </div>
      </div>
    </div>
  );

  const renderContent = () => {
    switch (activeTab) {
      case 'overview': return renderOverview();
      case 'players': return renderPlayers();
      case 'economy': return renderEconomy();
      case 'api-keys': return renderApiKeys();
      case 'backup': return renderBackup();
      case 'optimization': return renderOptimization();
      case 'performance': return renderPerformance();
      case 'archive': return renderArchive();
      case 'sync': return renderSync();
      default: return renderOverview();
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-dark-900 p-6">
      <div className="max-w-7xl mx-auto">
        <div className="mb-6">
          <h1 className="text-3xl font-bold flex items-center gap-3">
            <Database className="w-8 h-8" />
            {t('Database Management')}
          </h1>
          <p className="text-gray-500 mt-2">{t('Manage your Minecraft server database, backups, and data')}</p>
        </div>

        <div className="flex gap-2 mb-6 overflow-x-auto pb-2">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as TabType)}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg whitespace-nowrap transition ${
                  activeTab === tab.id
                    ? 'bg-blue-500 text-white shadow-lg'
                    : 'bg-white dark:bg-dark-800 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-dark-700'
                }`}
              >
                <Icon className="w-4 h-4" />
                {tab.label}
              </button>
            );
          })}
        </div>

        {error && (
          <div className="mb-4 p-4 bg-red-500/10 border border-red-500/20 rounded-xl text-red-500">
            {error}
          </div>
        )}

        {renderContent()}
      </div>
    </div>
  );
};

export default DatabaseDashboard;

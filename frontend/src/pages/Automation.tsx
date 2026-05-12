import { useState, useEffect } from 'react';
import {
  Clock,
  Database,
  FileTrash,
  RefreshCw,
  Cpu,
  Download,
  HardDrive,
  TestTube,
  GitBranch,
  ArrowRightLeft,
  Play,
  Pause,
  Trash2,
  RotateCcw,
  Check,
  AlertTriangle,
  ChevronRight,
  Loader2,
} from 'lucide-react';
import { clsx } from 'clsx';

interface TaskStatus {
  id: string;
  name: string;
  task_type: string;
  enabled: boolean;
  last_run: string | null;
  next_run: string | null;
  schedule: string;
  last_result?: {
    success: boolean;
    message: string;
    duration_ms: number;
    timestamp: string;
  };
}

interface BackupInfo {
  id: string;
  name: string;
  path: string;
  size_bytes: number;
  created_at: string;
  world_count: number;
  config_count: number;
}

interface DiskInfo {
  path: string;
  total_bytes: number;
  used_bytes: number;
  available_bytes: number;
  usage_percent: number;
}

interface DiskAlert {
  id: string;
  path: string;
  level: string;
  usage_percent: number;
  available_bytes: number;
  timestamp: string;
  acknowledged: boolean;
}

interface VersionInfo {
  current_version: string;
  latest_version: string | null;
  update_available: boolean;
  release_date: string | null;
  download_url: string | null;
}

interface TestSuite {
  id: string;
  name: string;
  total_tests: number;
  passed_tests: number;
  failed_tests: number;
  last_run: string | null;
}

interface ConfigVersion {
  id: string;
  version: string;
  created_at: string;
  description: string;
}

interface MigrationPlan {
  id: string;
  source_path: string;
  target_path: string;
  status: string;
  estimated_size: number;
  steps: {
    id: number;
    description: string;
    status: string;
    progress_percent: number;
  }[];
}

interface AutomationSummary {
  task_statuses: TaskStatus[];
  backup_count: number;
  pending_migrations: number;
  config_versions: number;
  disk_alerts: number;
}

type TabType = 'overview' | 'backup' | 'cleanup' | 'restart' | 'cron' | 'warmup' | 'updates' | 'disk' | 'tests' | 'versions' | 'migration';

const API_BASE = '/api';

async function fetchApi<T>(endpoint: string): Promise<T | null> {
  try {
    const response = await fetch(`${API_BASE}${endpoint}`);
    const data = await response.json();
    return data.success ? data.data : null;
  } catch (error) {
    console.error(`API Error [${endpoint}]:`, error);
    return null;
  }
}

async function postApi<T>(endpoint: string, body?: object): Promise<T | null> {
  try {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: body ? JSON.stringify(body) : undefined,
    });
    const data = await response.json();
    return data.success ? data.data : null;
  } catch (error) {
    console.error(`API Error [${endpoint}]:`, error);
    return null;
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function formatDate(dateStr: string | null): string {
  if (!dateStr) return '从未';
  const date = new Date(dateStr);
  return date.toLocaleString('zh-CN');
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.round((ms % 60000) / 1000)}s`;
}

export function Automation() {
  const [activeTab, setActiveTab] = useState<TabType>('overview');
  const [summary, setSummary] = useState<AutomationSummary | null>(null);
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [diskInfo, setDiskInfo] = useState<DiskInfo[]>([]);
  const [diskAlerts, setDiskAlerts] = useState<DiskAlert[]>([]);
  const [versionInfo, setVersionInfo] = useState<VersionInfo | null>(null);
  const [testSuites, setTestSuites] = useState<TestSuite[]>([]);
  const [configVersions, setConfigVersions] = useState<ConfigVersion[]>([]);
  const [migrations, setMigrations] = useState<MigrationPlan[]>([]);
  const [loading, setLoading] = useState(false);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    const [summaryData, backupData, diskData, alertsData, versionData, suitesData, versionsData, migrationData] =
      await Promise.all([
        fetchApi<AutomationSummary>('/automation/summary'),
        fetchApi<BackupInfo[]>('/automation/backup/list'),
        fetchApi<DiskInfo[]>('/automation/disk'),
        fetchApi<DiskAlert[]>('/automation/disk/alerts'),
        fetchApi<VersionInfo>('/automation/updates/cached'),
        fetchApi<TestSuite[]>('/automation/tests/suites'),
        fetchApi<ConfigVersion[]>('/automation/config-versions'),
        fetchApi<MigrationPlan[]>('/automation/migrations'),
      ]);

    setSummary(summaryData);
    setBackups(backupData || []);
    setDiskInfo(diskData || []);
    setDiskAlerts(alertsData || []);
    setVersionInfo(versionData);
    setTestSuites(suitesData || []);
    setConfigVersions(versionsData || []);
    setMigrations(migrationData || []);
    setLoading(false);
  };

  const handleBackup = async () => {
    setActionLoading('backup');
    await postApi('/automation/backup/create', { server_path: '.' });
    await loadData();
    setActionLoading(null);
  };

  const handleDeleteBackup = async (id: string) => {
    if (confirm('确定要删除此备份吗？')) {
      setActionLoading(`delete-${id}`);
      await fetch(`${API_BASE}/automation/backup/${id}`, { method: 'DELETE' });
      await loadData();
      setActionLoading(null);
    }
  };

  const handleLogCleanup = async () => {
    setActionLoading('cleanup');
    await postApi('/automation/log-cleanup', { server_path: '.' });
    await loadData();
    setActionLoading(null);
  };

  const handleCheckUpdates = async () => {
    setActionLoading('update-check');
    await postApi('/automation/updates');
    await loadData();
    setActionLoading(null);
  };

  const handleRunTests = async () => {
    setActionLoading('tests');
    await postApi('/automation/tests');
    await loadData();
    setActionLoading(null);
  };

  const handleAcknowledgeAlert = async (id: string) => {
    await fetch(`${API_BASE}/automation/disk/alerts/${id}`, { method: 'PUT' });
    await loadData();
  };

  const tabs: { id: TabType; label: string; icon: React.ReactNode }[] = [
    { id: 'overview', label: '概览', icon: <Clock className="w-4 h-4" /> },
    { id: 'backup', label: '备份管理', icon: <Database className="w-4 h-4" /> },
    { id: 'cleanup', label: '日志清理', icon: <FileTrash className="w-4 h-4" /> },
    { id: 'restart', label: '自动重启', icon: <RefreshCw className="w-4 h-4" /> },
    { id: 'cron', label: 'Cron任务', icon: <Clock className="w-4 h-4" /> },
    { id: 'warmup', label: '预热脚本', icon: <Cpu className="w-4 h-4" /> },
    { id: 'updates', label: '版本更新', icon: <Download className="w-4 h-4" /> },
    { id: 'disk', label: '磁盘监控', icon: <HardDrive className="w-4 h-4" /> },
    { id: 'tests', label: '测试套件', icon: <TestTube className="w-4 h-4" /> },
    { id: 'versions', label: '配置版本', icon: <GitBranch className="w-4 h-4" /> },
    { id: 'migration', label: '数据迁移', icon: <ArrowRightLeft className="w-4 h-4" /> },
  ];

  return (
    <div className="space-y-4 md:space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="font-display text-2xl md:text-3xl font-bold text-text-primary">自动化运维</h1>
          <p className="text-text-secondary font-mono mt-1">定时任务与自动化管理</p>
        </div>
        <button
          onClick={loadData}
          className="game-button flex items-center gap-2"
          disabled={loading}
        >
          <RefreshCw className={clsx('w-4 h-4', loading && 'animate-spin')} />
          刷新数据
        </button>
      </div>

      <div className="flex overflow-x-auto pb-2 gap-2 scrollbar-thin">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={clsx(
              'flex items-center gap-2 px-4 py-2 rounded-lg whitespace-nowrap transition-all duration-200',
              activeTab === tab.id
                ? 'bg-mc-green text-nether-900 font-semibold'
                : 'bg-nether-800 text-text-secondary hover:text-text-primary hover:bg-nether-700'
            )}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="w-8 h-8 animate-spin text-mc-green" />
        </div>
      ) : (
        <>
          {activeTab === 'overview' && <OverviewTab summary={summary} />}
          {activeTab === 'backup' && (
            <BackupTab
              backups={backups}
              onBackup={handleBackup}
              onDelete={handleDeleteBackup}
              isLoading={actionLoading === 'backup'}
            />
          )}
          {activeTab === 'cleanup' && (
            <CleanupTab onCleanup={handleLogCleanup} isLoading={actionLoading === 'cleanup'} />
          )}
          {activeTab === 'restart' && <RestartTab summary={summary} />}
          {activeTab === 'cron' && <CronTab summary={summary} />}
          {activeTab === 'warmup' && <WarmupTab />}
          {activeTab === 'updates' && (
            <UpdatesTab
              versionInfo={versionInfo}
              onCheck={handleCheckUpdates}
              isLoading={actionLoading === 'update-check'}
            />
          )}
          {activeTab === 'disk' && (
            <DiskTab
              diskInfo={diskInfo}
              alerts={diskAlerts}
              onAcknowledge={handleAcknowledgeAlert}
            />
          )}
          {activeTab === 'tests' && (
            <TestsTab
              suites={testSuites}
              onRunTests={handleRunTests}
              isLoading={actionLoading === 'tests'}
            />
          )}
          {activeTab === 'versions' && <VersionsTab versions={configVersions} />}
          {activeTab === 'migration' && <MigrationTab plans={migrations} />}
        </>
      )}
    </div>
  );
}

function OverviewTab({ summary }: { summary: AutomationSummary | null }) {
  if (!summary) return null;

  const stats = [
    { label: '活跃任务', value: summary.task_statuses.filter(t => t.enabled).length, color: 'text-mc-green' },
    { label: '备份数量', value: summary.backup_count, color: 'text-blue-400' },
    { label: '待处理迁移', value: summary.pending_migrations, color: 'text-yellow-400' },
    { label: '配置版本', value: summary.config_versions, color: 'text-purple-400' },
    { label: '磁盘预警', value: summary.disk_alerts, color: summary.disk_alerts > 0 ? 'text-red-400' : 'text-text-secondary' },
  ];

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
        {stats.map((stat) => (
          <div key={stat.label} className="game-card">
            <p className="text-text-secondary text-sm">{stat.label}</p>
            <p className={clsx('text-3xl font-bold font-mono mt-1', stat.color)}>
              {stat.value}
            </p>
          </div>
        ))}
      </div>

      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">任务状态总览</h3>
        <div className="space-y-3">
          {summary.task_statuses.map((task) => (
            <div key={task.id} className="flex items-center justify-between p-3 bg-nether-800 rounded-lg">
              <div className="flex items-center gap-3">
                <div className={clsx(
                  'w-2 h-2 rounded-full',
                  task.enabled ? 'bg-mc-green' : 'bg-gray-500'
                )} />
                <div>
                  <p className="font-medium">{task.name}</p>
                  <p className="text-xs text-text-secondary font-mono">
                    {task.schedule}
                  </p>
                </div>
              </div>
              <div className="text-right">
                <p className="text-sm text-text-secondary">
                  上次运行: {formatDate(task.last_run)}
                </p>
                {task.last_result && (
                  <p className={clsx(
                    'text-xs font-mono',
                    task.last_result.success ? 'text-mc-green' : 'text-red-400'
                  )}>
                    {task.last_result.success ? '成功' : '失败'} ({formatDuration(task.last_result.duration_ms)})
                  </p>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function BackupTab({
  backups,
  onBackup,
  onDelete,
  isLoading,
}: {
  backups: BackupInfo[];
  onBackup: () => void;
  onDelete: (id: string) => void;
  isLoading: boolean;
}) {
  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="font-display text-lg font-semibold">备份列表</h3>
        <button
          onClick={onBackup}
          className="game-button game-button-primary flex items-center gap-2"
          disabled={isLoading}
        >
          {isLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Database className="w-4 h-4" />}
          创建备份
        </button>
      </div>

      {backups.length === 0 ? (
        <div className="game-card text-center py-8">
          <Database className="w-12 h-12 mx-auto text-text-muted mb-4" />
          <p className="text-text-secondary">暂无备份记录</p>
          <p className="text-sm text-text-muted mt-1">点击上方按钮创建第一个备份</p>
        </div>
      ) : (
        <div className="space-y-3">
          {backups.map((backup) => (
            <div key={backup.id} className="game-card">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 bg-blue-500/20 rounded-lg flex items-center justify-center">
                    <Database className="w-5 h-5 text-blue-400" />
                  </div>
                  <div>
                    <p className="font-medium font-mono">{backup.name}</p>
                    <p className="text-sm text-text-secondary">
                      {formatDate(backup.created_at)} · {formatBytes(backup.size_bytes)}
                    </p>
                    <p className="text-xs text-text-muted">
                      世界: {backup.world_count} · 配置: {backup.config_count}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <button className="p-2 hover:bg-nether-700 rounded-lg transition-colors" title="恢复备份">
                    <RotateCcw className="w-4 h-4 text-text-secondary" />
                  </button>
                  <button
                    onClick={() => onDelete(backup.id)}
                    className="p-2 hover:bg-red-500/20 rounded-lg transition-colors"
                    title="删除备份"
                  >
                    <Trash2 className="w-4 h-4 text-red-400" />
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function CleanupTab({ onCleanup, isLoading }: { onCleanup: () => void; isLoading: boolean }) {
  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">日志自动清理</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-nether-800 rounded-lg">
            <div>
              <p className="font-medium">执行日志清理</p>
              <p className="text-sm text-text-secondary">
                清理超过14天的日志文件，释放磁盘空间
              </p>
            </div>
            <button
              onClick={onCleanup}
              className="game-button flex items-center gap-2"
              disabled={isLoading}
            >
              {isLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : <FileTrash className="w-4 h-4" />}
              立即清理
            </button>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="p-4 bg-nether-800 rounded-lg">
              <p className="text-text-secondary text-sm">保留天数</p>
              <p className="text-2xl font-bold font-mono">14天</p>
            </div>
            <div className="p-4 bg-nether-800 rounded-lg">
              <p className="text-text-secondary text-sm">最大单个文件</p>
              <p className="text-2xl font-bold font-mono">100MB</p>
            </div>
            <div className="p-4 bg-nether-800 rounded-lg">
              <p className="text-text-secondary text-sm">清理模式</p>
              <p className="text-2xl font-bold font-mono">自动</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function RestartTab({ summary }: { summary: AutomationSummary | null }) {
  const restartTask = summary?.task_statuses.find(t => t.id === 'restart_strategy');

  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">自动重启策略</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-nether-800 rounded-lg">
            <div>
              <p className="font-medium">智能重启</p>
              <p className="text-sm text-text-secondary">
                根据服务器状态自动触发重启
              </p>
            </div>
            <div className={clsx(
              'px-3 py-1 rounded-full text-sm font-medium',
              restartTask?.enabled ? 'bg-green-500/20 text-green-400' : 'bg-gray-500/20 text-gray-400'
            )}>
              {restartTask?.enabled ? '已启用' : '已禁用'}
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="p-4 bg-nether-800 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Check className="w-4 h-4 text-mc-green" />
                <span className="font-medium">崩溃重启</span>
              </div>
              <p className="text-sm text-text-secondary">服务器崩溃时自动重启</p>
            </div>
            <div className="p-4 bg-nether-800 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Check className="w-4 h-4 text-mc-green" />
                <span className="font-medium">低内存重启</span>
              </div>
              <p className="text-sm text-text-secondary">内存使用超过90%时重启</p>
            </div>
            <div className="p-4 bg-nether-800 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Check className="w-4 h-4 text-mc-green" />
                <span className="font-medium">低TPS重启</span>
              </div>
              <p className="text-sm text-text-secondary">TPS低于15时重启</p>
            </div>
            <div className="p-4 bg-nether-800 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Clock className="w-4 h-4 text-yellow-400" />
                <span className="font-medium">重启冷却</span>
              </div>
              <p className="text-sm text-text-secondary">重启间隔至少5分钟</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function CronTab({ summary }: { summary: AutomationSummary | null }) {
  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">Cron 任务调度</h3>
        <div className="space-y-3">
          {summary?.task_statuses.filter(t => t.schedule !== 'on_condition' && t.schedule !== 'on_start').map((task) => (
            <div key={task.id} className="flex items-center justify-between p-4 bg-nether-800 rounded-lg">
              <div className="flex items-center gap-4">
                <div className={clsx(
                  'w-10 h-10 rounded-lg flex items-center justify-center',
                  task.enabled ? 'bg-mc-green/20' : 'bg-gray-500/20'
                )}>
                  <Clock className={clsx('w-5 h-5', task.enabled ? 'text-mc-green' : 'text-gray-400')} />
                </div>
                <div>
                  <p className="font-medium">{task.name}</p>
                  <p className="text-sm text-text-secondary font-mono">{task.schedule}</p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <button className={clsx(
                  'p-2 rounded-lg transition-colors',
                  task.enabled ? 'bg-yellow-500/20 hover:bg-yellow-500/30' : 'bg-green-500/20 hover:bg-green-500/30'
                )}>
                  {task.enabled ? <Pause className="w-4 h-4 text-yellow-400" /> : <Play className="w-4 h-4 text-green-400" />}
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function WarmupTab() {
  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">服务器预热脚本</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-nether-800 rounded-lg">
            <div>
              <p className="font-medium">启动时自动预热</p>
              <p className="text-sm text-text-secondary">
                服务器启动后自动执行预热命令
              </p>
            </div>
            <div className="px-3 py-1 rounded-full text-sm font-medium bg-green-500/20 text-green-400">
              已启用
            </div>
          </div>

          <div className="p-4 bg-nether-800 rounded-lg">
            <h4 className="font-medium mb-3">预热命令列表</h4>
            <div className="space-y-2 font-mono text-sm">
              <div className="flex items-center gap-2">
                <ChevronRight className="w-4 h-4 text-mc-green" />
                <code className="px-2 py-1 bg-nether-900 rounded">list</code>
                <span className="text-text-secondary">- 获取玩家列表</span>
              </div>
              <div className="flex items-center gap-2">
                <ChevronRight className="w-4 h-4 text-mc-green" />
                <code className="px-2 py-1 bg-nether-900 rounded">timings on</code>
                <span className="text-text-secondary">- 启用性能监控</span>
              </div>
              <div className="flex items-center gap-2">
                <ChevronRight className="w-4 h-4 text-mc-green" />
                <code className="px-2 py-1 bg-nether-900 rounded">reload</code>
                <span className="text-text-secondary">- 重载配置 (必需)</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function UpdatesTab({
  versionInfo,
  onCheck,
  isLoading,
}: {
  versionInfo: VersionInfo | null;
  onCheck: () => void;
  isLoading: boolean;
}) {
  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">Minecraft 版本更新</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-nether-800 rounded-lg">
            <div>
              <p className="font-medium">检查更新</p>
              <p className="text-sm text-text-secondary">
                从 Mojang 服务器获取最新版本信息
              </p>
            </div>
            <button
              onClick={onCheck}
              className="game-button flex items-center gap-2"
              disabled={isLoading}
            >
              {isLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
              检查更新
            </button>
          </div>

          {versionInfo && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="p-4 bg-nether-800 rounded-lg">
                <p className="text-text-secondary text-sm">当前版本</p>
                <p className="text-2xl font-bold font-mono">{versionInfo.current_version}</p>
              </div>
              <div className="p-4 bg-nether-800 rounded-lg">
                <p className="text-text-secondary text-sm">最新版本</p>
                <p className="text-2xl font-bold font-mono">
                  {versionInfo.latest_version || '未知'}
                  {versionInfo.update_available && (
                    <span className="ml-2 px-2 py-0.5 text-sm bg-green-500/20 text-green-400 rounded">
                      有更新
                    </span>
                  )}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function DiskTab({
  diskInfo,
  alerts,
  onAcknowledge,
}: {
  diskInfo: DiskInfo[];
  alerts: DiskAlert[];
  onAcknowledge: (id: string) => void;
}) {
  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">磁盘空间监控</h3>

        {diskInfo.length > 0 && (
          <div className="mb-6">
            {diskInfo.map((disk) => (
              <div key={disk.path} className="p-4 bg-nether-800 rounded-lg mb-3 last:mb-0">
                <div className="flex items-center justify-between mb-2">
                  <span className="font-mono text-sm">{disk.path}</span>
                  <span className="font-bold">
                    {formatBytes(disk.used_bytes)} / {formatBytes(disk.total_bytes)}
                  </span>
                </div>
                <div className="w-full bg-nether-700 rounded-full h-3">
                  <div
                    className={clsx(
                      'h-3 rounded-full transition-all duration-300',
                      disk.usage_percent >= 95 ? 'bg-red-500' :
                      disk.usage_percent >= 80 ? 'bg-yellow-500' : 'bg-mc-green'
                    )}
                    style={{ width: `${Math.min(disk.usage_percent, 100)}%` }}
                  />
                </div>
                <p className="text-right text-sm text-text-secondary mt-1">
                  {disk.usage_percent.toFixed(1)}% 使用 · {formatBytes(disk.available_bytes)} 可用
                </p>
              </div>
            ))}
          </div>
        )}

        {alerts.length > 0 && (
          <div>
            <h4 className="font-medium mb-3 flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 text-yellow-400" />
              待处理预警 ({alerts.length})
            </h4>
            <div className="space-y-2">
              {alerts.map((alert) => (
                <div
                  key={alert.id}
                  className={clsx(
                    'flex items-center justify-between p-3 rounded-lg',
                    alert.level === 'critical' ? 'bg-red-500/20' : 'bg-yellow-500/20'
                  )}
                >
                  <div className="flex items-center gap-3">
                    {alert.level === 'critical' ? (
                      <AlertTriangle className="w-5 h-5 text-red-400" />
                    ) : (
                      <AlertTriangle className="w-5 h-5 text-yellow-400" />
                    )}
                    <div>
                      <p className="font-medium">{alert.path}</p>
                      <p className="text-sm text-text-secondary">
                        使用率: {alert.usage_percent.toFixed(1)}% · 可用: {formatBytes(alert.available_bytes)}
                      </p>
                    </div>
                  </div>
                  <button
                    onClick={() => onAcknowledge(alert.id)}
                    className="p-2 hover:bg-nether-700 rounded-lg"
                    title="确认预警"
                  >
                    <Check className="w-4 h-4 text-mc-green" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {diskInfo.length === 0 && alerts.length === 0 && (
          <div className="text-center py-8">
            <HardDrive className="w-12 h-12 mx-auto text-text-muted mb-4" />
            <p className="text-text-secondary">磁盘状态正常</p>
          </div>
        )}
      </div>
    </div>
  );
}

function TestsTab({
  suites,
  onRunTests,
  isLoading,
}: {
  suites: TestSuite[];
  onRunTests: () => void;
  isLoading: boolean;
}) {
  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="font-display text-lg font-semibold">自动化测试套件</h3>
        <button
          onClick={onRunTests}
          className="game-button game-button-primary flex items-center gap-2"
          disabled={isLoading}
        >
          {isLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : <TestTube className="w-4 h-4" />}
          运行全部测试
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {suites.map((suite) => (
          <div key={suite.id} className="game-card">
            <h4 className="font-medium mb-3">{suite.name}</h4>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-text-secondary">总测试数</span>
                <span className="font-mono">{suite.total_tests}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-secondary">通过</span>
                <span className="font-mono text-mc-green">{suite.passed_tests}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-text-secondary">失败</span>
                <span className="font-mono text-red-400">{suite.failed_tests}</span>
              </div>
            </div>
            {suite.last_run && (
              <p className="text-xs text-text-muted mt-3">
                上次运行: {formatDate(suite.last_run)}
              </p>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function VersionsTab({ versions }: { versions: ConfigVersion[] }) {
  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">配置版本管理</h3>

        {versions.length === 0 ? (
          <div className="text-center py-8">
            <GitBranch className="w-12 h-12 mx-auto text-text-muted mb-4" />
            <p className="text-text-secondary">暂无配置版本记录</p>
            <p className="text-sm text-text-muted mt-1">修改配置时会自动创建快照</p>
          </div>
        ) : (
          <div className="space-y-3">
            {versions.map((version) => (
              <div key={version.id} className="flex items-center justify-between p-4 bg-nether-800 rounded-lg">
                <div className="flex items-center gap-4">
                  <div className="w-10 h-10 bg-purple-500/20 rounded-lg flex items-center justify-center">
                    <GitBranch className="w-5 h-5 text-purple-400" />
                  </div>
                  <div>
                    <p className="font-medium font-mono">{version.version}</p>
                    <p className="text-sm text-text-secondary">{version.description}</p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-sm text-text-muted">
                    {formatDate(version.created_at)}
                  </span>
                  <button className="p-2 hover:bg-nether-700 rounded-lg" title="回滚到此版本">
                    <RotateCcw className="w-4 h-4 text-text-secondary" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function MigrationTab({ plans }: { plans: MigrationPlan[] }) {
  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="font-display text-lg font-semibold mb-4">数据迁移工具</h3>

        {plans.length === 0 ? (
          <div className="text-center py-8">
            <ArrowRightLeft className="w-12 h-12 mx-auto text-text-muted mb-4" />
            <p className="text-text-secondary">暂无迁移计划</p>
            <p className="text-sm text-text-muted mt-1">创建新迁移计划来迁移服务器数据</p>
          </div>
        ) : (
          <div className="space-y-3">
            {plans.map((plan) => (
              <div key={plan.id} className="p-4 bg-nether-800 rounded-lg">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-3">
                    <ArrowRightLeft className="w-5 h-5 text-blue-400" />
                    <div>
                      <p className="font-medium">{plan.source_path}</p>
                      <p className="text-sm text-text-secondary">→ {plan.target_path}</p>
                    </div>
                  </div>
                  <div className={clsx(
                    'px-3 py-1 rounded-full text-sm font-medium',
                    plan.status === 'completed' ? 'bg-green-500/20 text-green-400' :
                    plan.status === 'running' ? 'bg-blue-500/20 text-blue-400' :
                    plan.status === 'failed' ? 'bg-red-500/20 text-red-400' :
                    'bg-gray-500/20 text-gray-400'
                  )}>
                    {plan.status === 'completed' ? '已完成' :
                     plan.status === 'running' ? '进行中' :
                     plan.status === 'failed' ? '失败' : '待处理'}
                  </div>
                </div>
                <div className="w-full bg-nether-700 rounded-full h-2">
                  <div
                    className="h-2 rounded-full bg-blue-500 transition-all duration-300"
                    style={{ width: `${plan.steps.filter(s => s.status === 'completed').length / plan.steps.length * 100}%` }}
                  />
                </div>
                <p className="text-sm text-text-muted mt-2">
                  预计大小: {formatBytes(plan.estimated_size)}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

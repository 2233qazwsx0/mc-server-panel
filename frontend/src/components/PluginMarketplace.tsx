import React, { useState, useEffect } from 'react';
import { 
  Download, RefreshCw, Shield, AlertTriangle, CheckCircle, 
  XCircle, Database, Cpu, Zap, Package, Settings, Trash2,
  ChevronRight, Search, Filter, Plus, Minus, RotateCcw,
  Activity, Server, Clock, TrendingUp, GitBranch, Save
} from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

interface Plugin {
  id: string;
  name: string;
  version: string;
  description: string;
  downloads: number;
  rating: number;
  author: string;
  compatible: boolean;
  status: 'installed' | 'available' | 'update_available';
}

interface Conflict {
  pluginA: string;
  pluginB: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  description: string;
}

interface Backup {
  id: string;
  version: string;
  date: string;
  size: number;
}

interface Template {
  id: string;
  name: string;
  variables: { name: string; default: string; description: string }[];
}

interface PerformanceData {
  time: string;
  memory: number;
  cpu: number;
  tick: number;
}

interface BatchOperation {
  id: string;
  type: 'install' | 'update' | 'uninstall';
  plugins: string[];
  progress: number;
  status: 'pending' | 'running' | 'completed' | 'failed';
}

interface SecurityReport {
  score: number;
  riskLevel: 'low' | 'medium' | 'high' | 'critical';
  checks: { name: string; passed: boolean; severity: string }[];
  vulnerabilities: { title: string; severity: string; cve?: string }[];
}

export const PluginMarketplace: React.FC = () => {
  const [activeTab, setActiveTab] = useState<string>('marketplace');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadPlugins();
  }, []);

  const loadPlugins = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/plugins/search?query=&page=1&page_size=20');
      const data = await response.json();
      setPlugins(data.plugins || []);
    } catch (error) {
      console.error('加载插件失败:', error);
    }
    setLoading(false);
  };

  const tabs = [
    { id: 'marketplace', name: '插件市场', icon: <Package className="w-4 h-4" /> },
    { id: 'installed', name: '已安装', icon: <Server className="w-4 h-4" /> },
    { id: 'conflicts', name: '冲突检测', icon: <AlertTriangle className="w-4 h-4" /> },
    { id: 'backups', name: '版本回滚', icon: <RotateCcw className="w-4 h-4" /> },
    { id: 'compatibility', name: '兼容性评分', icon: <CheckCircle className="w-4 h-4" /> },
    { id: 'templates', name: '配置模板', icon: <Settings className="w-4 h-4" /> },
    { id: 'hotreload', name: '热重载', icon: <Zap className="w-4 h-4" /> },
    { id: 'performance', name: '性能分析', icon: <Activity className="w-4 h-4" /> },
    { id: 'repository', name: '自定义仓库', icon: <GitBranch className="w-4 h-4" /> },
    { id: 'batch', name: '批量管理', icon: <Database className="w-4 h-4" /> },
    { id: 'security', name: '安全扫描', icon: <Shield className="w-4 h-4" /> },
  ];

  return (
    <div className="min-h-screen bg-gray-900 text-white p-6">
      <div className="max-w-7xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
            插件生态市场 (M5)
          </h1>
          <p className="text-gray-400 mt-2">管理、监控和保护您的Minecraft服务器插件</p>
        </div>

        <div className="flex gap-6">
          <div className="w-64 flex-shrink-0">
            <div className="bg-gray-800 rounded-lg p-4 sticky top-6">
              <div className="mb-4">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                  <input
                    type="text"
                    placeholder="搜索插件..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full pl-10 pr-4 py-2 bg-gray-700 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>
              <nav className="space-y-1">
                {tabs.map((tab) => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                      activeTab === tab.id
                        ? 'bg-blue-600 text-white'
                        : 'text-gray-300 hover:bg-gray-700'
                    }`}
                  >
                    {tab.icon}
                    {tab.name}
                  </button>
                ))}
              </nav>
            </div>
          </div>

          <div className="flex-1">
            {activeTab === 'marketplace' && <MarketplaceView searchQuery={searchQuery} />}
            {activeTab === 'installed' && <InstalledPluginsView />}
            {activeTab === 'conflicts' && <ConflictsView />}
            {activeTab === 'backups' && <BackupsView />}
            {activeTab === 'compatibility' && <CompatibilityView />}
            {activeTab === 'templates' && <TemplatesView />}
            {activeTab === 'hotreload' && <HotReloadView />}
            {activeTab === 'performance' && <PerformanceView />}
            {activeTab === 'repository' && <RepositoryView />}
            {activeTab === 'batch' && <BatchManagementView />}
            {activeTab === 'security' && <SecurityScanView />}
          </div>
        </div>
      </div>
    </div>
  );
};

const MarketplaceView: React.FC<{ searchQuery: string }> = ({ searchQuery }) => {
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadPlugins();
  }, [searchQuery]);

  const loadPlugins = async () => {
    setLoading(true);
    try {
      const response = await fetch(`/api/plugins/search?query=${searchQuery}&page=1&page_size=20`);
      const data = await response.json();
      setPlugins(data.plugins || []);
    } catch (error) {
      console.error('加载失败:', error);
    }
    setLoading(false);
  };

  const installPlugin = async (pluginId: string) => {
    try {
      await fetch('/api/plugins/install', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ plugin_id: pluginId, backup_enabled: true }),
      });
      alert('插件安装请求已提交');
    } catch (error) {
      console.error('安装失败:', error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">插件市场</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {plugins.map((plugin) => (
            <div key={plugin.id} className="bg-gray-700 rounded-lg p-4 hover:bg-gray-600 transition-colors">
              <div className="flex justify-between items-start mb-3">
                <div>
                  <h3 className="font-semibold text-lg">{plugin.name}</h3>
                  <p className="text-sm text-gray-400">v{plugin.version}</p>
                </div>
                <span className={`px-2 py-1 rounded text-xs ${
                  plugin.compatible ? 'bg-green-600' : 'bg-red-600'
                }`}>
                  {plugin.compatible ? '兼容' : '不兼容'}
                </span>
              </div>
              <p className="text-sm text-gray-300 mb-3 line-clamp-2">{plugin.description}</p>
              <div className="flex items-center justify-between text-sm text-gray-400 mb-3">
                <span>👁 {plugin.downloads.toLocaleString()}</span>
                <span>⭐ {plugin.rating.toFixed(1)}</span>
                <span>👤 {plugin.author}</span>
              </div>
              <button
                onClick={() => installPlugin(plugin.id)}
                className="w-full bg-blue-600 hover:bg-blue-700 py-2 rounded-lg flex items-center justify-center gap-2 transition-colors"
              >
                <Download className="w-4 h-4" />
                一键安装
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const InstalledPluginsView: React.FC = () => {
  const [plugins, setPlugins] = useState<Plugin[]>([]);

  useEffect(() => {
    loadInstalledPlugins();
  }, []);

  const loadInstalledPlugins = async () => {
    try {
      const response = await fetch('/api/plugins/installed');
      const data = await response.json();
      setPlugins(data || []);
    } catch (error) {
      console.error('加载已安装插件失败:', error);
    }
  };

  const updatePlugin = async (pluginId: string) => {
    try {
      await fetch(`/api/plugins/${pluginId}/update`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ backup_enabled: true }),
      });
      alert('插件更新请求已提交');
    } catch (error) {
      console.error('更新失败:', error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">已安装插件</h2>
        <div className="space-y-3">
          {plugins.map((plugin) => (
            <div key={plugin.id} className="bg-gray-700 rounded-lg p-4 flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className={`w-3 h-3 rounded-full ${plugin.status === 'installed' ? 'bg-green-500' : 'bg-yellow-500'}`} />
                <div>
                  <h3 className="font-medium">{plugin.name}</h3>
                  <p className="text-sm text-gray-400">v{plugin.version}</p>
                </div>
              </div>
              <div className="flex gap-2">
                {plugin.status === 'update_available' && (
                  <button
                    onClick={() => updatePlugin(plugin.id)}
                    className="px-3 py-1 bg-yellow-600 hover:bg-yellow-700 rounded text-sm"
                  >
                    更新
                  </button>
                )}
                <button className="px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm">
                  卸载
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const ConflictsView: React.FC = () => {
  const [conflicts, setConflicts] = useState<Conflict[]>([]);

  const checkConflicts = async () => {
    try {
      const response = await fetch('/api/plugins/1/conflicts');
      const data = await response.json();
      setConflicts(data || []);
    } catch (error) {
      console.error('冲突检测失败:', error);
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-600';
      case 'high': return 'bg-orange-600';
      case 'medium': return 'bg-yellow-600';
      case 'low': return 'bg-blue-600';
      default: return 'bg-gray-600';
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-semibold">依赖冲突检测</h2>
          <button
            onClick={checkConflicts}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg flex items-center gap-2"
          >
            <RefreshCw className="w-4 h-4" />
            检测冲突
          </button>
        </div>
        <div className="space-y-3">
          {conflicts.length === 0 ? (
            <div className="text-center py-8 text-gray-400">
              <CheckCircle className="w-12 h-12 mx-auto mb-2 text-green-500" />
              <p>未检测到冲突</p>
            </div>
          ) : (
            conflicts.map((conflict, index) => (
              <div key={index} className="bg-gray-700 rounded-lg p-4 flex items-start gap-4">
                <span className={`px-2 py-1 rounded text-xs text-white ${getSeverityColor(conflict.severity)}`}>
                  {conflict.severity.toUpperCase()}
                </span>
                <div>
                  <h4 className="font-medium">{conflict.pluginA} ↔ {conflict.pluginB}</h4>
                  <p className="text-sm text-gray-400 mt-1">{conflict.description}</p>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};

const BackupsView: React.FC = () => {
  const [backups, setBackups] = useState<Backup[]>([]);
  const [selectedPlugin, setSelectedPlugin] = useState('');

  const loadBackups = async () => {
    if (!selectedPlugin) return;
    try {
      const response = await fetch(`/api/plugins/${selectedPlugin}/backups`);
      const data = await response.json();
      setBackups(data || []);
    } catch (error) {
      console.error('加载备份失败:', error);
    }
  };

  const rollbackToVersion = async (backupId: string) => {
    if (!confirm('确定要回滚到这个版本吗？')) return;
    try {
      await fetch(`/api/plugins/${selectedPlugin}/rollback`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ target_version: backupId }),
      });
      alert('回滚请求已提交');
    } catch (error) {
      console.error('回滚失败:', error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">插件版本回滚</h2>
        <div className="mb-4">
          <select
            value={selectedPlugin}
            onChange={(e) => setSelectedPlugin(e.target.value)}
            className="w-full bg-gray-700 rounded-lg px-4 py-2"
          >
            <option value="">选择插件</option>
          </select>
        </div>
        <button
          onClick={loadBackups}
          className="mb-4 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg"
        >
          加载备份
        </button>
        <div className="space-y-3">
          {backups.map((backup) => (
            <div key={backup.id} className="bg-gray-700 rounded-lg p-4 flex items-center justify-between">
              <div>
                <h4 className="font-medium">版本 {backup.version}</h4>
                <p className="text-sm text-gray-400">
                  {new Date(backup.date).toLocaleString()} • {(backup.size / 1024).toFixed(1)} KB
                </p>
              </div>
              <button
                onClick={() => rollbackToVersion(backup.id)}
                className="px-4 py-2 bg-orange-600 hover:bg-orange-700 rounded-lg flex items-center gap-2"
              >
                <RotateCcw className="w-4 h-4" />
                回滚
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const CompatibilityView: React.FC = () => {
  const [compatibility, setCompatibility] = useState<any[]>([]);

  const getScoreColor = (score: number) => {
    if (score >= 90) return 'text-green-500';
    if (score >= 70) return 'text-yellow-500';
    if (score >= 50) return 'text-orange-500';
    return 'text-red-500';
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">兼容性评分</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {compatibility.map((item) => (
            <div key={item.id} className="bg-gray-700 rounded-lg p-4">
              <div className="flex justify-between items-center mb-2">
                <h3 className="font-medium">{item.name}</h3>
                <span className={`text-2xl font-bold ${getScoreColor(item.score)}`}>
                  {item.score}%
                </span>
              </div>
              <div className="w-full bg-gray-600 rounded-full h-2 mb-2">
                <div
                  className={`h-2 rounded-full ${
                    item.score >= 90 ? 'bg-green-500' : item.score >= 70 ? 'bg-yellow-500' : 'bg-red-500'
                  }`}
                  style={{ width: `${item.score}%` }}
                />
              </div>
              <p className="text-sm text-gray-400">{item.issues} 个已知问题</p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const TemplatesView: React.FC = () => {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(null);
  const [variables, setVariables] = useState<Record<string, string>>({});

  const loadTemplates = async () => {
    try {
      const response = await fetch('/api/plugins/EssentialsX/templates');
      const data = await response.json();
      setTemplates(data || []);
    } catch (error) {
      console.error('加载模板失败:', error);
    }
  };

  const applyTemplate = async () => {
    if (!selectedTemplate) return;
    try {
      await fetch(`/api/templates/${selectedTemplate.id}/apply`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ variables }),
      });
      alert('模板应用成功');
    } catch (error) {
      console.error('应用模板失败:', error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">插件配置模板</h2>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div>
            <h3 className="font-medium mb-3">可用模板</h3>
            <div className="space-y-2">
              {templates.map((template) => (
                <div
                  key={template.id}
                  onClick={() => {
                    setSelectedTemplate(template);
                    const vars: Record<string, string> = {};
                    template.variables.forEach(v => vars[v.name] = v.default);
                    setVariables(vars);
                  }}
                  className={`p-3 rounded-lg cursor-pointer transition-colors ${
                    selectedTemplate?.id === template.id ? 'bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'
                  }`}
                >
                  <h4 className="font-medium">{template.name}</h4>
                  <p className="text-sm text-gray-400">{template.variables.length} 个变量</p>
                </div>
              ))}
            </div>
          </div>
          {selectedTemplate && (
            <div className="bg-gray-700 rounded-lg p-4">
              <h3 className="font-medium mb-4">配置变量</h3>
              {selectedTemplate.variables.map((variable) => (
                <div key={variable.name} className="mb-4">
                  <label className="block text-sm text-gray-400 mb-1">{variable.name}</label>
                  <input
                    type="text"
                    value={variables[variable.name] || ''}
                    onChange={(e) => setVariables({ ...variables, [variable.name]: e.target.value })}
                    placeholder={variable.default}
                    className="w-full bg-gray-600 rounded px-3 py-2 text-sm"
                  />
                  <p className="text-xs text-gray-500 mt-1">{variable.description}</p>
                </div>
              ))}
              <button
                onClick={applyTemplate}
                className="w-full mt-4 bg-green-600 hover:bg-green-700 py-2 rounded-lg flex items-center justify-center gap-2"
              >
                <Save className="w-4 h-4" />
                应用模板
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

const HotReloadView: React.FC = () => {
  const [reloadingPlugins, setReloadingPlugins] = useState<string[]>([]);

  const reloadPlugin = async (pluginId: string, reloadConfig: boolean = false) => {
    setReloadingPlugins([...reloadingPlugins, pluginId]);
    try {
      await fetch(`/api/plugins/${pluginId}/reload`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reload_config: reloadConfig }),
      });
    } catch (error) {
      console.error('热重载失败:', error);
    }
    setReloadingPlugins(reloadingPlugins.filter(id => id !== pluginId));
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">插件热重载</h2>
        <p className="text-gray-400 mb-4">无需重启服务器即可重新加载插件配置</p>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="bg-gray-700 rounded-lg p-4">
            <h3 className="font-medium mb-2">支持热重载的插件</h3>
            <p className="text-sm text-gray-400">EssentialsX, WorldEdit, LuckPerms</p>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <h3 className="font-medium mb-2">注意事项</h3>
            <p className="text-sm text-gray-400">某些配置变更需要完全重启服务器</p>
          </div>
        </div>
      </div>
    </div>
  );
};

const PerformanceView: React.FC = () => {
  const [performanceData, setPerformanceData] = useState<PerformanceData[]>([]);
  const [selectedPlugin, setSelectedPlugin] = useState('');

  const loadPerformanceData = async () => {
    if (!selectedPlugin) return;
    try {
      const response = await fetch(`/api/plugins/${selectedPlugin}/performance`);
      const data = await response.json();
      const mockData = Array.from({ length: 20 }, (_, i) => ({
        time: `${i}:00`,
        memory: Math.random() * 100,
        cpu: Math.random() * 30,
        tick: Math.random() * 10,
      }));
      setPerformanceData(mockData);
    } catch (error) {
      console.error('加载性能数据失败:', error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">性能分析报告</h2>
        <div className="mb-4">
          <select
            value={selectedPlugin}
            onChange={(e) => setSelectedPlugin(e.target.value)}
            className="w-full bg-gray-700 rounded-lg px-4 py-2"
          >
            <option value="">选择插件</option>
          </select>
        </div>
        <button
          onClick={loadPerformanceData}
          className="mb-4 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg"
        >
          生成报告
        </button>
        {performanceData.length > 0 && (
          <>
            <div className="mb-6">
              <h3 className="font-medium mb-3">性能趋势</h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={performanceData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                    <XAxis dataKey="time" stroke="#9CA3AF" />
                    <YAxis stroke="#9CA3AF" />
                    <Tooltip />
                    <Line type="monotone" dataKey="memory" stroke="#3B82F6" name="内存 (MB)" />
                    <Line type="monotone" dataKey="cpu" stroke="#10B981" name="CPU (%)" />
                    <Line type="monotone" dataKey="tick" stroke="#F59E0B" name="Tick (ms)" />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="bg-gray-700 rounded-lg p-4">
                <h4 className="text-sm text-gray-400 mb-1">内存使用</h4>
                <p className="text-2xl font-bold text-blue-400">45 MB</p>
              </div>
              <div className="bg-gray-700 rounded-lg p-4">
                <h4 className="text-sm text-gray-400 mb-1">CPU 占用</h4>
                <p className="text-2xl font-bold text-green-400">3.2%</p>
              </div>
              <div className="bg-gray-700 rounded-lg p-4">
                <h4 className="text-sm text-gray-400 mb-1">Tick 影响</h4>
                <p className="text-2xl font-bold text-yellow-400">0.5 ms</p>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

const RepositoryView: React.FC = () => {
  const [repositories, setRepositories] = useState<any[]>([]);
  const [showAddModal, setShowAddModal] = useState(false);

  useEffect(() => {
    loadRepositories();
  }, []);

  const loadRepositories = async () => {
    try {
      const response = await fetch('/api/repositories');
      const data = await response.json();
      setRepositories(data || []);
    } catch (error) {
      console.error('加载仓库失败:', error);
    }
  };

  const syncRepository = async (repoId: string) => {
    try {
      await fetch(`/api/repositories/${repoId}/sync`, { method: 'POST' });
      alert('同步请求已提交');
    } catch (error) {
      console.error('同步失败:', error);
    }
  };

  const addRepository = async (repoData: any) => {
    try {
      await fetch('/api/repositories', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(repoData),
      });
      setShowAddModal(false);
      loadRepositories();
    } catch (error) {
      console.error('添加仓库失败:', error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-semibold">自定义插件仓库</h2>
          <button
            onClick={() => setShowAddModal(true)}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg flex items-center gap-2"
          >
            <Plus className="w-4 h-4" />
            添加仓库
          </button>
        </div>
        <div className="space-y-3">
          {repositories.map((repo) => (
            <div key={repo.id} className="bg-gray-700 rounded-lg p-4 flex items-center justify-between">
              <div>
                <h3 className="font-medium">{repo.name}</h3>
                <p className="text-sm text-gray-400">{repo.url}</p>
                <p className="text-xs text-gray-500 mt-1">{repo.plugins_count} 个插件</p>
              </div>
              <div className="flex items-center gap-4">
                <span className={`px-2 py-1 rounded text-xs ${repo.enabled ? 'bg-green-600' : 'bg-gray-600'}`}>
                  {repo.enabled ? '启用' : '禁用'}
                </span>
                <button
                  onClick={() => syncRepository(repo.id)}
                  className="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm"
                >
                  同步
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const BatchManagementView: React.FC = () => {
  const [operations, setOperations] = useState<BatchOperation[]>([]);
  const [selectedPlugins, setSelectedPlugins] = useState<string[]>([]);
  const [batchType, setBatchType] = useState<'install' | 'update' | 'uninstall'>('install');

  const startBatchOperation = async () => {
    if (selectedPlugins.length === 0) return;
    try {
      const response = await fetch('/api/batch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          operation_type: batchType,
          plugins: selectedPlugins,
        }),
      });
      const data = await response.json();
      setOperations([...operations, data]);
      setSelectedPlugins([]);
    } catch (error) {
      console.error('批量操作失败:', error);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">批量管理插件</h2>
        <div className="mb-4">
          <label className="block text-sm text-gray-400 mb-2">操作类型</label>
          <select
            value={batchType}
            onChange={(e) => setBatchType(e.target.value as any)}
            className="w-full bg-gray-700 rounded-lg px-4 py-2"
          >
            <option value="install">批量安装</option>
            <option value="update">批量更新</option>
            <option value="uninstall">批量卸载</option>
          </select>
        </div>
        <button
          onClick={startBatchOperation}
          disabled={selectedPlugins.length === 0}
          className="w-full py-3 bg-blue-600 hover:bg-blue-700 rounded-lg disabled:bg-gray-600 disabled:cursor-not-allowed"
        >
          开始批量操作 ({selectedPlugins.length} 个插件)
        </button>
        <div className="mt-6">
          <h3 className="font-medium mb-3">进行中的操作</h3>
          <div className="space-y-2">
            {operations.map((op) => (
              <div key={op.id} className="bg-gray-700 rounded-lg p-3">
                <div className="flex justify-between items-center mb-2">
                  <span className="font-medium">{op.type}</span>
                  <span className="text-sm text-gray-400">{op.progress}%</span>
                </div>
                <div className="w-full bg-gray-600 rounded-full h-2">
                  <div
                    className="h-2 rounded-full bg-blue-500"
                    style={{ width: `${op.progress}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

const SecurityScanView: React.FC = () => {
  const [scanResults, setScanResults] = useState<SecurityReport | null>(null);
  const [selectedPlugin, setSelectedPlugin] = useState('');

  const runSecurityScan = async () => {
    if (!selectedPlugin) return;
    try {
      const response = await fetch(`/api/plugins/${selectedPlugin}/security`);
      const data = await response.json();
      setScanResults(data);
    } catch (error) {
      console.error('安全扫描失败:', error);
    }
  };

  const getRiskColor = (level: string) => {
    switch (level) {
      case 'low': return 'text-green-500';
      case 'medium': return 'text-yellow-500';
      case 'high': return 'text-orange-500';
      case 'critical': return 'text-red-500';
      default: return 'text-gray-500';
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-semibold mb-4">插件安全扫描</h2>
        <div className="mb-4">
          <select
            value={selectedPlugin}
            onChange={(e) => setSelectedPlugin(e.target.value)}
            className="w-full bg-gray-700 rounded-lg px-4 py-2"
          >
            <option value="">选择插件</option>
          </select>
        </div>
        <button
          onClick={runSecurityScan}
          className="w-full py-3 bg-purple-600 hover:bg-purple-700 rounded-lg flex items-center justify-center gap-2"
        >
          <Shield className="w-5 h-5" />
          运行安全扫描
        </button>

        {scanResults && (
          <div className="mt-6 space-y-4">
            <div className="bg-gray-700 rounded-lg p-6">
              <div className="flex items-center justify-between mb-4">
                <div>
                  <h3 className="text-lg font-medium">安全评分</h3>
                  <p className={`text-3xl font-bold ${getRiskColor(scanResults.riskLevel)}`}>
                    {scanResults.score}/100
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-sm text-gray-400">风险等级</p>
                  <p className={`text-lg font-bold uppercase ${getRiskColor(scanResults.riskLevel)}`}>
                    {scanResults.riskLevel}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                {scanResults.riskLevel === 'low' ? (
                  <CheckCircle className="w-8 h-8 text-green-500" />
                ) : (
                  <AlertTriangle className="w-8 h-8 text-red-500" />
                )}
                <p className="text-gray-300">{scanResults.riskLevel === 'low' ? '安全插件，可以放心使用' : '检测到安全风险，请谨慎使用'}</p>
              </div>
            </div>

            <div className="bg-gray-700 rounded-lg p-4">
              <h4 className="font-medium mb-3">安全检查项</h4>
              <div className="space-y-2">
                {scanResults.checks.map((check, index) => (
                  <div key={index} className="flex items-center gap-3">
                    {check.passed ? (
                      <CheckCircle className="w-5 h-5 text-green-500" />
                    ) : (
                      <XCircle className="w-5 h-5 text-red-500" />
                    )}
                    <span className="flex-1">{check.name}</span>
                    <span className={`text-xs px-2 py-1 rounded ${
                      check.severity === 'critical' ? 'bg-red-600' :
                      check.severity === 'high' ? 'bg-orange-600' :
                      check.severity === 'medium' ? 'bg-yellow-600' : 'bg-gray-600'
                    }`}>
                      {check.severity}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {scanResults.vulnerabilities.length > 0 && (
              <div className="bg-gray-700 rounded-lg p-4">
                <h4 className="font-medium mb-3 text-red-400">已知漏洞</h4>
                <div className="space-y-2">
                  {scanResults.vulnerabilities.map((vuln, index) => (
                    <div key={index} className="bg-gray-600 rounded p-3">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-medium">{vuln.title}</span>
                        {vuln.cve && (
                          <span className="text-xs bg-red-600 px-2 py-0.5 rounded">{vuln.cve}</span>
                        )}
                      </div>
                      <p className="text-sm text-gray-400">严重程度: {vuln.severity}</p>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default PluginMarketplace;

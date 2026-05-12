import { useState, useEffect } from 'react';
import {
  Network, Server, Users, Cpu, HardDrive, Zap, Shield,
  RefreshCw, Plus, Trash2, Settings, AlertTriangle, CheckCircle,
  XCircle, Clock, Activity, GitBranch, ChevronRight, X,
  Wifi, WifiOff, Play, Pause
} from 'lucide-react';

interface Node {
  id: string;
  name: string;
  host: string;
  port: number;
  status: 'online' | 'offline' | 'maintenance' | 'draining';
  cpuUsage: number;
  memoryUsage: number;
  playerCount: number;
  maxPlayers: number;
  tps: number;
  region?: string;
  isProxy: boolean;
  lastHeartbeat: string;
}

interface ProxyServer {
  id: string;
  name: string;
  address: string;
  motd: string;
  online: boolean;
  playerCount: number;
  maxPlayers: number;
  priority: number;
}

interface Alert {
  id: string;
  type: string;
  severity: 'info' | 'warning' | 'critical';
  nodeId: string;
  message: string;
  timestamp: string;
  acknowledged: boolean;
}

interface ClusterMetrics {
  totalNodes: number;
  onlineNodes: number;
  totalPlayers: number;
  avgCpu: number;
  avgMemory: number;
  avgTps: number;
}

type TabType = 'overview' | 'nodes' | 'proxy' | 'topology' | 'config' | 'failover' | 'updates' | 'monitoring';

export function Cluster() {
  const [activeTab, setActiveTab] = useState<TabType>('overview');
  const [isLoading, setIsLoading] = useState(true);
  const [clusterMetrics] = useState<ClusterMetrics>({
    totalNodes: 0, onlineNodes: 0, totalPlayers: 0, avgCpu: 0, avgMemory: 0, avgTps: 20
  });
  const [nodes] = useState<Node[]>([]);
  const [proxyServers] = useState<ProxyServer[]>([]);
  const [alerts] = useState<Alert[]>([]);
  const [showAddNode, setShowAddNode] = useState(false);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);

  useEffect(() => {
    const timer = setTimeout(() => setIsLoading(false), 1000);
    return () => clearTimeout(timer);
  }, []);

  const tabs = [
    { id: 'overview', label: '概览', icon: Activity },
    { id: 'nodes', label: '节点管理', icon: Server },
    { id: 'proxy', label: '代理配置', icon: Network },
    { id: 'topology', label: '拓扑视图', icon: GitBranch },
    { id: 'config', label: '配置中心', icon: Settings },
    { id: 'failover', label: '故障转移', icon: Shield },
    { id: 'updates', label: '滚动更新', icon: RefreshCw },
    { id: 'monitoring', label: '集群监控', icon: Activity },
  ];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return 'text-mc-green';
      case 'offline': return 'text-rust';
      case 'maintenance': return 'text-yellow-400';
      case 'draining': return 'text-orange-400';
      default: return 'text-text-muted';
    }
  };

  const getStatusBg = (status: string) => {
    switch (status) {
      case 'online': return 'bg-mc-green/10 border-mc-green/30';
      case 'offline': return 'bg-rust/10 border-rust/30';
      case 'maintenance': return 'bg-yellow-400/10 border-yellow-400/30';
      case 'draining': return 'bg-orange-400/10 border-orange-400/30';
      default: return 'bg-nether-700/50 border-nether-600';
    }
  };

  const renderOverview = () => (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard title="总节点数" value={clusterMetrics.totalNodes} icon={<Server className="w-5 h-5" />} color="green" />
        <MetricCard title="在线节点" value={clusterMetrics.onlineNodes} icon={<CheckCircle className="w-5 h-5" />} color="green" />
        <MetricCard title="总玩家数" value={clusterMetrics.totalPlayers} icon={<Users className="w-5 h-5" />} color="purple" />
        <MetricCard title="平均 TPS" value={clusterMetrics.avgTps.toFixed(1)} icon={<Zap className="w-5 h-5" />} color="yellow" suffix="/20" />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="game-card">
          <h3 className="font-display text-lg text-text-primary mb-4">节点状态</h3>
          <div className="space-y-3">
            {nodes.length === 0 ? (
              <p className="text-text-muted text-sm">暂无节点</p>
            ) : (
              nodes.slice(0, 4).map(node => (
                <div key={node.id} className={`p-3 rounded-lg border ${getStatusBg(node.status)}`}>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <Server className={`w-4 h-4 ${getStatusColor(node.status)}`} />
                      <span className="font-mono text-sm text-text-primary">{node.name}</span>
                      {node.isProxy && <span className="px-2 py-0.5 bg-mc-green/20 text-mc-green text-xs rounded">代理</span>}
                    </div>
                    <span className={`text-xs font-mono ${getStatusColor(node.status)}`}>
                      {node.status === 'online' ? `${node.playerCount}/${node.maxPlayers}` : node.status}
                    </span>
                  </div>
                  <div className="mt-2 flex gap-4 text-xs text-text-secondary">
                    <span>CPU: {node.cpuUsage.toFixed(1)}%</span>
                    <span>MEM: {node.memoryUsage.toFixed(1)}%</span>
                    <span>TPS: {node.tps.toFixed(1)}</span>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="game-card">
          <h3 className="font-display text-lg text-text-primary mb-4">告警信息</h3>
          <div className="space-y-3">
            {alerts.length === 0 ? (
              <div className="text-center py-6">
                <CheckCircle className="w-8 h-8 text-mc-green mx-auto mb-2" />
                <p className="text-text-secondary text-sm">暂无告警</p>
              </div>
            ) : (
              alerts.slice(0, 4).map(alert => (
                <div key={alert.id} className={`p-3 rounded-lg border ${
                  alert.severity === 'critical' ? 'bg-rust/10 border-rust/30' :
                  alert.severity === 'warning' ? 'bg-yellow-400/10 border-yellow-400/30' :
                  'bg-blue-500/10 border-blue-500/30'
                }`}>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      {alert.severity === 'critical' ? <XCircle className="w-4 h-4 text-rust" /> :
                       alert.severity === 'warning' ? <AlertTriangle className="w-4 h-4 text-yellow-400" /> :
                       <Activity className="w-4 h-4 text-blue-400" />}
                      <span className="text-sm text-text-primary">{alert.message}</span>
                    </div>
                    {!alert.acknowledged && <button className="text-xs text-mc-green hover:underline">确认</button>}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );

  const renderNodes = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="font-display text-xl text-text-primary">节点列表</h2>
        <button onClick={() => setShowAddNode(true)} className="game-button game-button-primary flex items-center gap-2">
          <Plus className="w-4 h-4" /> 添加节点
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {nodes.length === 0 ? (
          <div className="col-span-full text-center py-12">
            <Server className="w-12 h-12 text-text-muted mx-auto mb-4" />
            <p className="text-text-secondary">暂无节点，点击添加第一个节点</p>
          </div>
        ) : (
          nodes.map(node => (
            <div
              key={node.id}
              className={`game-card cursor-pointer hover:border-mc-green/50 transition-colors ${selectedNode?.id === node.id ? 'border-mc-green/50' : ''}`}
              onClick={() => setSelectedNode(node)}
            >
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                  <Server className={`w-5 h-5 ${getStatusColor(node.status)}`} />
                  <div>
                    <h3 className="font-mono text-text-primary">{node.name}</h3>
                    <p className="text-xs text-text-muted">{node.host}:{node.port}</p>
                  </div>
                </div>
                <span className={`px-2 py-1 text-xs rounded border ${getStatusBg(node.status)} ${getStatusColor(node.status)}`}>
                  {node.status}
                </span>
              </div>

              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <p className="text-text-muted text-xs">CPU</p>
                  <div className="flex items-center gap-2">
                    <div className="flex-1 h-1.5 bg-nether-700 rounded-full overflow-hidden">
                      <div className="h-full bg-mc-green transition-all" style={{ width: `${node.cpuUsage}%` }} />
                    </div>
                    <span className="text-xs text-text-secondary">{node.cpuUsage.toFixed(0)}%</span>
                  </div>
                </div>
                <div>
                  <p className="text-text-muted text-xs">内存</p>
                  <div className="flex items-center gap-2">
                    <div className="flex-1 h-1.5 bg-nether-700 rounded-full overflow-hidden">
                      <div className="h-full bg-yellow-400 transition-all" style={{ width: `${node.memoryUsage}%` }} />
                    </div>
                    <span className="text-xs text-text-secondary">{node.memoryUsage.toFixed(0)}%</span>
                  </div>
                </div>
              </div>

              <div className="mt-4 flex justify-between text-xs text-text-secondary">
                <span>玩家: {node.playerCount}/{node.maxPlayers}</span>
                <span>TPS: {node.tps.toFixed(1)}</span>
                {node.region && <span>{node.region}</span>}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );

  const renderProxy = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="font-display text-xl text-text-primary">BungeeCord/Velocity 代理配置</h2>
        <div className="flex gap-2">
          <button className="game-button flex items-center gap-2">
            <Settings className="w-4 h-4" /> 配置
          </button>
          <button className="game-button game-button-primary flex items-center gap-2">
            <Plus className="w-4 h-4" /> 添加服务器
          </button>
        </div>
      </div>

      <div className="game-card">
        <h3 className="font-display text-lg text-text-primary mb-4">代理设置</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="p-4 bg-nether-700/50 rounded-lg">
            <p className="text-text-muted text-xs mb-1">代理类型</p>
            <p className="text-text-primary font-mono">Velocity</p>
          </div>
          <div className="p-4 bg-nether-700/50 rounded-lg">
            <p className="text-text-muted text-xs mb-1">监听地址</p>
            <p className="text-text-primary font-mono">0.0.0.0:25577</p>
          </div>
          <div className="p-4 bg-nether-700/50 rounded-lg">
            <p className="text-text-muted text-xs mb-1">最大玩家数</p>
            <p className="text-text-primary font-mono">1000</p>
          </div>
        </div>
      </div>

      <div className="game-card">
        <h3 className="font-display text-lg text-text-primary mb-4">已注册服务器</h3>
        <div className="space-y-3">
          {proxyServers.length === 0 ? (
            <p className="text-text-muted text-sm text-center py-4">暂无注册服务器</p>
          ) : (
            proxyServers.map(server => (
              <div key={server.id} className="p-4 bg-nether-700/50 rounded-lg flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className={`w-3 h-3 rounded-full ${server.online ? 'bg-mc-green' : 'bg-rust'}`} />
                  <div>
                    <p className="text-text-primary font-mono">{server.name}</p>
                    <p className="text-xs text-text-muted">{server.address} - {server.motd}</p>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  <span className="text-sm text-text-secondary">{server.playerCount}/{server.maxPlayers}</span>
                  <span className="text-xs text-text-muted">优先级: {server.priority}</span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );

  const renderTopology = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="font-display text-xl text-text-primary">集群拓扑可视化</h2>
        <div className="flex gap-2">
          <button className="game-button flex items-center gap-2">
            <RefreshCw className="w-4 h-4" /> 刷新
          </button>
          <button className="game-button flex items-center gap-2">
            <Settings className="w-4 h-4" /> 布局设置
          </button>
        </div>
      </div>

      <div className="game-card">
        <div className="h-[500px] flex items-center justify-center bg-nether-900/50 rounded-lg">
          <div className="text-center">
            <GitBranch className="w-16 h-16 text-text-muted mx-auto mb-4" />
            <p className="text-text-secondary mb-2">拓扑视图</p>
            <p className="text-text-muted text-sm">
              {nodes.length === 0 ? '暂无节点数据' : `${nodes.length} 个节点, ${proxyServers.length} 个服务器`}
            </p>
            <div className="mt-6 flex justify-center gap-8">
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 bg-mc-green/20 border border-mc-green rounded flex items-center justify-center">
                  <Network className="w-4 h-4 text-mc-green" />
                </div>
                <span className="text-xs text-text-secondary">代理节点</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 bg-blue-500/20 border border-blue-500 rounded flex items-center justify-center">
                  <Server className="w-4 h-4 text-blue-400" />
                </div>
                <span className="text-xs text-text-secondary">游戏节点</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-8 h-8 bg-nether-700 border border-nether-600 rounded" />
                <span className="text-xs text-text-secondary">连接线</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  const renderConfig = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="font-display text-xl text-text-primary">集群配置中心</h2>
        <button className="game-button game-button-primary flex items-center gap-2">
          <Plus className="w-4 h-4" /> 新建配置
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {[
          { name: 'Proxy Config', type: '代理配置', version: '1.0.0', updated: '2小时前' },
          { name: 'Load Balancer', type: '负载均衡', version: '1.2.0', updated: '5小时前' },
          { name: 'Chat Sync', type: '聊天同步', version: '1.0.5', updated: '1天前' },
          { name: 'Failover Rules', type: '故障转移', version: '1.1.0', updated: '3天前' },
        ].map((config, i) => (
          <div key={i} className="game-card hover:border-mc-green/30 transition-colors">
            <div className="flex items-start justify-between mb-3">
              <div>
                <h3 className="font-mono text-text-primary">{config.name}</h3>
                <p className="text-xs text-mc-green mt-1">{config.type}</p>
              </div>
              <span className="px-2 py-0.5 bg-nether-600 text-text-muted text-xs rounded">v{config.version}</span>
            </div>
            <div className="flex justify-between items-center text-xs text-text-muted">
              <span>更新于 {config.updated}</span>
              <div className="flex gap-2">
                <button className="hover:text-mc-green transition-colors">编辑</button>
                <button className="hover:text-mc-green transition-colors">历史</button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  const renderFailover = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="font-display text-xl text-text-primary">故障转移管理</h2>
        <div className="flex gap-2">
          <button className="game-button flex items-center gap-2">
            <Settings className="w-4 h-4" /> 设置
          </button>
          <button className="game-button game-button-primary flex items-center gap-2">
            <Shield className="w-4 h-4" /> 手动触发
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="game-card text-center">
          <Shield className="w-8 h-8 text-mc-green mx-auto mb-2" />
          <p className="text-2xl font-display text-text-primary">5</p>
          <p className="text-xs text-text-muted">健康节点</p>
        </div>
        <div className="game-card text-center">
          <AlertTriangle className="w-8 h-8 text-yellow-400 mx-auto mb-2" />
          <p className="text-2xl font-display text-text-primary">2</p>
          <p className="text-xs text-text-muted">降级节点</p>
        </div>
        <div className="game-card text-center">
          <XCircle className="w-8 h-8 text-rust mx-auto mb-2" />
          <p className="text-2xl font-display text-text-primary">0</p>
          <p className="text-xs text-text-muted">不健康节点</p>
        </div>
        <div className="game-card text-center">
          <RefreshCw className="w-8 h-8 text-blue-400 mx-auto mb-2" />
          <p className="text-2xl font-display text-text-primary">3</p>
          <p className="text-xs text-text-muted">总转移次数</p>
        </div>
      </div>

      <div className="game-card">
        <h3 className="font-display text-lg text-text-primary mb-4">故障转移历史</h3>
        <div className="text-center py-8">
          <Shield className="w-12 h-12 text-text-muted mx-auto mb-3" />
          <p className="text-text-secondary">暂无故障转移记录</p>
          <p className="text-xs text-text-muted mt-1">当发生故障转移时，记录将显示在这里</p>
        </div>
      </div>
    </div>
  );

  const renderUpdates = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="font-display text-xl text-text-primary">滚动更新策略</h2>
        <button className="game-button game-button-primary flex items-center gap-2">
          <Plus className="w-4 h-4" /> 新建更新计划
        </button>
      </div>

      <div className="game-card">
        <h3 className="font-display text-lg text-text-primary mb-4">更新策略配置</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-4 bg-nether-700/50 rounded-lg">
            <p className="text-text-muted text-xs mb-2">当前策略</p>
            <select className="w-full bg-nether-600 text-text-primary px-3 py-2 rounded border border-nether-500">
              <option>滚动更新 (Rolling)</option>
              <option>蓝绿部署 (Blue-Green)</option>
              <option>金丝雀发布 (Canary)</option>
            </select>
          </div>
          <div className="p-4 bg-nether-700/50 rounded-lg">
            <p className="text-text-muted text-xs mb-2">批次大小</p>
            <input type="number" defaultValue={1} className="w-full bg-nether-600 text-text-primary px-3 py-2 rounded border border-nether-500" />
          </div>
          <div className="p-4 bg-nether-700/50 rounded-lg">
            <p className="text-text-muted text-xs mb-2">等待时间 (秒)</p>
            <input type="number" defaultValue={60} className="w-full bg-nether-600 text-text-primary px-3 py-2 rounded border border-nether-500" />
          </div>
          <div className="p-4 bg-nether-700/50 rounded-lg">
            <p className="text-text-muted text-xs mb-2">健康检查宽限期 (秒)</p>
            <input type="number" defaultValue={30} className="w-full bg-nether-600 text-text-primary px-3 py-2 rounded border border-nether-500" />
          </div>
        </div>
        <div className="mt-4 flex items-center gap-4">
          <label className="flex items-center gap-2 text-sm text-text-secondary">
            <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-nether-500 bg-nether-600" />
            故障时自动回滚
          </label>
          <label className="flex items-center gap-2 text-sm text-text-secondary">
            <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-nether-500 bg-nether-600" />
            先进入维护模式
          </label>
        </div>
      </div>

      <div className="game-card">
        <h3 className="font-display text-lg text-text-primary mb-4">更新历史</h3>
        <div className="text-center py-8">
          <Clock className="w-12 h-12 text-text-muted mx-auto mb-3" />
          <p className="text-text-secondary">暂无更新记录</p>
          <p className="text-xs text-text-muted mt-1">创建更新计划开始集群升级</p>
        </div>
      </div>
    </div>
  );

  const renderMonitoring = () => (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="font-display text-xl text-text-primary">集群资源监控</h2>
        <div className="flex gap-2">
          <button className="game-button flex items-center gap-2">
            <RefreshCw className="w-4 h-4" /> 刷新
          </button>
          <button className="game-button flex items-center gap-2">
            <Settings className="w-4 h-4" /> 告警设置
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard title="平均 CPU" value={`${clusterMetrics.avgCpu.toFixed(1)}%`} icon={<Cpu className="w-5 h-5" />} color="green" progress={clusterMetrics.avgCpu} />
        <MetricCard title="平均内存" value={`${clusterMetrics.avgMemory.toFixed(1)}%`} icon={<HardDrive className="w-5 h-5" />} color="yellow" progress={clusterMetrics.avgMemory} />
        <MetricCard title="平均 TPS" value={clusterMetrics.avgTps.toFixed(1)} icon={<Zap className="w-5 h-5" />} color="purple" suffix="/20" />
        <MetricCard title="活跃告警" value={alerts.filter(a => !a.acknowledged).length} icon={<AlertTriangle className="w-5 h-5" />} color="rust" />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="game-card">
          <h3 className="font-display text-lg text-text-primary mb-4">CPU 使用趋势</h3>
          <div className="h-48 flex items-center justify-center text-text-muted">
            图表加载中...
          </div>
        </div>
        <div className="game-card">
          <h3 className="font-display text-lg text-text-primary mb-4">内存使用趋势</h3>
          <div className="h-48 flex items-center justify-center text-text-muted">
            图表加载中...
          </div>
        </div>
      </div>
    </div>
  );

  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="h-8 w-48 bg-nether-700 animate-pulse rounded" />
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="h-24 bg-nether-700 animate-pulse rounded-lg" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="font-display text-2xl md:text-3xl font-bold text-text-primary">集群管理</h1>
          <p className="text-text-secondary font-mono mt-1">多节点 Minecraft 服务器集群管理</p>
        </div>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 px-3 py-1.5 bg-mc-green/10 border border-mc-green/30 rounded-lg">
            <div className="w-2 h-2 bg-mc-green rounded-full animate-pulse" />
            <span className="text-mc-green text-sm font-mono">集群在线</span>
          </div>
        </div>
      </div>

      <div className="flex overflow-x-auto gap-1 pb-2 -mx-4 px-4">
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as TabType)}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg whitespace-nowrap transition-colors ${
              activeTab === tab.id
                ? 'bg-mc-green/20 text-mc-green border border-mc-green/30'
                : 'text-text-secondary hover:text-text-primary hover:bg-nether-700/50'
            }`}
          >
            <tab.icon className="w-4 h-4" />
            <span className="text-sm font-mono">{tab.label}</span>
          </button>
        ))}
      </div>

      {activeTab === 'overview' && renderOverview()}
      {activeTab === 'nodes' && renderNodes()}
      {activeTab === 'proxy' && renderProxy()}
      {activeTab === 'topology' && renderTopology()}
      {activeTab === 'config' && renderConfig()}
      {activeTab === 'failover' && renderFailover()}
      {activeTab === 'updates' && renderUpdates()}
      {activeTab === 'monitoring' && renderMonitoring()}

      {showAddNode && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" onClick={() => setShowAddNode(false)}>
          <div className="game-card w-full max-w-md" onClick={e => e.stopPropagation()}>
            <div className="flex justify-between items-center mb-6">
              <h3 className="font-display text-xl text-text-primary">添加节点</h3>
              <button onClick={() => setShowAddNode(false)} className="text-text-muted hover:text-text-primary">
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-text-secondary mb-2">节点名称</label>
                <input type="text" placeholder="e.g., lobby-1" className="w-full bg-nether-700 text-text-primary px-4 py-2 rounded-lg border border-nether-600 focus:border-mc-green outline-none" />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-text-secondary mb-2">主机地址</label>
                  <input type="text" placeholder="192.168.1.100" className="w-full bg-nether-700 text-text-primary px-4 py-2 rounded-lg border border-nether-600 focus:border-mc-green outline-none" />
                </div>
                <div>
                  <label className="block text-sm text-text-secondary mb-2">端口</label>
                  <input type="number" placeholder="8080" defaultValue={8080} className="w-full bg-nether-700 text-text-primary px-4 py-2 rounded-lg border border-nether-600 focus:border-mc-green outline-none" />
                </div>
              </div>
              <div>
                <label className="block text-sm text-text-secondary mb-2">区域</label>
                <select className="w-full bg-nether-700 text-text-primary px-4 py-2 rounded-lg border border-nether-600 focus:border-mc-green outline-none">
                  <option>默认</option>
                  <option>亚洲</option>
                  <option>北美</option>
                  <option>欧洲</option>
                </select>
              </div>
              <div className="flex items-center gap-2">
                <input type="checkbox" id="isProxy" className="w-4 h-4 rounded border-nether-500 bg-nether-600" />
                <label htmlFor="isProxy" className="text-sm text-text-secondary">这是代理节点</label>
              </div>
              <button className="w-full game-button game-button-primary py-2 mt-2">
                添加节点
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function MetricCard({ title, value, icon, color, progress, suffix }: {
  title: string;
  value: string | number;
  icon: React.ReactNode;
  color: 'green' | 'yellow' | 'purple' | 'rust';
  progress?: number;
  suffix?: string;
}) {
  const colorClasses = {
    green: 'text-mc-green bg-mc-green/10 border-mc-green/20',
    yellow: 'text-yellow-400 bg-yellow-400/10 border-yellow-400/20',
    purple: 'text-purple-400 bg-purple-400/10 border-purple-400/20',
    rust: 'text-rust bg-rust/10 border-rust/20',
  };

  return (
    <div className={`game-card ${colorClasses[color]}`}>
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm opacity-80">{title}</span>
        {icon}
      </div>
      <div className="flex items-baseline gap-1">
        <span className="text-2xl font-display font-bold">{value}</span>
        {suffix && <span className="text-sm opacity-60">{suffix}</span>}
      </div>
      {progress !== undefined && (
        <div className="mt-2 h-1.5 bg-black/20 rounded-full overflow-hidden">
          <div className="h-full bg-current transition-all" style={{ width: `${Math.min(progress, 100)}%` }} />
        </div>
      )}
    </div>
  );
}

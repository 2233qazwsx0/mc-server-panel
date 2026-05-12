import { useState, useEffect } from 'react';
import {
  Code, Plug, FileText, Activity, Box, ShieldCheck,
  Gauge, Bell, Terminal, Copy, Download, Check, AlertTriangle,
  Play, Square, Trash2, RefreshCw, Send, Plus,
  Zap, Key, FileCode, Radio
} from 'lucide-react';
import { clsx } from 'clsx';

type TabType =
  | 'api-docs'
  | 'ws-debug'
  | 'plugin-hooks'
  | 'request-logs'
  | 'profiler'
  | 'sdk'
  | 'webhook'
  | 'rate-limit'
  | 'events'
  | 'console';

interface RequestLog {
  id: string;
  method: string;
  path: string;
  status: number;
  duration_ms: number;
  timestamp: string;
  client_ip: string;
}

interface WsSession {
  id: string;
  created_at: string;
  message_count: number;
  is_active: boolean;
}

interface Hook {
  id: string;
  name: string;
  hook_type: string;
  enabled: boolean;
  callback_url?: string;
  created_at: string;
}

interface EventSub {
  id: string;
  name: string;
  event_type: string;
  callback_url: string;
  enabled: boolean;
  has_secret: boolean;
}

interface ConsoleEntry {
  id: string;
  command: string;
  output: string;
  duration_ms: number;
  executed_at: string;
}

interface ProfilerSnapshot {
  id: string;
  name: string;
  timestamp: string;
  duration_ns: number;
  duration_ms: number;
}

const tabs: { id: TabType; label: string; icon: typeof Code }[] = [
  { id: 'api-docs', label: 'API 文档', icon: FileText },
  { id: 'ws-debug', label: 'WebSocket 调试', icon: Radio },
  { id: 'plugin-hooks', label: '插件钩子', icon: Plug },
  { id: 'request-logs', label: '请求日志', icon: FileText },
  { id: 'profiler', label: '性能分析', icon: Activity },
  { id: 'sdk', label: 'SDK 生成', icon: Box },
  { id: 'webhook', label: 'Webhook', icon: ShieldCheck },
  { id: 'rate-limit', label: '速率限制', icon: Gauge },
  { id: 'events', label: '事件订阅', icon: Bell },
  { id: 'console', label: '开发者控制台', icon: Terminal },
];

export function Developer() {
  const [activeTab, setActiveTab] = useState<TabType>('api-docs');
  const [notification, setNotification] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  const showNotification = (type: 'success' | 'error', message: string) => {
    setNotification({ type, message });
    setTimeout(() => setNotification(null), 3000);
  };

  return (
    <div className="space-y-4 md:space-y-6">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="font-display text-2xl md:text-3xl font-bold text-text-primary">
            开发者工具
          </h1>
          <p className="text-text-secondary font-mono mt-1">
            API 调试、插件开发、性能分析
          </p>
        </div>
      </div>

      {notification && (
        <div
          className={clsx(
            'fixed top-4 right-4 z-50 px-4 py-3 rounded-lg shadow-lg max-w-sm animate-slide-in',
            notification.type === 'success'
              ? 'bg-mc-green/20 border border-mc-green text-mc-green'
              : 'bg-rust/20 border border-rust text-rust'
          )}
        >
          <div className="flex items-center gap-2">
            {notification.type === 'success' ? (
              <Check className="w-4 h-4" />
            ) : (
              <AlertTriangle className="w-4 h-4" />
            )}
            <span>{notification.message}</span>
          </div>
        </div>
      )}

      <div className="game-card">
        <div className="flex overflow-x-auto gap-1 p-1 bg-nether-700 rounded-lg">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={clsx(
                'flex items-center gap-2 px-3 py-2 rounded-md text-sm font-mono whitespace-nowrap transition-all',
                activeTab === tab.id
                  ? 'bg-mc-green text-nether-900 shadow-mc-glow'
                  : 'text-text-secondary hover:text-text-primary hover:bg-nether-600'
              )}
            >
              <tab.icon className="w-4 h-4" />
              <span className="hidden sm:inline">{tab.label}</span>
            </button>
          ))}
        </div>
      </div>

      <div className="min-h-[400px]">
        {activeTab === 'api-docs' && <ApiDocsTab />}
        {activeTab === 'ws-debug' && <WsDebugTab showNotification={showNotification} />}
        {activeTab === 'plugin-hooks' && <PluginHooksTab showNotification={showNotification} />}
        {activeTab === 'request-logs' && <RequestLogsTab showNotification={showNotification} />}
        {activeTab === 'profiler' && <ProfilerTab showNotification={showNotification} />}
        {activeTab === 'sdk' && <SdkTab showNotification={showNotification} />}
        {activeTab === 'webhook' && <WebhookTab showNotification={showNotification} />}
        {activeTab === 'rate-limit' && <RateLimitTab showNotification={showNotification} />}
        {activeTab === 'events' && <EventsTab showNotification={showNotification} />}
        {activeTab === 'console' && <ConsoleTab showNotification={showNotification} />}
      </div>
    </div>
  );
}

function ApiDocsTab() {
  const [openApiSpec, setOpenApiSpec] = useState<any>(null);
  const [paths, setPaths] = useState<string[]>([]);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  useEffect(() => {
    fetch('/api/docs/openapi.json')
      .then(res => res.json())
      .then(setOpenApiSpec)
      .catch(console.error);

    fetch('/api/developer/paths')
      .then(res => res.json())
      .then(data => setPaths(data))
      .catch(console.error);
  }, []);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <div className="grid grid-cols-1 xl:grid-cols-3 gap-4 md:gap-6">
      <div className="xl:col-span-1 space-y-4">
        <div className="game-card">
          <h3 className="font-display text-lg font-semibold text-text-primary mb-4 flex items-center gap-2">
            <FileText className="w-5 h-5" />
            API 端点
          </h3>
          <div className="space-y-1 max-h-[500px] overflow-y-auto">
            {paths.map((path) => (
              <button
                key={path}
                onClick={() => setSelectedPath(path)}
                className={clsx(
                  'w-full text-left px-3 py-2 rounded font-mono text-sm transition-colors',
                  selectedPath === path
                    ? 'bg-mc-green/20 text-mc-green'
                    : 'hover:bg-nether-700 text-text-secondary'
                )}
              >
                {path}
              </button>
            ))}
          </div>
        </div>

        <div className="game-card">
          <h3 className="font-display text-lg font-semibold text-text-primary mb-4 flex items-center gap-2">
            <Code className="w-5 h-5" />
            快速操作
          </h3>
          <div className="space-y-2">
            <button
              onClick={() => window.open('/api/docs/openapi.json', '_blank')}
              className="w-full game-button flex items-center justify-center gap-2"
            >
              <Download className="w-4 h-4" />
              下载 OpenAPI JSON
            </button>
            <button
              onClick={() => copyToClipboard(JSON.stringify(openApiSpec, null, 2))}
              className="w-full game-button flex items-center justify-center gap-2"
            >
              <Copy className="w-4 h-4" />
              复制规范
            </button>
          </div>
        </div>
      </div>

      <div className="xl:col-span-2">
        <div className="game-card">
          <h3 className="font-display text-lg font-semibold text-text-primary mb-4 flex items-center gap-2">
            <FileCode className="w-5 h-5" />
            OpenAPI 规范预览
          </h3>
          <pre className="bg-nether-900 p-4 rounded-lg overflow-auto max-h-[600px] text-sm font-mono">
            <code className="text-text-secondary">
              {openApiSpec ? JSON.stringify(openApiSpec, null, 2) : '加载中...'}
            </code>
          </pre>
        </div>
      </div>
    </div>
  );
}

function WsDebugTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [sessions, setSessions] = useState<WsSession[]>([]);
  const [selectedSession, setSelectedSession] = useState<WsSession | null>(null);
  const [messages, setMessages] = useState<any[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [direction, setDirection] = useState<'incoming' | 'outgoing'>('outgoing');

  const loadSessions = () => {
    fetch('/api/developer/ws-debug/sessions')
      .then(res => res.json())
      .then(data => setSessions(data.sessions || []))
      .catch(console.error);
  };

  useEffect(() => {
    loadSessions();
  }, []);

  useEffect(() => {
    if (selectedSession) {
      fetch(`/api/developer/ws-debug/sessions/${selectedSession.id}`)
        .then(res => res.json())
        .then(data => setMessages(data.messages || []))
        .catch(console.error);
    }
  }, [selectedSession]);

  const createSession = async () => {
    try {
      const res = await fetch('/api/developer/ws-debug/sessions', { method: 'POST' });
      const data = await res.json();
      setSessions([...sessions, data]);
      setSelectedSession(data);
      showNotification('success', 'WebSocket 调试会话已创建');
    } catch {
      showNotification('error', '创建会话失败');
    }
  };

  const sendMessage = async () => {
    if (!selectedSession || !newMessage) return;
    try {
      await fetch('/api/developer/ws-debug/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: selectedSession.id,
          message: newMessage,
          direction,
        }),
      });
      setNewMessage('');
      loadSessions();
    } catch {
      showNotification('error', '发送消息失败');
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
          <Radio className="w-5 h-5" />
          WebSocket 调试会话
        </h3>
        <button onClick={createSession} className="game-button game-button-primary flex items-center gap-2">
          <Plus className="w-4 h-4" />
          新建会话
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <div className="game-card">
          <h4 className="font-semibold text-text-primary mb-3">会话列表</h4>
          <div className="space-y-2 max-h-[400px] overflow-y-auto">
            {sessions.map((session) => (
              <button
                key={session.id}
                onClick={() => setSelectedSession(session)}
                className={clsx(
                  'w-full text-left p-3 rounded-lg transition-colors',
                  selectedSession?.id === session.id
                    ? 'bg-mc-green/20 border border-mc-green'
                    : 'bg-nether-700 hover:bg-nether-600'
                )}
              >
                <div className="font-mono text-sm truncate">{session.id.slice(0, 8)}...</div>
                <div className="text-xs text-text-muted mt-1">
                  {session.message_count} 消息 | {session.is_active ? '活跃' : '已关闭'}
                </div>
              </button>
            ))}
            {sessions.length === 0 && (
              <p className="text-text-muted text-center py-4">暂无会话</p>
            )}
          </div>
        </div>

        <div className="lg:col-span-2 game-card">
          <h4 className="font-semibold text-text-primary mb-3">消息记录</h4>
          <div className="space-y-2 max-h-[300px] overflow-y-auto mb-4">
            {messages.map((msg, idx) => (
              <div
                key={idx}
                className={clsx(
                  'p-2 rounded text-sm font-mono',
                  msg.direction === 'incoming' ? 'bg-blue-500/10 text-blue-400' : 'bg-mc-green/10 text-mc-green'
                )}
              >
                <span className="text-text-muted text-xs">
                  [{msg.direction}] {new Date(msg.timestamp).toLocaleTimeString()}
                </span>
                <div className="mt-1">{msg.content}</div>
              </div>
            ))}
            {messages.length === 0 && (
              <p className="text-text-muted text-center py-4">暂无消息</p>
            )}
          </div>

          {selectedSession && (
            <div className="flex gap-2">
              <select
                value={direction}
                onChange={(e) => setDirection(e.target.value as 'incoming' | 'outgoing')}
                className="bg-nether-700 border border-nether-600 rounded px-2 py-1 text-sm"
              >
                <option value="outgoing">发送</option>
                <option value="incoming">接收</option>
              </select>
              <input
                type="text"
                value={newMessage}
                onChange={(e) => setNewMessage(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && sendMessage()}
                placeholder="输入消息..."
                className="flex-1 bg-nether-700 border border-nether-600 rounded px-3 py-1 text-sm"
              />
              <button onClick={sendMessage} className="game-button flex items-center gap-1">
                <Send className="w-4 h-4" />
                发送
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function PluginHooksTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [hooks, setHooks] = useState<Hook[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({ name: '', hook_type: '', callback_url: '' });

  const loadHooks = () => {
    fetch('/api/developer/plugins/hooks')
      .then(res => res.json())
      .then(data => setHooks(data.hooks || []))
      .catch(console.error);
  };

  useEffect(() => {
    loadHooks();
  }, []);

  const createHook = async () => {
    try {
      await fetch('/api/developer/plugins/hooks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      });
      loadHooks();
      setShowForm(false);
      setFormData({ name: '', hook_type: '', callback_url: '' });
      showNotification('success', '插件钩子已创建');
    } catch {
      showNotification('error', '创建钩子失败');
    }
  };

  const deleteHook = async (id: string) => {
    try {
      await fetch(`/api/developer/plugins/hooks/${id}`, { method: 'DELETE' });
      loadHooks();
      showNotification('success', '插件钩子已删除');
    } catch {
      showNotification('error', '删除钩子失败');
    }
  };

  const reloadAll = async () => {
    try {
      const res = await fetch('/api/developer/plugins/reload', { method: 'POST' });
      const data = await res.json();
      showNotification(data.success ? 'success' : 'error', `已重载 ${data.reloaded_hooks?.length || 0} 个钩子`);
    } catch {
      showNotification('error', '重载失败');
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
          <Plug className="w-5 h-5" />
          插件热重载钩子
        </h3>
        <div className="flex gap-2">
          <button onClick={reloadAll} className="game-button flex items-center gap-2">
            <RefreshCw className="w-4 h-4" />
            重载全部
          </button>
          <button onClick={() => setShowForm(!showForm)} className="game-button game-button-primary flex items-center gap-2">
            <Plus className="w-4 h-4" />
            添加钩子
          </button>
        </div>
      </div>

      {showForm && (
        <div className="game-card">
          <h4 className="font-semibold text-text-primary mb-4">创建新钩子</h4>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <input
              type="text"
              placeholder="钩子名称"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
            <input
              type="text"
              placeholder="事件类型 (如 server.start)"
              value={formData.hook_type}
              onChange={(e) => setFormData({ ...formData, hook_type: e.target.value })}
              className="bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
            <input
              type="text"
              placeholder="回调 URL"
              value={formData.callback_url}
              onChange={(e) => setFormData({ ...formData, callback_url: e.target.value })}
              className="bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <button onClick={() => setShowForm(false)} className="game-button">取消</button>
            <button onClick={createHook} className="game-button game-button-primary">创建</button>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {hooks.map((hook) => (
          <div key={hook.id} className="game-card">
            <div className="flex items-start justify-between">
              <div>
                <h4 className="font-semibold text-text-primary">{hook.name}</h4>
                <p className="text-sm text-text-muted font-mono mt-1">{hook.hook_type}</p>
                {hook.callback_url && (
                  <p className="text-xs text-text-muted mt-1 truncate max-w-[200px]">{hook.callback_url}</p>
                )}
              </div>
              <div className="flex items-center gap-2">
                <span
                  className={clsx(
                    'px-2 py-1 rounded text-xs font-mono',
                    hook.enabled ? 'bg-mc-green/20 text-mc-green' : 'bg-nether-600 text-text-muted'
                  )}
                >
                  {hook.enabled ? '启用' : '禁用'}
                </span>
                <button onClick={() => deleteHook(hook.id)} className="p-1 hover:text-rust transition-colors">
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        ))}
        {hooks.length === 0 && (
          <div className="col-span-2 game-card text-center py-8 text-text-muted">
            暂无插件钩子
          </div>
        )}
      </div>
    </div>
  );
}

function RequestLogsTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [logs, setLogs] = useState<RequestLog[]>([]);
  const [stats, setStats] = useState<any>(null);
  const [filter, setFilter] = useState({ method: '', path: '' });

  const loadLogs = () => {
    const params = new URLSearchParams();
    if (filter.method) params.set('method', filter.method);
    if (filter.path) params.set('path', filter.path);
    params.set('limit', '50');

    fetch(`/api/developer/request-logs?${params}`)
      .then(res => res.json())
      .then(data => setLogs(data.logs || []))
      .catch(console.error);

    fetch('/api/developer/request-logs/stats')
      .then(res => res.json())
      .then(setStats)
      .catch(console.error);
  };

  useEffect(() => {
    loadLogs();
  }, [filter]);

  const clearLogs = async () => {
    try {
      await fetch('/api/developer/request-logs', { method: 'DELETE' });
      loadLogs();
      showNotification('success', '请求日志已清除');
    } catch {
      showNotification('error', '清除日志失败');
    }
  };

  const getStatusColor = (status: number) => {
    if (status >= 500) return 'text-rust';
    if (status >= 400) return 'text-yellow-500';
    if (status >= 200) return 'text-mc-green';
    return 'text-text-secondary';
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
          <FileText className="w-5 h-5" />
          API 请求日志
        </h3>
        <div className="flex gap-2">
          <button onClick={loadLogs} className="game-button flex items-center gap-2">
            <RefreshCw className="w-4 h-4" />
            刷新
          </button>
          <button onClick={clearLogs} className="game-button game-button-danger flex items-center gap-2">
            <Trash2 className="w-4 h-4" />
            清除
          </button>
        </div>
      </div>

      {stats && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="game-card text-center">
            <div className="text-2xl font-bold text-mc-green">{stats.total_requests}</div>
            <div className="text-sm text-text-muted">总请求数</div>
          </div>
          <div className="game-card text-center">
            <div className="text-2xl font-bold text-text-primary">{stats.average_duration_ms?.toFixed(2)}ms</div>
            <div className="text-sm text-text-muted">平均响应时间</div>
          </div>
          <div className="game-card text-center">
            <div className="text-2xl font-bold text-yellow-500">{stats.slow_requests?.length || 0}</div>
            <div className="text-sm text-text-muted">慢请求</div>
          </div>
          <div className="game-card text-center">
            <div className="text-2xl font-bold text-rust">{stats.error_requests?.length || 0}</div>
            <div className="text-sm text-text-muted">错误请求</div>
          </div>
        </div>
      )}

      <div className="game-card">
        <div className="flex gap-4 mb-4">
          <select
            value={filter.method}
            onChange={(e) => setFilter({ ...filter, method: e.target.value })}
            className="bg-nether-700 border border-nether-600 rounded px-3 py-2"
          >
            <option value="">所有方法</option>
            <option value="GET">GET</option>
            <option value="POST">POST</option>
            <option value="PUT">PUT</option>
            <option value="DELETE">DELETE</option>
          </select>
          <input
            type="text"
            placeholder="过滤路径..."
            value={filter.path}
            onChange={(e) => setFilter({ ...filter, path: e.target.value })}
            className="flex-1 bg-nether-700 border border-nether-600 rounded px-3 py-2"
          />
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-text-muted border-b border-nether-600">
                <th className="pb-2 font-mono">时间</th>
                <th className="pb-2 font-mono">方法</th>
                <th className="pb-2 font-mono">路径</th>
                <th className="pb-2 font-mono">状态</th>
                <th className="pb-2 font-mono">耗时</th>
                <th className="pb-2 font-mono">IP</th>
              </tr>
            </thead>
            <tbody>
              {logs.map((log) => (
                <tr key={log.id} className="border-b border-nether-700 hover:bg-nether-800">
                  <td className="py-2 font-mono text-xs">
                    {new Date(log.timestamp).toLocaleTimeString()}
                  </td>
                  <td className="py-2">
                    <span className={clsx(
                      'px-2 py-0.5 rounded text-xs font-mono',
                      log.method === 'GET' ? 'bg-blue-500/20 text-blue-400' :
                      log.method === 'POST' ? 'bg-mc-green/20 text-mc-green' :
                      log.method === 'PUT' ? 'bg-yellow-500/20 text-yellow-400' :
                      'bg-rust/20 text-rust'
                    )}>
                      {log.method}
                    </span>
                  </td>
                  <td className="py-2 font-mono text-xs truncate max-w-[200px]">{log.path}</td>
                  <td className={clsx('py-2 font-mono font-bold', getStatusColor(log.status))}>
                    {log.status}
                  </td>
                  <td className={clsx('py-2 font-mono', log.duration_ms > 1000 ? 'text-yellow-500' : '')}>
                    {log.duration_ms}ms
                  </td>
                  <td className="py-2 font-mono text-xs text-text-muted">{log.client_ip}</td>
                </tr>
              ))}
            </tbody>
          </table>
          {logs.length === 0 && (
            <p className="text-center py-8 text-text-muted">暂无请求日志</p>
          )}
        </div>
      </div>
    </div>
  );
}

function ProfilerTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [snapshots, setSnapshots] = useState<ProfilerSnapshot[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [profilerName, setProfilerName] = useState('profile-1');
  const [activeProfilerId, setActiveProfilerId] = useState<string | null>(null);

  const loadSnapshots = () => {
    fetch('/api/developer/profiler/snapshots')
      .then(res => res.json())
      .then(data => setSnapshots(data.snapshots || []))
      .catch(console.error);
  };

  useEffect(() => {
    loadSnapshots();
  }, []);

  const startProfiler = async () => {
    try {
      const res = await fetch('/api/developer/profiler/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: profilerName }),
      });
      const data = await res.json();
      setActiveProfilerId(data.profiler_id);
      setIsRunning(true);
      showNotification('success', '性能分析已启动');
    } catch {
      showNotification('error', '启动分析失败');
    }
  };

  const stopProfiler = async () => {
    try {
      await fetch(`/api/developer/profiler/stop?id=${activeProfilerId}`, { method: 'POST' });
      setIsRunning(false);
      setActiveProfilerId(null);
      loadSnapshots();
      showNotification('success', '性能分析已停止并保存快照');
    } catch {
      showNotification('error', '停止分析失败');
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
          <Activity className="w-5 h-5" />
          性能剖析工具
        </h3>
        <div className="flex gap-2 items-center">
          {!isRunning ? (
            <>
              <input
                type="text"
                value={profilerName}
                onChange={(e) => setProfilerName(e.target.value)}
                placeholder="分析名称"
                className="bg-nether-700 border border-nether-600 rounded px-3 py-2 w-40"
              />
              <button onClick={startProfiler} className="game-button game-button-primary flex items-center gap-2">
                <Play className="w-4 h-4" />
                开始分析
              </button>
            </>
          ) : (
            <button onClick={stopProfiler} className="game-button game-button-danger flex items-center gap-2">
              <Square className="w-4 h-4" />
              停止分析
            </button>
          )}
        </div>
      </div>

      {isRunning && (
        <div className="game-card bg-mc-green/10 border-mc-green">
          <div className="flex items-center gap-3">
            <div className="w-3 h-3 rounded-full bg-mc-green animate-pulse" />
            <span className="text-mc-green font-mono">分析运行中: {profilerName}</span>
          </div>
        </div>
      )}

      <div className="game-card">
        <h4 className="font-semibold text-text-primary mb-4">分析快照</h4>
        <div className="space-y-2">
          {snapshots.map((snapshot) => (
            <div key={snapshot.id} className="bg-nether-700 rounded-lg p-4 flex items-center justify-between">
              <div>
                <div className="font-mono text-text-primary">{snapshot.name}</div>
                <div className="text-xs text-text-muted mt-1">
                  {new Date(snapshot.timestamp).toLocaleString()}
                </div>
              </div>
              <div className="text-right">
                <div className="text-mc-green font-mono font-bold">
                  {snapshot.duration_ms.toFixed(2)}ms
                </div>
                <div className="text-xs text-text-muted">
                  {(snapshot.duration_ns / 1_000_000).toFixed(2)}ns
                </div>
              </div>
            </div>
          ))}
          {snapshots.length === 0 && (
            <p className="text-center py-8 text-text-muted">暂无分析快照</p>
          )}
        </div>
      </div>
    </div>
  );
}

function SdkTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [selectedLang, setSelectedLang] = useState('typescript');
  const [generatedCode, setGeneratedCode] = useState('');
  const [languages, setLanguages] = useState<any[]>([]);

  useEffect(() => {
    fetch('/api/developer/sdk/languages')
      .then(res => res.json())
      .then(setLanguages)
      .catch(console.error);
  }, []);

  const generateSdk = async () => {
    try {
      const res = await fetch('/api/developer/sdk/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ language: selectedLang, include_examples: true }),
      });
      const data = await res.json();
      setGeneratedCode(data.code);
      showNotification('success', `${data.language} SDK 已生成`);
    } catch {
      showNotification('error', '生成 SDK 失败');
    }
  };

  const copyCode = () => {
    navigator.clipboard.writeText(generatedCode);
    showNotification('success', '代码已复制到剪贴板');
  };

  const downloadCode = () => {
    const ext = languages.find(l => l.id === selectedLang)?.extension || 'txt';
    const blob = new Blob([generatedCode], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mc-server-client.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-4">
      <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
        <Box className="w-5 h-5" />
        SDK 代码生成
      </h3>

      <div className="game-card">
        <div className="flex flex-wrap gap-4 mb-4">
          <select
            value={selectedLang}
            onChange={(e) => setSelectedLang(e.target.value)}
            className="bg-nether-700 border border-nether-600 rounded px-4 py-2"
          >
            {languages.map((lang) => (
              <option key={lang.id} value={lang.id}>
                {lang.name} {lang.featured ? '⭐' : ''}
              </option>
            ))}
          </select>
          <button onClick={generateSdk} className="game-button game-button-primary flex items-center gap-2">
            <Zap className="w-4 h-4" />
            生成 SDK
          </button>
        </div>

        {generatedCode && (
          <div className="space-y-2">
            <div className="flex justify-end gap-2">
              <button onClick={copyCode} className="game-button flex items-center gap-2">
                <Copy className="w-4 h-4" />
                复制
              </button>
              <button onClick={downloadCode} className="game-button flex items-center gap-2">
                <Download className="w-4 h-4" />
                下载
              </button>
            </div>
            <pre className="bg-nether-900 p-4 rounded-lg overflow-auto max-h-[500px] text-sm font-mono">
              <code className="text-text-secondary">{generatedCode}</code>
            </pre>
          </div>
        )}

        {!generatedCode && (
          <div className="text-center py-12 text-text-muted">
            <Box className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>选择语言并点击"生成 SDK"开始</p>
          </div>
        )}
      </div>
    </div>
  );
}

function WebhookTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [testUrl, setTestUrl] = useState('');
  const [payload, setPayload] = useState('{"event": "test", "data": {}}');
  const [secret, setSecret] = useState('');
  const [testResult, setTestResult] = useState<any>(null);
  const [algorithms, setAlgorithms] = useState<any[]>([]);

  useEffect(() => {
    fetch('/api/developer/webhook/algorithms')
      .then(res => res.json())
      .then(setAlgorithms)
      .catch(console.error);
  }, []);

  const testWebhook = async () => {
    try {
      const res = await fetch('/api/developer/webhook/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          url: testUrl,
          payload: JSON.parse(payload),
          secret,
        }),
      });
      const data = await res.json();
      setTestResult(data);
      showNotification(data.success ? 'success' : 'error', data.success ? 'Webhook 测试成功' : `测试失败: ${data.error}`);
    } catch (e) {
      showNotification('error', '测试请求失败');
    }
  };

  const validateSignature = async () => {
    try {
      const res = await fetch('/api/developer/webhook/validate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          payload: payload,
          signature: 'test-signature',
          secret,
        }),
      });
      const data = await res.json();
      setTestResult(data);
    } catch {
      showNotification('error', '验证失败');
    }
  };

  return (
    <div className="space-y-4">
      <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
        <ShieldCheck className="w-5 h-5" />
        Webhook 签名验证
      </h3>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div className="game-card">
          <h4 className="font-semibold text-text-primary mb-4">测试 Webhook</h4>
          <div className="space-y-4">
            <input
              type="text"
              placeholder="Webhook URL"
              value={testUrl}
              onChange={(e) => setTestUrl(e.target.value)}
              className="w-full bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
            <div>
              <label className="text-sm text-text-muted mb-1 block">Payload (JSON)</label>
              <textarea
                value={payload}
                onChange={(e) => setPayload(e.target.value)}
                className="w-full h-32 bg-nether-700 border border-nether-600 rounded px-3 py-2 font-mono text-sm"
              />
            </div>
            <input
              type="password"
              placeholder="签名密钥"
              value={secret}
              onChange={(e) => setSecret(e.target.value)}
              className="w-full bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
            <button onClick={testWebhook} className="w-full game-button game-button-primary flex items-center justify-center gap-2">
              <Send className="w-4 h-4" />
              发送测试请求
            </button>
            <button onClick={validateSignature} className="w-full game-button flex items-center justify-center gap-2">
              <ShieldCheck className="w-4 h-4" />
              验证签名
            </button>
          </div>
        </div>

        <div className="game-card">
          <h4 className="font-semibold text-text-primary mb-4">支持算法</h4>
          <div className="space-y-2 mb-4">
            {algorithms.map((alg) => (
              <div key={alg.id} className="bg-nether-700 rounded-lg p-3">
                <div className="flex items-center justify-between">
                  <span className="font-mono text-text-primary">{alg.name}</span>
                  {alg.secure ? (
                    <span className="px-2 py-0.5 rounded text-xs bg-mc-green/20 text-mc-green">推荐</span>
                  ) : (
                    <span className="px-2 py-0.5 rounded text-xs bg-yellow-500/20 text-yellow-500">旧</span>
                  )}
                </div>
                <p className="text-xs text-text-muted mt-1">{alg.description}</p>
              </div>
            ))}
          </div>

          {testResult && (
            <div className="bg-nether-900 rounded-lg p-4">
              <h5 className="text-sm font-semibold mb-2">测试结果</h5>
              <pre className="text-xs font-mono overflow-auto">
                {JSON.stringify(testResult, null, 2)}
              </pre>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function RateLimitTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [status, setStatus] = useState<any>(null);
  const [config, setConfig] = useState({ requests_per_minute: 60, requests_per_hour: 1000 });

  const loadStatus = () => {
    fetch('/api/developer/rate-limit/status')
      .then(res => res.json())
      .then(setStatus)
      .catch(console.error);
  };

  useEffect(() => {
    loadStatus();
  }, []);

  const updateConfig = async () => {
    try {
      await fetch('/api/developer/rate-limit/config', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      loadStatus();
      showNotification('success', '速率限制配置已更新');
    } catch {
      showNotification('error', '更新配置失败');
    }
  };

  const resetAll = async () => {
    try {
      await fetch('/api/developer/rate-limit/clients', { method: 'DELETE' });
      loadStatus();
      showNotification('success', '所有速率限制已重置');
    } catch {
      showNotification('error', '重置失败');
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
          <Gauge className="w-5 h-5" />
          API 速率限制
        </h3>
        <button onClick={resetAll} className="game-button game-button-danger flex items-center gap-2">
          <RefreshCw className="w-4 h-4" />
          重置所有限制
        </button>
      </div>

      {status && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="game-card text-center">
            <div className={clsx(
              'text-2xl font-bold',
              status.enabled ? 'text-mc-green' : 'text-text-muted'
            )}>
              {status.enabled ? '启用' : '禁用'}
            </div>
            <div className="text-sm text-text-muted">状态</div>
          </div>
          <div className="game-card text-center">
            <div className="text-2xl font-bold text-text-primary">{status.requests_per_minute}</div>
            <div className="text-sm text-text-muted">每分钟请求</div>
          </div>
          <div className="game-card text-center">
            <div className="text-2xl font-bold text-text-primary">{status.requests_per_hour}</div>
            <div className="text-sm text-text-muted">每小时请求</div>
          </div>
          <div className="game-card text-center">
            <div className="text-2xl font-bold text-text-primary">{status.total_tracked_clients}</div>
            <div className="text-sm text-text-muted">跟踪的客户端</div>
          </div>
        </div>
      )}

      <div className="game-card">
        <h4 className="font-semibold text-text-primary mb-4">配置</h4>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label className="text-sm text-text-muted mb-1 block">每分钟请求数</label>
            <input
              type="number"
              value={config.requests_per_minute}
              onChange={(e) => setConfig({ ...config, requests_per_minute: parseInt(e.target.value) })}
              className="w-full bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
          </div>
          <div>
            <label className="text-sm text-text-muted mb-1 block">每小时请求数</label>
            <input
              type="number"
              value={config.requests_per_hour}
              onChange={(e) => setConfig({ ...config, requests_per_hour: parseInt(e.target.value) })}
              className="w-full bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
          </div>
          <div className="flex items-end">
            <button onClick={updateConfig} className="w-full game-button game-button-primary">
              保存配置
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function EventsTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [subscriptions, setSubscriptions] = useState<EventSub[]>([]);
  const [eventTypes, setEventTypes] = useState<any[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({ name: '', event_type: '', callback_url: '', secret: '' });

  useEffect(() => {
    loadSubscriptions();
    loadEventTypes();
  }, []);

  const loadSubscriptions = () => {
    fetch('/api/developer/events/subscriptions')
      .then(res => res.json())
      .then(data => setSubscriptions(data.subscriptions || []))
      .catch(console.error);
  };

  const loadEventTypes = () => {
    fetch('/api/developer/events/types')
      .then(res => res.json())
      .then(setEventTypes)
      .catch(console.error);
  };

  const createSubscription = async () => {
    try {
      await fetch('/api/developer/events/subscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData),
      });
      loadSubscriptions();
      setShowForm(false);
      setFormData({ name: '', event_type: '', callback_url: '', secret: '' });
      showNotification('success', '事件订阅已创建');
    } catch {
      showNotification('error', '创建订阅失败');
    }
  };

  const deleteSubscription = async (id: string) => {
    try {
      await fetch(`/api/developer/events/subscribe/${id}`, { method: 'DELETE' });
      loadSubscriptions();
      showNotification('success', '订阅已删除');
    } catch {
      showNotification('error', '删除订阅失败');
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
          <Bell className="w-5 h-5" />
          事件订阅系统
        </h3>
        <button onClick={() => setShowForm(!showForm)} className="game-button game-button-primary flex items-center gap-2">
          <Plus className="w-4 h-4" />
          添加订阅
        </button>
      </div>

      <div className="game-card">
        <h4 className="font-semibold text-text-primary mb-4">可用事件类型</h4>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
          {eventTypes.map((type) => (
            <div key={type.id} className="bg-nether-700 rounded px-3 py-2 text-sm">
              <div className="font-mono text-mc-green">{type.id}</div>
              <div className="text-xs text-text-muted">{type.name}</div>
            </div>
          ))}
        </div>
      </div>

      {showForm && (
        <div className="game-card">
          <h4 className="font-semibold text-text-primary mb-4">创建新订阅</h4>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <input
              type="text"
              placeholder="订阅名称"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
            <select
              value={formData.event_type}
              onChange={(e) => setFormData({ ...formData, event_type: e.target.value })}
              className="bg-nether-700 border border-nether-600 rounded px-3 py-2"
            >
              <option value="">选择事件类型</option>
              {eventTypes.map((type) => (
                <option key={type.id} value={type.id}>{type.name} ({type.id})</option>
              ))}
            </select>
            <input
              type="text"
              placeholder="回调 URL"
              value={formData.callback_url}
              onChange={(e) => setFormData({ ...formData, callback_url: e.target.value })}
              className="bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
            <input
              type="password"
              placeholder="签名密钥 (可选)"
              value={formData.secret}
              onChange={(e) => setFormData({ ...formData, secret: e.target.value })}
              className="bg-nether-700 border border-nether-600 rounded px-3 py-2"
            />
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <button onClick={() => setShowForm(false)} className="game-button">取消</button>
            <button onClick={createSubscription} className="game-button game-button-primary">创建</button>
          </div>
        </div>
      )}

      <div className="space-y-2">
        {subscriptions.map((sub) => (
          <div key={sub.id} className="game-card">
            <div className="flex items-start justify-between">
              <div>
                <div className="font-semibold text-text-primary">{sub.name}</div>
                <div className="text-sm font-mono text-mc-green mt-1">{sub.event_type}</div>
                <div className="text-xs text-text-muted mt-1 truncate max-w-[300px]">{sub.callback_url}</div>
              </div>
              <div className="flex items-center gap-2">
                {sub.has_secret && <Key className="w-4 h-4 text-text-muted" />}
                <span className={clsx(
                  'px-2 py-1 rounded text-xs',
                  sub.enabled ? 'bg-mc-green/20 text-mc-green' : 'bg-nether-600 text-text-muted'
                )}>
                  {sub.enabled ? '启用' : '禁用'}
                </span>
                <button onClick={() => deleteSubscription(sub.id)} className="p-1 hover:text-rust">
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        ))}
        {subscriptions.length === 0 && (
          <div className="game-card text-center py-8 text-text-muted">
            暂无事件订阅
          </div>
        )}
      </div>
    </div>
  );
}

function ConsoleTab({ showNotification }: { showNotification: (type: 'success' | 'error', message: string) => void }) {
  const [command, setCommand] = useState('');
  const [history, setHistory] = useState<ConsoleEntry[]>([]);
  const [cheatsheet, setCheatsheet] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadHistory();
    loadCheatsheet();
  }, []);

  const loadHistory = () => {
    fetch('/api/developer/console/history?limit=20')
      .then(res => res.json())
      .then(data => setHistory(data.entries || []))
      .catch(console.error);
  };

  const loadCheatsheet = () => {
    fetch('/api/developer/console/cheatsheet')
      .then(res => res.json())
      .then(setCheatsheet)
      .catch(console.error);
  };

  const executeCommand = async () => {
    if (!command.trim() || isLoading) return;
    setIsLoading(true);
    try {
      const res = await fetch('/api/developer/console/execute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: command.trim() }),
      });
      const data = await res.json();
      setHistory([...history, {
        id: data.id,
        command: data.command,
        output: data.output,
        duration_ms: data.duration_ms,
        executed_at: new Date().toISOString(),
      }]);
      setCommand('');
      showNotification('success', '命令执行完成');
    } catch {
      showNotification('error', '执行命令失败');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <h3 className="font-display text-lg font-semibold text-text-primary flex items-center gap-2">
        <Terminal className="w-5 h-5" />
        插件开发者控制台
      </h3>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <div className="lg:col-span-2 game-card">
          <h4 className="font-semibold text-text-primary mb-4">命令执行</h4>
          <div className="flex gap-2 mb-4">
            <input
              type="text"
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && executeCommand()}
              placeholder="输入服务器命令..."
              className="flex-1 bg-nether-900 border border-nether-600 rounded px-4 py-3 font-mono text-sm"
              disabled={isLoading}
            />
            <button
              onClick={executeCommand}
              disabled={isLoading}
              className="game-button game-button-primary flex items-center gap-2"
            >
              {isLoading ? (
                <RefreshCw className="w-4 h-4 animate-spin" />
              ) : (
                <Send className="w-4 h-4" />
              )}
              执行
            </button>
          </div>

          <div className="bg-nether-900 rounded-lg p-4 max-h-[400px] overflow-y-auto space-y-2">
            {history.map((entry) => (
              <div key={entry.id}>
                <div className="flex items-center gap-2 text-mc-green font-mono text-sm">
                  <span className="text-text-muted">{'>'}</span>
                  <span>{entry.command}</span>
                  <span className="text-text-muted text-xs">({entry.duration_ms}ms)</span>
                </div>
                <pre className="text-text-secondary font-mono text-sm mt-1 pl-4 whitespace-pre-wrap">
                  {entry.output || '(无输出)'}
                </pre>
              </div>
            ))}
            {history.length === 0 && (
              <p className="text-text-muted text-center py-8">暂无命令历史</p>
            )}
          </div>
        </div>

        <div className="game-card">
          <h4 className="font-semibold text-text-primary mb-4">命令速查表</h4>
          <div className="space-y-1 max-h-[500px] overflow-y-auto">
            {cheatsheet.map((cmd, idx) => (
              <div key={idx} className="bg-nether-700 rounded p-2">
                <div className="font-mono text-xs text-mc-green">{cmd.command}</div>
                <div className="text-xs text-text-muted mt-1">{cmd.description}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Search, Filter, Download, Trash2, AlertTriangle, Activity,
  Clock, ChevronDown, ChevronUp,
  Info, AlertCircle, XCircle, Terminal, BookOpen, Settings,
  RefreshCw, Check, BarChart3, Sparkles,
  Layers, Zap, Shield
} from 'lucide-react';
import type {
  LogEntry, LogLevel, DashboardSummary, DiagnosticResult,
  KnowledgeBaseEntry, BottleneckReport, CustomParseRule
} from './types';
import {
  searchLogs, getDashboard, getDiagnostic, getKnowledgeBase,
  detectBottlenecks, exportLogs, getCustomRules, addCustomRule,
  deleteCustomRule, toggleCustomRule, clearLogs, analyzeLog, lookupKnowledge
} from './api';

type Tab = 'logs' | 'dashboard' | 'analysis' | 'knowledge' | 'rules';

const LEVEL_COLORS: Record<LogLevel, { bg: string; text: string; border: string }> = {
  fatal: { bg: 'bg-red-900/50', text: 'text-red-200', border: 'border-red-500' },
  error: { bg: 'bg-red-900/30', text: 'text-red-300', border: 'border-red-400' },
  warn: { bg: 'bg-yellow-900/30', text: 'text-yellow-300', border: 'border-yellow-400' },
  info: { bg: 'bg-blue-900/30', text: 'text-blue-300', border: 'border-blue-400' },
  debug: { bg: 'bg-gray-700/30', text: 'text-gray-300', border: 'border-gray-500' },
  trace: { bg: 'bg-gray-800/30', text: 'text-gray-400', border: 'border-gray-600' },
};

const LEVEL_ICONS: Record<LogLevel, React.ReactNode> = {
  fatal: <XCircle className="w-4 h-4" />,
  error: <AlertCircle className="w-4 h-4" />,
  warn: <AlertTriangle className="w-4 h-4" />,
  info: <Info className="w-4 h-4" />,
  debug: <Settings className="w-4 h-4" />,
  trace: <Terminal className="w-4 h-4" />,
};

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleString('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

function highlightText(text: string, query: string): React.ReactNode {
  if (!query.trim()) return text;
  
  const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
  const parts = text.split(regex);
  
  return parts.map((part, i) => 
    regex.test(part) ? (
      <mark key={i} className="bg-yellow-500/40 text-yellow-200 rounded px-0.5">
        {part}
      </mark>
    ) : part
  );
}

function LogEntryRow({ 
  entry, 
  query,
  onSelect,
  selected 
}: { 
  entry: LogEntry; 
  query: string;
  onSelect: (id: string) => void;
  selected: boolean;
}) {
  const colors = LEVEL_COLORS[entry.level];
  const [expanded, setExpanded] = useState(false);
  
  return (
    <div
      className={`${colors.bg} ${selected ? 'ring-2 ring-primary' : ''} rounded-lg border-l-4 ${colors.border} overflow-hidden transition-all`}
    >
      <div 
        className="flex items-start gap-3 p-3 cursor-pointer hover:bg-white/5"
        onClick={() => setExpanded(!expanded)}
      >
        <span className={`${colors.text} mt-0.5`}>
          {LEVEL_ICONS[entry.level]}
        </span>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 text-xs text-gray-400">
            <Clock className="w-3 h-3" />
            <span>{formatTimestamp(entry.timestamp)}</span>
            <span className="text-gray-600">|</span>
            <span className="text-gray-500">{entry.source}</span>
          </div>
          <p className="text-sm text-gray-200 mt-1 font-mono break-all">
            {highlightText(entry.message, query)}
          </p>
          {entry.metadata.performance_data && (
            <div className="flex gap-3 mt-2 text-xs">
              <span className={entry.metadata.performance_data.tps < 18 ? 'text-red-400' : 'text-green-400'}>
                TPS: {entry.metadata.performance_data.tps.toFixed(2)}
              </span>
              <span className={entry.metadata.performance_data.mspt > 50 ? 'text-yellow-400' : 'text-green-400'}>
                MSPT: {entry.metadata.performance_data.mspt.toFixed(2)}ms
              </span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={(e) => { e.stopPropagation(); onSelect(entry.id); }}
            className="p-1.5 rounded hover:bg-white/10 text-gray-400"
            title="查看详情"
          >
            <Info className="w-4 h-4" />
          </button>
          {expanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
        </div>
      </div>
      
      {expanded && entry.metadata.stack_trace && (
        <div className="px-4 pb-3">
          <div className="bg-black/40 rounded p-3 font-mono text-xs overflow-x-auto">
            <p className="text-red-400 mb-2 font-bold">Stack Trace:</p>
            {entry.metadata.stack_trace.map((frame, i) => (
              <p key={i} className="text-orange-300">
                at {frame.class}.{frame.method} ({frame.file}:{frame.line})
              </p>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function DashboardView({ dashboard }: { dashboard: DashboardSummary }) {
  return (
    <div className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="game-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-text-secondary text-sm">总日志数</p>
              <p className="text-2xl font-bold text-text-primary">{dashboard.total_logs.toLocaleString()}</p>
            </div>
            <Layers className="w-8 h-8 text-primary" />
          </div>
        </div>
        
        <div className="game-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-text-secondary text-sm">错误数</p>
              <p className="text-2xl font-bold text-red-400">{dashboard.error_count.toLocaleString()}</p>
            </div>
            <AlertCircle className="w-8 h-8 text-red-400" />
          </div>
        </div>
        
        <div className="game-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-text-secondary text-sm">警告数</p>
              <p className="text-2xl font-bold text-yellow-400">{dashboard.warn_count.toLocaleString()}</p>
            </div>
            <AlertTriangle className="w-8 h-8 text-yellow-400" />
          </div>
        </div>
        
        <div className="game-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-text-secondary text-sm">错误率</p>
              <p className="text-2xl font-bold text-red-400">{dashboard.error_rate.toFixed(2)}%</p>
            </div>
            <Activity className="w-8 h-8 text-red-400" />
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="game-card">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 rounded-lg bg-primary/20">
              <Zap className="w-5 h-5 text-primary" />
            </div>
            <div>
              <p className="text-text-secondary text-sm">平均 TPS</p>
              <p className={`text-xl font-bold ${dashboard.average_tps < 18 ? 'text-red-400' : 'text-green-400'}`}>
                {dashboard.average_tps.toFixed(2)}
              </p>
            </div>
          </div>
        </div>
        
        <div className="game-card">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 rounded-lg bg-yellow-500/20">
              <Clock className="w-5 h-5 text-yellow-400" />
            </div>
            <div>
              <p className="text-text-secondary text-sm">平均 MSPT</p>
              <p className={`text-xl font-bold ${dashboard.average_mspt > 50 ? 'text-yellow-400' : 'text-green-400'}`}>
                {dashboard.average_mspt.toFixed(2)}ms
              </p>
            </div>
          </div>
        </div>
        
        <div className="game-card">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 rounded-lg bg-blue-500/20">
              <BarChart3 className="w-5 h-5 text-blue-400" />
            </div>
            <div>
              <p className="text-text-secondary text-sm">运行时间</p>
              <p className="text-xl font-bold text-text-primary">
                {dashboard.uptime_hours.toFixed(1)}h
              </p>
            </div>
          </div>
        </div>
      </div>

      {dashboard.top_error && (
        <div className="game-card">
          <h3 className="text-lg font-bold text-text-primary mb-3">最常见错误</h3>
          <div className="bg-red-900/20 rounded-lg p-4 border border-red-800">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-semibold text-red-300">{dashboard.top_error.error_type}</p>
                <p className="text-sm text-gray-400 mt-1">{dashboard.top_error.description}</p>
              </div>
              <div className="text-right">
                <p className="text-2xl font-bold text-red-400">{dashboard.top_error.count}</p>
                <p className="text-xs text-gray-500">次</p>
              </div>
            </div>
            {dashboard.top_error.solution && (
              <div className="mt-3 pt-3 border-t border-red-800">
                <p className="text-sm text-green-400">解决方案: {dashboard.top_error.solution}</p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function AnalysisView({ 
  diagnostic, 
  bottlenecks,
  onLookupKnowledge 
}: { 
  diagnostic: DiagnosticResult | null;
  bottlenecks: BottleneckReport[];
  onLookupKnowledge: (msg: string) => void;
}) {
  if (!diagnostic) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="w-6 h-6 animate-spin text-primary" />
        <span className="ml-2">加载诊断数据...</span>
      </div>
    );
  }

  const getSeverityBadge = (severity: string) => {
    const styles = {
      critical: 'bg-red-900/50 text-red-300 border-red-500',
      error: 'bg-red-900/30 text-red-300 border-red-600',
      warning: 'bg-yellow-900/30 text-yellow-300 border-yellow-600',
      info: 'bg-blue-900/30 text-blue-300 border-blue-600',
    };
    return `px-2 py-0.5 rounded text-xs border ${styles[severity as keyof typeof styles] || styles.info}`;
  };

  return (
    <div className="space-y-6">
      <div className="game-card">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-bold text-text-primary flex items-center gap-2">
            <Shield className="w-5 h-5 text-primary" />
            服务器健康状态
          </h3>
          <span className={`text-2xl font-bold ${
            diagnostic.overall_health.status === 'healthy' ? 'text-green-400' :
            diagnostic.overall_health.status === 'degraded' ? 'text-yellow-400' :
            diagnostic.overall_health.status === 'unhealthy' ? 'text-orange-400' :
            'text-red-400'
          }`}>
            {diagnostic.overall_health.score.toFixed(0)}分
          </span>
        </div>
        <div className="w-full bg-gray-700 rounded-full h-2 mb-2">
          <div 
            className={`h-2 rounded-full transition-all ${
              diagnostic.overall_health.score >= 80 ? 'bg-green-500' :
              diagnostic.overall_health.score >= 60 ? 'bg-yellow-500' :
              diagnostic.overall_health.score >= 30 ? 'bg-orange-500' :
              'bg-red-500'
            }`}
            style={{ width: `${diagnostic.overall_health.score}%` }}
          />
        </div>
        <p className="text-sm text-gray-400">{diagnostic.overall_health.summary}</p>
      </div>

      {diagnostic.issues.length > 0 && (
        <div className="game-card">
          <h3 className="text-lg font-bold text-text-primary mb-4 flex items-center gap-2">
            <AlertTriangle className="w-5 h-5 text-yellow-400" />
            检测到的问题 ({diagnostic.issues.length})
          </h3>
          <div className="space-y-3">
            {diagnostic.issues.map((issue) => (
              <div 
                key={issue.id}
                className="bg-gray-800/50 rounded-lg p-4 border-l-4 border-yellow-500"
              >
                <div className="flex items-start justify-between">
                  <div>
                    <div className="flex items-center gap-2">
                      <h4 className="font-semibold text-text-primary">{issue.title}</h4>
                      {getSeverityBadge(issue.severity)}
                    </div>
                    <p className="text-sm text-gray-400 mt-1">{issue.description}</p>
                    <p className="text-xs text-gray-500 mt-2">
                      影响日志: {issue.affected_logs} | 
                      首次: {new Date(issue.first_occurrence).toLocaleString()} |
                      最近: {new Date(issue.last_occurrence).toLocaleString()}
                    </p>
                  </div>
                  <button
                    onClick={() => onLookupKnowledge(issue.description)}
                    className="p-2 rounded-lg hover:bg-white/10 text-primary"
                    title="查找知识库"
                  >
                    <BookOpen className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {bottlenecks.length > 0 && (
        <div className="game-card">
          <h3 className="text-lg font-bold text-text-primary mb-4 flex items-center gap-2">
            <Zap className="w-5 h-5 text-orange-400" />
            性能瓶颈分析
          </h3>
          <div className="space-y-3">
            {bottlenecks.map((bottleneck, i) => (
              <div 
                key={i}
                className="bg-gray-800/50 rounded-lg p-4"
              >
                <div className="flex items-start gap-3">
                  <div className={`p-2 rounded-lg ${
                    bottleneck.severity === 'critical' ? 'bg-red-900/30' :
                    bottleneck.severity === 'error' ? 'bg-orange-900/30' :
                    'bg-yellow-900/30'
                  }`}>
                    <Activity className={`w-5 h-5 ${
                      bottleneck.severity === 'critical' ? 'text-red-400' :
                      bottleneck.severity === 'error' ? 'text-orange-400' :
                      'text-yellow-400'
                    }`} />
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <h4 className="font-semibold text-text-primary">{bottleneck.description}</h4>
                      {getSeverityBadge(bottleneck.severity)}
                    </div>
                    <div className="mt-2">
                      <p className="text-xs text-gray-400 mb-1">可能原因:</p>
                      <ul className="text-sm text-gray-300 space-y-1">
                        {bottleneck.possible_causes.map((cause, j) => (
                          <li key={j} className="flex items-center gap-2">
                            <span className="w-1 h-1 bg-gray-500 rounded-full" />
                            {cause}
                          </li>
                        ))}
                      </ul>
                    </div>
                    <div className="mt-3">
                      <p className="text-xs text-green-400 mb-1">建议修复:</p>
                      <ul className="text-sm text-green-300 space-y-1">
                        {bottleneck.suggested_fixes.map((fix, j) => (
                          <li key={j} className="flex items-center gap-2">
                            <Sparkles className="w-3 h-3" />
                            {fix}
                          </li>
                        ))}
                      </ul>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {diagnostic.recommendations.length > 0 && (
        <div className="game-card">
          <h3 className="text-lg font-bold text-text-primary mb-4 flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-primary" />
            优化建议
          </h3>
          <ul className="space-y-2">
            {diagnostic.recommendations.map((rec, i) => (
              <li key={i} className="flex items-start gap-2 text-text-secondary">
                <Check className="w-4 h-4 text-green-400 mt-0.5 flex-shrink-0" />
                {rec}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

function KnowledgeView({ knowledge }: { knowledge: KnowledgeBaseEntry[] }) {
  const [expanded, setExpanded] = useState<string | null>(null);
  
  const getSeverityBadge = (severity: number) => {
    if (severity >= 8) return 'bg-red-900/50 text-red-300';
    if (severity >= 6) return 'bg-orange-900/50 text-orange-300';
    if (severity >= 4) return 'bg-yellow-900/50 text-yellow-300';
    return 'bg-blue-900/50 text-blue-300';
  };

  return (
    <div className="space-y-4">
      <div className="game-card">
        <h3 className="text-lg font-bold text-text-primary mb-4 flex items-center gap-2">
          <BookOpen className="w-5 h-5 text-primary" />
          常见错误知识库 ({knowledge.length} 条)
        </h3>
        <div className="space-y-3">
          {knowledge.map((entry) => (
            <div 
              key={entry.id}
              className={`bg-gray-800/50 rounded-lg overflow-hidden transition-all ${
                expanded === entry.id ? 'ring-2 ring-primary' : ''
              }`}
            >
              <div 
                className="p-4 cursor-pointer hover:bg-white/5"
                onClick={() => setExpanded(expanded === entry.id ? null : entry.id)}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <h4 className="font-semibold text-text-primary">{entry.title}</h4>
                    <span className={`px-2 py-0.5 rounded text-xs ${getSeverityBadge(entry.severity)}`}>
                      严重度: {entry.severity}
                    </span>
                  </div>
                  {expanded === entry.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                </div>
                <p className="text-sm text-gray-400 mt-1">{entry.description}</p>
                <div className="flex gap-2 mt-2">
                  {entry.tags.map((tag, i) => (
                    <span key={i} className="px-2 py-0.5 bg-gray-700 rounded text-xs text-gray-400">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
              
              {expanded === entry.id && (
                <div className="px-4 pb-4 border-t border-gray-700 pt-4 space-y-4">
                  <div>
                    <p className="text-sm font-semibold text-red-400 mb-1">原因分析:</p>
                    <pre className="text-sm text-gray-300 whitespace-pre-wrap">{entry.cause}</pre>
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-green-400 mb-1">解决方案:</p>
                    <pre className="text-sm text-gray-300 whitespace-pre-wrap">{entry.solution}</pre>
                  </div>
                  {entry.prevention.length > 0 && (
                    <div>
                      <p className="text-sm font-semibold text-blue-400 mb-1">预防措施:</p>
                      <ul className="text-sm text-gray-300 space-y-1">
                        {entry.prevention.map((item, i) => (
                          <li key={i} className="flex items-center gap-2">
                            <Shield className="w-3 h-3 text-blue-400" />
                            {item}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function RulesView({ rules, onRefresh }: { rules: CustomParseRule[]; onRefresh: () => void }) {
  const [showAdd, setShowAdd] = useState(false);
  const [newRule, setNewRule] = useState<Partial<CustomParseRule>>({
    name: '',
    pattern: '',
    enabled: true,
    priority: 1,
    fields: [],
  });

  const handleAdd = async () => {
    if (!newRule.name || !newRule.pattern) return;
    
    try {
      await addCustomRule({
        id: `rule_${Date.now()}`,
        name: newRule.name,
        pattern: newRule.pattern,
        enabled: newRule.enabled ?? true,
        priority: newRule.priority ?? 1,
        fields: newRule.fields ?? [],
      });
      setShowAdd(false);
      setNewRule({ name: '', pattern: '', enabled: true, priority: 1, fields: [] });
      onRefresh();
    } catch (error) {
      console.error('Failed to add rule:', error);
    }
  };

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await toggleCustomRule(id, !enabled);
      onRefresh();
    } catch (error) {
      console.error('Failed to toggle rule:', error);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定要删除这个解析规则吗?')) return;
    
    try {
      await deleteCustomRule(id);
      onRefresh();
    } catch (error) {
      console.error('Failed to delete rule:', error);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-bold text-text-primary flex items-center gap-2">
          <Settings className="w-5 h-5 text-primary" />
          自定义解析规则 ({rules.length})
        </h3>
        <button
          onClick={() => setShowAdd(!showAdd)}
          className="game-button"
        >
          {showAdd ? '取消' : '添加规则'}
        </button>
      </div>

      {showAdd && (
        <div className="game-card">
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-text-secondary mb-1">规则名称</label>
              <input
                type="text"
                value={newRule.name}
                onChange={(e) => setNewRule({ ...newRule, name: e.target.value })}
                className="game-input w-full"
                placeholder="输入规则名称"
              />
            </div>
            <div>
              <label className="block text-sm text-text-secondary mb-1">正则表达式</label>
              <input
                type="text"
                value={newRule.pattern}
                onChange={(e) => setNewRule({ ...newRule, pattern: e.target.value })}
                className="game-input w-full font-mono"
                placeholder="^\[(\d{4}-\d{2}-\d{2})\].*$"
              />
            </div>
            <div>
              <label className="block text-sm text-text-secondary mb-1">优先级</label>
              <input
                type="number"
                value={newRule.priority}
                onChange={(e) => setNewRule({ ...newRule, priority: parseInt(e.target.value) || 1 })}
                className="game-input w-24"
                min={1}
              />
            </div>
            <button onClick={handleAdd} className="game-button-primary">
              保存规则
            </button>
          </div>
        </div>
      )}

      <div className="space-y-2">
        {rules.map((rule) => (
          <div key={rule.id} className="game-card flex items-center justify-between">
            <div className="flex items-center gap-3">
              <button
                onClick={() => handleToggle(rule.id, rule.enabled)}
                className={`w-10 h-6 rounded-full transition-colors ${
                  rule.enabled ? 'bg-primary' : 'bg-gray-600'
                }`}
              >
                <div className={`w-4 h-4 bg-white rounded-full transition-transform ${
                  rule.enabled ? 'translate-x-5' : 'translate-x-1'
                }`} />
              </button>
              <div>
                <p className="font-semibold text-text-primary">{rule.name}</p>
                <p className="text-xs text-gray-500 font-mono">{rule.pattern}</p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-gray-500">优先级: {rule.priority}</span>
              <button
                onClick={() => handleDelete(rule.id)}
                className="p-2 rounded hover:bg-red-900/30 text-red-400"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          </div>
        ))}
        
        {rules.length === 0 && (
          <div className="text-center py-8 text-gray-500">
            暂无自定义解析规则
          </div>
        )}
      </div>
    </div>
  );
}

export function Logs() {
  const [activeTab, setActiveTab] = useState<Tab>('logs');
  const [searchQuery, setSearchQuery] = useState('');
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [totalLogs, setTotalLogs] = useState(0);
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedLevels, setSelectedLevels] = useState<LogLevel[]>([]);
  const [showFilters, setShowFilters] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [dashboard, setDashboard] = useState<DashboardSummary | null>(null);
  const [diagnostic, setDiagnostic] = useState<DiagnosticResult | null>(null);
  const [bottlenecks, setBottlenecks] = useState<BottleneckReport[]>([]);
  const [knowledge, setKnowledge] = useState<KnowledgeBaseEntry[]>([]);
  const [rules, setRules] = useState<CustomParseRule[]>([]);
  const [selectedLogId, setSelectedLogId] = useState<string | null>(null);
  const [selectedLogAnalysis, setSelectedLogAnalysis] = useState<any>(null);
  const [knowledgeLookupResult, setKnowledgeLookupResult] = useState<KnowledgeBaseEntry | null>(null);
  const logsContainerRef = useRef<HTMLDivElement>(null);

  const loadLogs = useCallback(async (page = 1) => {
    setIsLoading(true);
    try {
      const levels = selectedLevels.length > 0 ? selectedLevels.join(',') : undefined;
      const result = await searchLogs({
        query: searchQuery,
        page,
        per_page: 50,
        levels,
      });
      setLogs(result.entries);
      setTotalLogs(result.total);
      setCurrentPage(page);
    } catch (error) {
      console.error('Failed to load logs:', error);
    } finally {
      setIsLoading(false);
    }
  }, [searchQuery, selectedLevels]);

  const loadDashboard = useCallback(async () => {
    try {
      const data = await getDashboard();
      setDashboard(data);
    } catch (error) {
      console.error('Failed to load dashboard:', error);
    }
  }, []);

  const loadDiagnostic = useCallback(async () => {
    try {
      const [diag, bots] = await Promise.all([
        getDiagnostic(),
        detectBottlenecks(),
      ]);
      setDiagnostic(diag);
      setBottlenecks(bots);
    } catch (error) {
      console.error('Failed to load diagnostic:', error);
    }
  }, []);

  const loadKnowledge = useCallback(async () => {
    try {
      const data = await getKnowledgeBase();
      setKnowledge(data);
    } catch (error) {
      console.error('Failed to load knowledge:', error);
    }
  }, []);

  const loadRules = useCallback(async () => {
    try {
      const data = await getCustomRules();
      setRules(data);
    } catch (error) {
      console.error('Failed to load rules:', error);
    }
  }, []);

  useEffect(() => {
    if (activeTab === 'logs') {
      loadLogs(currentPage);
    } else if (activeTab === 'dashboard') {
      loadDashboard();
    } else if (activeTab === 'analysis') {
      loadDiagnostic();
    } else if (activeTab === 'knowledge') {
      loadKnowledge();
    } else if (activeTab === 'rules') {
      loadRules();
    }
  }, [activeTab, loadLogs, loadDashboard, loadDiagnostic, loadKnowledge, loadRules, currentPage]);

  const handleLevelToggle = (level: LogLevel) => {
    setSelectedLevels(prev => 
      prev.includes(level) 
        ? prev.filter(l => l !== level)
        : [...prev, level]
    );
  };

  const handleExport = async (format: 'json' | 'csv' | 'text' | 'html') => {
    try {
      const content = await exportLogs({
        format,
        filters: {
          levels: selectedLevels.length > 0 ? selectedLevels : undefined,
          stack_trace_only: false,
          performance_issues_only: false,
        },
        include_metadata: true,
        include_context: true,
        max_entries: 1000,
      });

      const blob = new Blob([content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `minecraft-logs-${Date.now()}.${format}`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Export failed:', error);
    }
  };

  const handleClearLogs = async () => {
    if (!confirm('确定要清空所有日志吗?')) return;
    
    try {
      await clearLogs();
      setLogs([]);
      setTotalLogs(0);
    } catch (error) {
      console.error('Clear logs failed:', error);
    }
  };

  const handleSelectLog = async (id: string) => {
    setSelectedLogId(id);
    try {
      const analysis = await analyzeLog(id);
      setSelectedLogAnalysis(analysis);
    } catch (error) {
      console.error('Failed to analyze log:', error);
    }
  };

  const handleKnowledgeLookup = async (msg: string) => {
    try {
      const result = await lookupKnowledge(msg);
      setKnowledgeLookupResult(result);
    } catch (error) {
      console.error('Knowledge lookup failed:', error);
    }
  };

  const tabs = [
    { id: 'logs' as Tab, label: '日志查看', icon: <Terminal className="w-4 h-4" /> },
    { id: 'dashboard' as Tab, label: '统计仪表盘', icon: <BarChart3 className="w-4 h-4" /> },
    { id: 'analysis' as Tab, label: '诊断分析', icon: <Activity className="w-4 h-4" /> },
    { id: 'knowledge' as Tab, label: '知识库', icon: <BookOpen className="w-4 h-4" /> },
    { id: 'rules' as Tab, label: '解析规则', icon: <Settings className="w-4 h-4" /> },
  ];

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-2xl font-bold text-text-primary">日志分析与诊断</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={() => activeTab === 'logs' && loadLogs(currentPage)}
            className="game-button"
            title="刷新"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div className="flex border-b border-gray-700 mb-4 overflow-x-auto">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-2 px-4 py-2 text-sm whitespace-nowrap transition-colors ${
              activeTab === tab.id
                ? 'text-primary border-b-2 border-primary bg-primary/10'
                : 'text-gray-400 hover:text-text-primary'
            }`}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {activeTab === 'logs' && (
        <>
          <div className="flex flex-col md:flex-row gap-3 mb-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && loadLogs()}
                placeholder="搜索日志内容..."
                className="game-input w-full pl-10"
              />
            </div>
            
            <div className="flex gap-2">
              <button
                onClick={() => setShowFilters(!showFilters)}
                className={`game-button ${showFilters ? 'bg-primary/20' : ''}`}
              >
                <Filter className="w-4 h-4 mr-1" />
                筛选
              </button>
              
              <div className="relative group">
                <button className="game-button">
                  <Download className="w-4 h-4 mr-1" />
                  导出
                </button>
                <div className="absolute right-0 mt-1 w-32 bg-gray-800 rounded-lg shadow-xl opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-10">
                  <button onClick={() => handleExport('json')} className="w-full px-3 py-2 text-left text-sm hover:bg-white/10">
                    JSON 格式
                  </button>
                  <button onClick={() => handleExport('csv')} className="w-full px-3 py-2 text-left text-sm hover:bg-white/10">
                    CSV 格式
                  </button>
                  <button onClick={() => handleExport('text')} className="w-full px-3 py-2 text-left text-sm hover:bg-white/10">
                    文本格式
                  </button>
                  <button onClick={() => handleExport('html')} className="w-full px-3 py-2 text-left text-sm hover:bg-white/10">
                    HTML 报告
                  </button>
                </div>
              </div>
              
              <button
                onClick={handleClearLogs}
                className="game-button text-red-400 hover:bg-red-900/20"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          </div>

          {showFilters && (
            <div className="game-card mb-4">
              <p className="text-sm text-text-secondary mb-2">日志级别筛选:</p>
              <div className="flex flex-wrap gap-2">
                {(['fatal', 'error', 'warn', 'info', 'debug', 'trace'] as LogLevel[]).map((level) => (
                  <button
                    key={level}
                    onClick={() => handleLevelToggle(level)}
                    className={`px-3 py-1 rounded text-sm transition-colors ${
                      selectedLevels.includes(level)
                        ? `${LEVEL_COLORS[level].bg} ${LEVEL_COLORS[level].text} ring-1 ring-current`
                        : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                    }`}
                  >
                    {level.toUpperCase()}
                  </button>
                ))}
              </div>
            </div>
          )}

          <div className="flex items-center justify-between text-sm text-gray-400 mb-2">
            <span>共 {totalLogs.toLocaleString()} 条日志</span>
            <span>第 {currentPage} 页</span>
          </div>

          <div 
            ref={logsContainerRef}
            className="flex-1 overflow-y-auto space-y-2 pr-2"
            style={{ maxHeight: 'calc(100vh - 300px)' }}
          >
            {isLoading ? (
              <div className="flex items-center justify-center h-64">
                <RefreshCw className="w-6 h-6 animate-spin text-primary" />
              </div>
            ) : logs.length === 0 ? (
              <div className="text-center py-12 text-gray-500">
                <Terminal className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p>暂无日志记录</p>
              </div>
            ) : (
              logs.map((entry) => (
                <LogEntryRow
                  key={entry.id}
                  entry={entry}
                  query={searchQuery}
                  onSelect={handleSelectLog}
                  selected={selectedLogId === entry.id}
                />
              ))
            )}
          </div>

          {totalLogs > 50 && (
            <div className="flex justify-center gap-2 mt-4">
              <button
                onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
                disabled={currentPage <= 1}
                className="game-button disabled:opacity-50"
              >
                上一页
              </button>
              <span className="px-4 py-2 text-gray-400">
                {currentPage} / {Math.ceil(totalLogs / 50)}
              </span>
              <button
                onClick={() => setCurrentPage(currentPage + 1)}
                disabled={currentPage >= Math.ceil(totalLogs / 50)}
                className="game-button disabled:opacity-50"
              >
                下一页
              </button>
            </div>
          )}
        </>
      )}

      {activeTab === 'dashboard' && dashboard && (
        <div className="overflow-y-auto" style={{ maxHeight: 'calc(100vh - 180px)' }}>
          <DashboardView dashboard={dashboard} />
        </div>
      )}

      {activeTab === 'analysis' && (
        <div className="overflow-y-auto" style={{ maxHeight: 'calc(100vh - 180px)' }}>
          <AnalysisView 
            diagnostic={diagnostic} 
            bottlenecks={bottlenecks}
            onLookupKnowledge={handleKnowledgeLookup}
          />
        </div>
      )}

      {activeTab === 'knowledge' && (
        <div className="overflow-y-auto" style={{ maxHeight: 'calc(100vh - 180px)' }}>
          <KnowledgeView knowledge={knowledge} />
        </div>
      )}

      {activeTab === 'rules' && (
        <div className="overflow-y-auto" style={{ maxHeight: 'calc(100vh - 180px)' }}>
          <RulesView rules={rules} onRefresh={loadRules} />
        </div>
      )}

      {selectedLogId && selectedLogAnalysis && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-gray-900 rounded-xl max-w-2xl w-full max-h-[80vh] overflow-hidden">
            <div className="flex items-center justify-between p-4 border-b border-gray-700">
              <h3 className="text-lg font-bold text-text-primary">日志分析详情</h3>
              <button
                onClick={() => { setSelectedLogId(null); setSelectedLogAnalysis(null); }}
                className="p-1 rounded hover:bg-white/10"
              >
                <XCircle className="w-5 h-5" />
              </button>
            </div>
            <div className="p-4 overflow-y-auto" style={{ maxHeight: 'calc(80vh - 60px)' }}>
              <div className="mb-4">
                <p className="text-sm text-gray-400 mb-1">异常评分</p>
                <div className="flex items-center gap-3">
                  <div className="flex-1 bg-gray-700 rounded-full h-2">
                    <div 
                      className={`h-2 rounded-full ${
                        selectedLogAnalysis.anomaly_score > 70 ? 'bg-red-500' :
                        selectedLogAnalysis.anomaly_score > 40 ? 'bg-yellow-500' :
                        'bg-green-500'
                      }`}
                      style={{ width: `${selectedLogAnalysis.anomaly_score}%` }}
                    />
                  </div>
                  <span className="text-lg font-bold">{selectedLogAnalysis.anomaly_score.toFixed(0)}</span>
                </div>
              </div>

              {selectedLogAnalysis.issues?.length > 0 && (
                <div className="mb-4">
                  <p className="text-sm text-gray-400 mb-2">检测到的问题:</p>
                  <div className="space-y-2">
                    {selectedLogAnalysis.issues.map((issue: any, i: number) => (
                      <div key={i} className="bg-red-900/20 rounded-lg p-3 border border-red-800">
                        <p className="text-red-300 font-medium">{issue.description}</p>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {selectedLogAnalysis.recommendations?.length > 0 && (
                <div>
                  <p className="text-sm text-gray-400 mb-2">建议:</p>
                  <ul className="space-y-1">
                    {selectedLogAnalysis.recommendations.map((rec: string, i: number) => (
                      <li key={i} className="flex items-start gap-2 text-green-400">
                        <Check className="w-4 h-4 mt-0.5 flex-shrink-0" />
                        {rec}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {knowledgeLookupResult && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-gray-900 rounded-xl max-w-2xl w-full max-h-[80vh] overflow-hidden">
            <div className="flex items-center justify-between p-4 border-b border-gray-700">
              <h3 className="text-lg font-bold text-text-primary flex items-center gap-2">
                <BookOpen className="w-5 h-5 text-primary" />
                知识库匹配结果
              </h3>
              <button
                onClick={() => setKnowledgeLookupResult(null)}
                className="p-1 rounded hover:bg-white/10"
              >
                <XCircle className="w-5 h-5" />
              </button>
            </div>
            <div className="p-4 overflow-y-auto" style={{ maxHeight: 'calc(80vh - 60px)' }}>
              <div className="mb-4">
                <h4 className="text-xl font-bold text-primary">{knowledgeLookupResult.title}</h4>
                <p className="text-gray-400 mt-2">{knowledgeLookupResult.description}</p>
              </div>
              
              <div className="mb-4">
                <p className="text-sm text-red-400 font-semibold mb-1">原因:</p>
                <pre className="text-sm text-gray-300 whitespace-pre-wrap bg-gray-800 rounded p-3">
                  {knowledgeLookupResult.cause}
                </pre>
              </div>
              
              <div className="mb-4">
                <p className="text-sm text-green-400 font-semibold mb-1">解决方案:</p>
                <pre className="text-sm text-gray-300 whitespace-pre-wrap bg-gray-800 rounded p-3">
                  {knowledgeLookupResult.solution}
                </pre>
              </div>
              
              {knowledgeLookupResult.prevention.length > 0 && (
                <div>
                  <p className="text-sm text-blue-400 font-semibold mb-1">预防措施:</p>
                  <ul className="space-y-1">
                    {knowledgeLookupResult.prevention.map((item, i) => (
                      <li key={i} className="flex items-center gap-2 text-gray-300">
                        <Shield className="w-4 h-4 text-blue-400" />
                        {item}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default Logs;

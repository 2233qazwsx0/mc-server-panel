import { useState, useEffect } from 'react';
import type { AuditLogEntry, AuditStats, AuditAction } from '../../types/security';

interface Props {
  apiBase: string;
}

export function AuditLogPanel({ apiBase }: Props) {
  const [logs, setLogs] = useState<AuditLogEntry[]>([]);
  const [stats, setStats] = useState<AuditStats | null>(null);
  const [filter, setFilter] = useState({
    action: '',
    status: '',
    user_id: '',
    start_date: '',
    end_date: '',
  });
  const [loading, setLoading] = useState(false);

  const fetchLogs = async () => {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      if (filter.action) params.append('action', filter.action);
      if (filter.status) params.append('status', filter.status);
      if (filter.user_id) params.append('user_id', filter.user_id);
      if (filter.start_date) params.append('start_date', filter.start_date);
      if (filter.end_date) params.append('end_date', filter.end_date);

      const [logsRes, statsRes] = await Promise.all([
        fetch(`${apiBase}/security/audit/logs?${params}`),
        fetch(`${apiBase}/security/audit/stats`),
      ]);
      setLogs(await logsRes.json());
      setStats(await statsRes.json());
    } catch (err) {
      console.error('Failed to fetch audit logs:', err);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchLogs();
  }, []);

  const getActionLabel = (action: AuditAction): string => {
    const labels: Record<AuditAction, string> = {
      Login: '登录',
      Logout: '登出',
      LoginFailed: '登录失败',
      PasswordChange: '密码更改',
      PasswordReset: '密码重置',
      UserCreate: '用户创建',
      UserDelete: '用户删除',
      UserUpdate: '用户更新',
      RoleChange: '角色更改',
      PermissionGrant: '权限授予',
      PermissionRevoke: '权限撤销',
      ServerStart: '服务器启动',
      ServerStop: '服务器停止',
      ServerRestart: '服务器重启',
      ServerCommand: '服务器命令',
      FileRead: '文件读取',
      FileWrite: '文件写入',
      FileDelete: '文件删除',
      ConfigChange: '配置更改',
      SecurityScan: '安全扫描',
      IpBlock: 'IP封禁',
      IpUnblock: 'IP解封',
      TotpEnable: '2FA启用',
      TotpDisable: '2FA禁用',
      TotpVerify: '2FA验证',
      ApiKeyCreate: 'API密钥创建',
      ApiKeyDelete: 'API密钥删除',
      ApiKeyUse: 'API密钥使用',
      SessionCreate: '会话创建',
      SessionDestroy: '会话销毁',
      BruteForceBlock: '暴力破解封禁',
      SslRenew: 'SSL续期',
      DataExport: '数据导出',
      DataImport: '数据导入',
      AdminAction: '管理操作',
      Other: '其他',
    };
    return labels[action] || action;
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">操作审计日志</h2>

        <div className="grid grid-cols-4 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.total_entries || 0}</div>
            <div className="text-sm text-gray-400">总日志数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-400">{stats?.success_count || 0}</div>
            <div className="text-sm text-gray-400">成功操作</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-red-400">{stats?.failure_count || 0}</div>
            <div className="text-sm text-gray-400">失败操作</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4 flex items-center justify-center">
            <button
              onClick={fetchLogs}
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded"
            >
              刷新
            </button>
          </div>
        </div>

        <div className="bg-gray-700 rounded-lg p-4 mb-6">
          <h3 className="text-lg font-semibold text-white mb-4">筛选条件</h3>
          <div className="grid grid-cols-5 gap-4">
            <select
              value={filter.action}
              onChange={(e) => setFilter({ ...filter, action: e.target.value })}
              className="bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">全部操作</option>
              <option value="Login">登录</option>
              <option value="Logout">登出</option>
              <option value="ServerCommand">服务器命令</option>
              <option value="ConfigChange">配置更改</option>
              <option value="UserCreate">用户创建</option>
              <option value="SecurityScan">安全扫描</option>
            </select>
            <select
              value={filter.status}
              onChange={(e) => setFilter({ ...filter, status: e.target.value })}
              className="bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">全部状态</option>
              <option value="Success">成功</option>
              <option value="Failure">失败</option>
            </select>
            <input
              type="text"
              value={filter.user_id}
              onChange={(e) => setFilter({ ...filter, user_id: e.target.value })}
              placeholder="用户 ID"
              className="bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            />
            <input
              type="date"
              value={filter.start_date}
              onChange={(e) => setFilter({ ...filter, start_date: e.target.value })}
              className="bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            />
            <input
              type="date"
              value={filter.end_date}
              onChange={(e) => setFilter({ ...filter, end_date: e.target.value })}
              className="bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <button
            onClick={fetchLogs}
            className="mt-4 bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded"
          >
            应用筛选
          </button>
        </div>

        <div className="bg-gray-700 rounded-lg overflow-hidden">
          <div className="overflow-auto max-h-96">
            <table className="w-full text-sm">
              <thead className="bg-gray-600 sticky top-0">
                <tr>
                  <th className="px-4 py-2 text-left text-gray-300">时间</th>
                  <th className="px-4 py-2 text-left text-gray-300">操作</th>
                  <th className="px-4 py-2 text-left text-gray-300">用户</th>
                  <th className="px-4 py-2 text-left text-gray-300">资源</th>
                  <th className="px-4 py-2 text-left text-gray-300">IP</th>
                  <th className="px-4 py-2 text-left text-gray-300">状态</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-600">
                {loading ? (
                  <tr>
                    <td colSpan={6} className="px-4 py-8 text-center text-gray-500">加载中...</td>
                  </tr>
                ) : logs.length === 0 ? (
                  <tr>
                    <td colSpan={6} className="px-4 py-8 text-center text-gray-500">暂无日志</td>
                  </tr>
                ) : (
                  logs.map((log) => (
                    <tr key={log.id} className="hover:bg-gray-600">
                      <td className="px-4 py-2 text-gray-400">{new Date(log.timestamp).toLocaleString()}</td>
                      <td className="px-4 py-2 text-white">{getActionLabel(log.action)}</td>
                      <td className="px-4 py-2 text-white">{log.username || log.user_id || '-'}</td>
                      <td className="px-4 py-2 text-gray-400">{log.resource}</td>
                      <td className="px-4 py-2 text-gray-400 font-mono text-xs">{log.ip_address || '-'}</td>
                      <td className="px-4 py-2">
                        <span className={`px-2 py-1 rounded text-xs ${
                          log.status === 'Success' ? 'bg-green-900 text-green-400' :
                          log.status === 'Failure' ? 'bg-red-900 text-red-400' :
                          'bg-yellow-900 text-yellow-400'
                        }`}>
                          {log.status === 'Success' ? '成功' : log.status === 'Failure' ? '失败' : '进行中'}
                        </span>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}

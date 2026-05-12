import { useState, useEffect } from 'react';
import type { Session, SessionStats } from '../../types/security';

interface Props {
  apiBase: string;
}

export function SessionPanel({ apiBase }: Props) {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [stats, setStats] = useState<SessionStats | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchData = async () => {
    setLoading(true);
    try {
      const [sessionsRes, statsRes] = await Promise.all([
        fetch(`${apiBase}/security/session/list`),
        fetch(`${apiBase}/security/session/stats`),
      ]);
      setSessions(await sessionsRes.json());
      setStats(await statsRes.json());
    } catch (err) {
      console.error('Failed to fetch sessions:', err);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleInvalidate = async (sessionId: string) => {
    try {
      await fetch(`${apiBase}/security/session/${sessionId}`, { method: 'DELETE' });
      fetchData();
    } catch (err) {
      console.error('Failed to invalidate session:', err);
    }
  };

  const handleInvalidateUser = async (userId: string) => {
    try {
      await fetch(`${apiBase}/security/session/invalidate/${userId}`, { method: 'DELETE' });
      fetchData();
    } catch (err) {
      console.error('Failed to invalidate user sessions:', err);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">会话管理</h2>

        <div className="grid grid-cols-4 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.total_sessions || 0}</div>
            <div className="text-sm text-gray-400">总会话数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-400">{stats?.active_sessions || 0}</div>
            <div className="text-sm text-gray-400">活跃会话</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-gray-400">{stats?.expired_sessions || 0}</div>
            <div className="text-sm text-gray-400">已过期</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-purple-400">{stats?.unique_users || 0}</div>
            <div className="text-sm text-gray-400">独立用户</div>
          </div>
        </div>

        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-white">活跃会话列表</h3>
          <button
            onClick={fetchData}
            disabled={loading}
            className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white px-4 py-2 rounded"
          >
            刷新
          </button>
        </div>

        <div className="bg-gray-700 rounded-lg overflow-hidden">
          <div className="overflow-auto max-h-96">
            <table className="w-full text-sm">
              <thead className="bg-gray-600 sticky top-0">
                <tr>
                  <th className="px-4 py-2 text-left text-gray-300">用户</th>
                  <th className="px-4 py-2 text-left text-gray-300">会话 ID</th>
                  <th className="px-4 py-2 text-left text-gray-300">IP 地址</th>
                  <th className="px-4 py-2 text-left text-gray-300">最后活动</th>
                  <th className="px-4 py-2 text-left text-gray-300">过期时间</th>
                  <th className="px-4 py-2 text-left text-gray-300">权限</th>
                  <th className="px-4 py-2 text-left text-gray-300">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-600">
                {loading ? (
                  <tr>
                    <td colSpan={7} className="px-4 py-8 text-center text-gray-500">加载中...</td>
                  </tr>
                ) : sessions.length === 0 ? (
                  <tr>
                    <td colSpan={7} className="px-4 py-8 text-center text-gray-500">暂无活跃会话</td>
                  </tr>
                ) : (
                  sessions.map((session) => (
                    <tr key={session.id} className="hover:bg-gray-600">
                      <td className="px-4 py-2 text-white">{session.username}</td>
                      <td className="px-4 py-2 text-gray-400 font-mono text-xs">{session.id.slice(0, 8)}...</td>
                      <td className="px-4 py-2 text-gray-400 font-mono text-xs">{session.ip_address || '-'}</td>
                      <td className="px-4 py-2 text-gray-400">{new Date(session.last_activity).toLocaleString()}</td>
                      <td className="px-4 py-2 text-gray-400">{new Date(session.expires_at).toLocaleString()}</td>
                      <td className="px-4 py-2">
                        <span className="text-xs text-blue-400">
                          {session.permissions.length} 个权限
                        </span>
                      </td>
                      <td className="px-4 py-2">
                        <div className="flex gap-2">
                          <button
                            onClick={() => handleInvalidate(session.id)}
                            className="text-red-400 hover:text-red-300 text-sm"
                          >
                            销毁
                          </button>
                          <button
                            onClick={() => handleInvalidateUser(session.user_id)}
                            className="text-yellow-400 hover:text-yellow-300 text-sm"
                          >
                            全部销毁
                          </button>
                        </div>
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

import { useState, useEffect } from 'react';
import type { ApiKey, ApiKeyStats, ApiPermission, CreateApiKeyResponse } from '../../types/security';

interface Props {
  apiBase: string;
}

const PERMISSION_OPTIONS: { value: ApiPermission; label: string }[] = [
  { value: 'Read', label: '读取' },
  { value: 'Write', label: '写入' },
  { value: 'Admin', label: '管理员' },
  { value: 'ServerStart', label: '服务器启动' },
  { value: 'ServerStop', label: '服务器停止' },
  { value: 'ServerCommand', label: '服务器命令' },
  { value: 'FileRead', label: '文件读取' },
  { value: 'FileWrite', label: '文件写入' },
  { value: 'ConfigRead', label: '配置读取' },
  { value: 'ConfigWrite', label: '配置写入' },
  { value: 'MetricsRead', label: '监控读取' },
  { value: 'LogsRead', label: '日志读取' },
  { value: 'SecurityRead', label: '安全读取' },
  { value: 'AuditRead', label: '审计读取' },
];

export function ApiKeyPanel({ apiBase }: Props) {
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [stats, setStats] = useState<ApiKeyStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  const [newKeyData, setNewKeyData] = useState<CreateApiKeyResponse | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    permissions: [] as ApiPermission[],
    user_id: '',
  });

  const fetchData = async () => {
    setLoading(true);
    try {
      const [keysRes, statsRes] = await Promise.all([
        fetch(`${apiBase}/security/apikey/list`),
        fetch(`${apiBase}/security/apikey/stats`),
      ]);
      setKeys(await keysRes.json());
      setStats(await statsRes.json());
    } catch (err) {
      console.error('Failed to fetch API keys:', err);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleCreate = async () => {
    if (!formData.name) return;
    setLoading(true);
    try {
      const res = await fetch(`${apiBase}/security/apikey/create`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: formData.name,
          permissions: formData.permissions,
          user_id: formData.user_id || null,
        }),
      });
      const data: CreateApiKeyResponse = await res.json();
      setNewKeyData(data);
      setShowCreate(false);
      fetchData();
    } catch (err) {
      console.error('Failed to create API key:', err);
    }
    setLoading(false);
  };

  const handleTogglePermission = (perm: ApiPermission) => {
    setFormData((prev) => ({
      ...prev,
      permissions: prev.permissions.includes(perm)
        ? prev.permissions.filter((p) => p !== perm)
        : [...prev.permissions, perm],
    }));
  };

  const handleDelete = async (keyId: string) => {
    if (!confirm('确定要删除此 API 密钥吗？')) return;
    try {
      await fetch(`${apiBase}/security/apikey/${keyId}`, { method: 'DELETE' });
      fetchData();
    } catch (err) {
      console.error('Failed to delete API key:', err);
    }
  };

  const handleToggle = async (keyId: string, enable: boolean) => {
    try {
      const endpoint = enable ? 'enable' : 'disable';
      await fetch(`${apiBase}/security/apikey/${keyId}/${endpoint}`, { method: 'POST' });
      fetchData();
    } catch (err) {
      console.error('Failed to toggle API key:', err);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">API 密钥权限</h2>

        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.total_keys || 0}</div>
            <div className="text-sm text-gray-400">总密钥数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-400">{stats?.active_keys || 0}</div>
            <div className="text-sm text-gray-400">活跃密钥</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-red-400">{stats?.expired_keys || 0}</div>
            <div className="text-sm text-gray-400">已过期</div>
          </div>
        </div>

        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-white">API 密钥列表</h3>
          <div className="flex gap-2">
            <button
              onClick={fetchData}
              className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded"
            >
              刷新
            </button>
            <button
              onClick={() => {
                setShowCreate(true);
                setNewKeyData(null);
              }}
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded"
            >
              创建密钥
            </button>
          </div>
        </div>

        {showCreate && (
          <div className="bg-gray-700 rounded-lg p-4 mb-4">
            <h4 className="text-white font-semibold mb-4">创建新 API 密钥</h4>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">密钥名称</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  placeholder="输入密钥名称"
                  className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-2">权限</label>
                <div className="grid grid-cols-4 gap-2">
                  {PERMISSION_OPTIONS.map((opt) => (
                    <label key={opt.value} className="flex items-center gap-2 text-white text-sm">
                      <input
                        type="checkbox"
                        checked={formData.permissions.includes(opt.value)}
                        onChange={() => handleTogglePermission(opt.value)}
                        className="rounded"
                      />
                      {opt.label}
                    </label>
                  ))}
                </div>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={handleCreate}
                  disabled={loading || !formData.name || formData.permissions.length === 0}
                  className="bg-green-600 hover:bg-green-700 disabled:bg-gray-600 text-white px-4 py-2 rounded"
                >
                  创建
                </button>
                <button
                  onClick={() => setShowCreate(false)}
                  className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded"
                >
                  取消
                </button>
              </div>
            </div>
          </div>
        )}

        {newKeyData && (
          <div className="bg-yellow-900/30 border border-yellow-700 rounded-lg p-4 mb-4">
            <div className="text-yellow-400 font-semibold mb-2">⚠️ 新密钥已创建，请立即复制保存！</div>
            <div className="bg-gray-700 p-3 rounded break-all">
              <div className="text-sm text-gray-400 mb-1">API 密钥:</div>
              <div className="text-white font-mono">{newKeyData.raw_key}</div>
            </div>
          </div>
        )}

        <div className="bg-gray-700 rounded-lg overflow-hidden">
          <div className="overflow-auto max-h-80">
            <table className="w-full text-sm">
              <thead className="bg-gray-600 sticky top-0">
                <tr>
                  <th className="px-4 py-2 text-left text-gray-300">名称</th>
                  <th className="px-4 py-2 text-left text-gray-300">前缀</th>
                  <th className="px-4 py-2 text-left text-gray-300">权限</th>
                  <th className="px-4 py-2 text-left text-gray-300">状态</th>
                  <th className="px-4 py-2 text-left text-gray-300">创建时间</th>
                  <th className="px-4 py-2 text-left text-gray-300">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-600">
                {keys.length === 0 ? (
                  <tr>
                    <td colSpan={6} className="px-4 py-8 text-center text-gray-500">暂无 API 密钥</td>
                  </tr>
                ) : (
                  keys.map((key) => (
                    <tr key={key.id} className="hover:bg-gray-600">
                      <td className="px-4 py-2 text-white">{key.name}</td>
                      <td className="px-4 py-2 text-gray-400 font-mono">{key.key_prefix}***</td>
                      <td className="px-4 py-2">
                        <div className="flex flex-wrap gap-1">
                          {key.permissions.slice(0, 3).map((p) => (
                            <span key={p} className="text-xs bg-blue-900 text-blue-300 px-1 rounded">{p}</span>
                          ))}
                          {key.permissions.length > 3 && (
                            <span className="text-xs text-gray-400">+{key.permissions.length - 3}</span>
                          )}
                        </div>
                      </td>
                      <td className="px-4 py-2">
                        <span className={`px-2 py-1 rounded text-xs ${key.is_active ? 'bg-green-900 text-green-400' : 'bg-red-900 text-red-400'}`}>
                          {key.is_active ? '活跃' : '禁用'}
                        </span>
                      </td>
                      <td className="px-4 py-2 text-gray-400">{new Date(key.created_at).toLocaleDateString()}</td>
                      <td className="px-4 py-2">
                        <div className="flex gap-2">
                          <button
                            onClick={() => handleToggle(key.id, !key.is_active)}
                            className="text-blue-400 hover:text-blue-300 text-sm"
                          >
                            {key.is_active ? '禁用' : '启用'}
                          </button>
                          <button
                            onClick={() => handleDelete(key.id)}
                            className="text-red-400 hover:text-red-300 text-sm"
                          >
                            删除
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

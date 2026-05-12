import { useState, useEffect } from 'react';
import type { IpFilterStats, IpCheckResponse } from '../../types/security';

interface Props {
  apiBase: string;
}

export function IpFilterPanel({ apiBase }: Props) {
  const [whitelist, setWhitelist] = useState<string[]>([]);
  const [blacklist, setBlacklist] = useState<string[]>([]);
  const [stats, setStats] = useState<IpFilterStats | null>(null);
  const [checkIp, setCheckIp] = useState('');
  const [checkResult, setCheckResult] = useState<IpCheckResponse | null>(null);
  const [newIp, setNewIp] = useState('');
  const [reason, setReason] = useState('');
  const [mode, setMode] = useState<'whitelist' | 'blacklist'>('blacklist');
  const [loading, setLoading] = useState(false);

  const fetchData = async () => {
    try {
      const [wlRes, blRes, statsRes] = await Promise.all([
        fetch(`${apiBase}/security/ip/whitelist`),
        fetch(`${apiBase}/security/ip/blacklist`),
        fetch(`${apiBase}/security/ip/stats`),
      ]);
      setWhitelist(await wlRes.json());
      setBlacklist(await blRes.json());
      setStats(await statsRes.json());
    } catch (err) {
      console.error('Failed to fetch IP data:', err);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleAddIp = async () => {
    if (!newIp) return;
    setLoading(true);
    try {
      const endpoint = mode === 'whitelist' ? 'whitelist' : 'blacklist';
      await fetch(`${apiBase}/security/ip/${endpoint}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ip: newIp, reason: reason || null }),
      });
      setNewIp('');
      setReason('');
      fetchData();
    } catch (err) {
      console.error('Failed to add IP:', err);
    }
    setLoading(false);
  };

  const handleCheckIp = async () => {
    if (!checkIp) return;
    try {
      const res = await fetch(`${apiBase}/security/ip/check/${checkIp}`);
      setCheckResult(await res.json());
    } catch (err) {
      console.error('Failed to check IP:', err);
    }
  };

  const handleClear = async (type: 'whitelist' | 'blacklist') => {
    try {
      await fetch(`${apiBase}/security/ip/${type}`, { method: 'DELETE' });
      fetchData();
    } catch (err) {
      console.error('Failed to clear list:', err);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">IP 黑名单/白名单管理</h2>
        
        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-400">{stats?.whitelist_count || 0}</div>
            <div className="text-sm text-gray-400">白名单 IP</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-red-400">{stats?.blacklist_count || 0}</div>
            <div className="text-sm text-gray-400">黑名单 IP</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.total_entries || 0}</div>
            <div className="text-sm text-gray-400">总条目</div>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-6">
          <div>
            <div className="flex justify-between items-center mb-3">
              <h3 className="text-lg font-semibold text-green-400">白名单</h3>
              <button
                onClick={() => handleClear('whitelist')}
                className="text-sm text-red-400 hover:text-red-300"
              >
                清空
              </button>
            </div>
            <div className="bg-gray-700 rounded-lg p-3 max-h-48 overflow-auto">
              {whitelist.length === 0 ? (
                <div className="text-gray-500 text-center py-4">无白名单 IP</div>
              ) : (
                <div className="space-y-2">
                  {whitelist.map((ip) => (
                    <div key={ip} className="flex justify-between items-center text-sm">
                      <span className="text-white font-mono">{ip}</span>
                      <button
                        onClick={async () => {
                          await fetch(`${apiBase}/security/ip/whitelist`, { method: 'DELETE' });
                          fetchData();
                        }}
                        className="text-red-400 hover:text-red-300"
                      >
                        移除
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          <div>
            <div className="flex justify-between items-center mb-3">
              <h3 className="text-lg font-semibold text-red-400">黑名单</h3>
              <button
                onClick={() => handleClear('blacklist')}
                className="text-sm text-red-400 hover:text-red-300"
              >
                清空
              </button>
            </div>
            <div className="bg-gray-700 rounded-lg p-3 max-h-48 overflow-auto">
              {blacklist.length === 0 ? (
                <div className="text-gray-500 text-center py-4">无黑名单 IP</div>
              ) : (
                <div className="space-y-2">
                  {blacklist.map((ip) => (
                    <div key={ip} className="flex justify-between items-center text-sm">
                      <span className="text-white font-mono">{ip}</span>
                      <button
                        onClick={async () => {
                          await fetch(`${apiBase}/security/ip/blacklist`, { method: 'DELETE' });
                          fetchData();
                        }}
                        className="text-red-400 hover:text-red-300"
                      >
                        移除
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>

        <div className="mt-6 border-t border-gray-700 pt-6">
          <h3 className="text-lg font-semibold text-white mb-4">添加 IP</h3>
          <div className="flex gap-4 items-center">
            <div className="flex-1">
              <input
                type="text"
                value={newIp}
                onChange={(e) => setNewIp(e.target.value)}
                placeholder="输入 IP 地址 (如 192.168.1.1)"
                className="w-full bg-gray-700 text-white rounded px-4 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <input
              type="text"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="原因 (可选)"
              className="flex-1 bg-gray-700 text-white rounded px-4 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            />
            <select
              value={mode}
              onChange={(e) => setMode(e.target.value as 'whitelist' | 'blacklist')}
              className="bg-gray-700 text-white rounded px-4 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="blacklist">添加到黑名单</option>
              <option value="whitelist">添加到白名单</option>
            </select>
            <button
              onClick={handleAddIp}
              disabled={loading || !newIp}
              className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white px-6 py-2 rounded transition-colors"
            >
              添加
            </button>
          </div>
        </div>

        <div className="mt-6 border-t border-gray-700 pt-6">
          <h3 className="text-lg font-semibold text-white mb-4">IP 检查</h3>
          <div className="flex gap-4 items-center">
            <input
              type="text"
              value={checkIp}
              onChange={(e) => setCheckIp(e.target.value)}
              placeholder="输入要检查的 IP 地址"
              className="flex-1 bg-gray-700 text-white rounded px-4 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
              onClick={handleCheckIp}
              className="bg-green-600 hover:bg-green-700 text-white px-6 py-2 rounded transition-colors"
            >
              检查
            </button>
          </div>
          {checkResult && (
            <div className={`mt-4 p-4 rounded-lg ${checkResult.allowed ? 'bg-green-900/30' : 'bg-red-900/30'}`}>
              <div className={`text-lg font-semibold ${checkResult.allowed ? 'text-green-400' : 'text-red-400'}`}>
                {checkResult.allowed ? '✅ 允许' : '❌ 拒绝'}
              </div>
              <div className="text-sm text-gray-400 mt-1">原因: {checkResult.reason}</div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

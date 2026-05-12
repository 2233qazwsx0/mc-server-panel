import { useState, useEffect } from 'react';
import type { DdosStats, RateLimitConfig } from '../../types/security';

interface Props {
  apiBase: string;
}

export function DdosProtectionPanel({ apiBase }: Props) {
  const [stats, setStats] = useState<DdosStats | null>(null);
  const [config, setConfig] = useState<RateLimitConfig | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchData = async () => {
    try {
      const [statsRes, configRes] = await Promise.all([
        fetch(`${apiBase}/security/ddos/stats`),
        fetch(`${apiBase}/security/ddos/config`),
      ]);
      setStats(await statsRes.json());
      setConfig(await configRes.json());
    } catch (err) {
      console.error('Failed to fetch DDoS data:', err);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleUnblock = async (ip: string) => {
    try {
      await fetch(`${apiBase}/security/ddos/unblock/${ip}`, { method: 'DELETE' });
      fetchData();
    } catch (err) {
      console.error('Failed to unblock IP:', err);
    }
  };

  const handleSaveConfig = async () => {
    if (!config) return;
    setLoading(true);
    try {
      await fetch(`${apiBase}/security/ddos/config`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
    } catch (err) {
      console.error('Failed to save config:', err);
    }
    setLoading(false);
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">DDoS 基础防护</h2>

        <div className="grid grid-cols-4 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.total_requests?.toLocaleString() || 0}</div>
            <div className="text-sm text-gray-400">总请求数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-400">{stats?.active_ips || 0}</div>
            <div className="text-sm text-gray-400">活跃 IP</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-yellow-400">{stats?.high_traffic_ips?.length || 0}</div>
            <div className="text-sm text-gray-400">高流量 IP</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-red-400">{stats?.currently_blocked || 0}</div>
            <div className="text-sm text-gray-400">已阻止 IP</div>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-6">
          <div>
            <h3 className="text-lg font-semibold text-white mb-3">防护配置</h3>
            {config && (
              <div className="space-y-4 bg-gray-700 rounded-lg p-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">每分钟请求限制</label>
                  <input
                    type="number"
                    value={config.requests_per_minute}
                    onChange={(e) => setConfig({ ...config, requests_per_minute: parseInt(e.target.value) || 60 })}
                    className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">每小时请求限制</label>
                  <input
                    type="number"
                    value={config.requests_per_hour}
                    onChange={(e) => setConfig({ ...config, requests_per_hour: parseInt(e.target.value) || 1000 })}
                    className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">突发大小</label>
                  <input
                    type="number"
                    value={config.burst_size}
                    onChange={(e) => setConfig({ ...config, burst_size: parseInt(e.target.value) || 10 })}
                    className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">封禁时长 (秒)</label>
                  <input
                    type="number"
                    value={config.block_duration_secs}
                    onChange={(e) => setConfig({ ...config, block_duration_secs: parseInt(e.target.value) || 300 })}
                    className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <button
                  onClick={handleSaveConfig}
                  disabled={loading}
                  className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white py-2 rounded transition-colors"
                >
                  保存配置
                </button>
              </div>
            )}
          </div>

          <div>
            <h3 className="text-lg font-semibold text-white mb-3">高流量 IP</h3>
            <div className="bg-gray-700 rounded-lg p-4 max-h-80 overflow-auto">
              {!stats?.high_traffic_ips?.length ? (
                <div className="text-gray-500 text-center py-4">无高流量 IP</div>
              ) : (
                <div className="space-y-2">
                  {stats.high_traffic_ips.map((ip) => (
                    <div key={ip.ip} className="flex justify-between items-center bg-gray-600 rounded p-2">
                      <div>
                        <div className="text-white font-mono text-sm">{ip.ip}</div>
                        <div className="text-xs text-gray-400">
                          分钟: {ip.minute_requests} | 小时: {ip.hour_requests}
                        </div>
                      </div>
                      <button
                        onClick={() => handleUnblock(ip.ip)}
                        className="text-red-400 hover:text-red-300 text-sm"
                      >
                        封禁
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

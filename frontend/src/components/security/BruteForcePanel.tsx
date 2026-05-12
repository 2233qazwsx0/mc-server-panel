import { useState, useEffect } from 'react';
import type { BruteForceStats, BruteForceConfig, LockedIpInfo } from '../../types/security';

interface Props {
  apiBase: string;
}

export function BruteForcePanel({ apiBase }: Props) {
  const [stats, setStats] = useState<BruteForceStats | null>(null);
  const [lockedIps, setLockedIps] = useState<LockedIpInfo[]>([]);
  const [config, setConfig] = useState<BruteForceConfig | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchData = async () => {
    try {
      const [statsRes, lockedRes, configRes] = await Promise.all([
        fetch(`${apiBase}/security/bruteforce/stats`),
        fetch(`${apiBase}/security/bruteforce/locked`),
        fetch(`${apiBase}/security/bruteforce/config`),
      ]);
      setStats(await statsRes.json());
      setLockedIps(await lockedRes.json());
      setConfig(await configRes.json());
    } catch (err) {
      console.error('Failed to fetch data:', err);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleUnblock = async (ip: string) => {
    try {
      await fetch(`${apiBase}/security/bruteforce/unblock/${ip}`, { method: 'DELETE' });
      fetchData();
    } catch (err) {
      console.error('Failed to unblock IP:', err);
    }
  };

  const handleSaveConfig = async () => {
    if (!config) return;
    setLoading(true);
    try {
      await fetch(`${apiBase}/security/bruteforce/config`, {
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
        <h2 className="text-xl font-bold text-white mb-4">RCON 暴力破解拦截</h2>

        <div className="grid grid-cols-4 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.tracked_ips || 0}</div>
            <div className="text-sm text-gray-400">追踪 IP</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-red-400">{stats?.locked_ips || 0}</div>
            <div className="text-sm text-gray-400">已锁定 IP</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-yellow-400">{stats?.total_attempts?.toLocaleString() || 0}</div>
            <div className="text-sm text-gray-400">总尝试次数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-orange-400">{stats?.alerts_count || 0}</div>
            <div className="text-sm text-gray-400">告警次数</div>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-6">
          <div>
            <h3 className="text-lg font-semibold text-white mb-3">防护配置</h3>
            {config && (
              <div className="space-y-4 bg-gray-700 rounded-lg p-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">最大尝试次数</label>
                  <input
                    type="number"
                    value={config.max_attempts}
                    onChange={(e) => setConfig({ ...config, max_attempts: parseInt(e.target.value) || 5 })}
                    className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">锁定时长 (秒)</label>
                  <input
                    type="number"
                    value={config.lockout_duration_secs}
                    onChange={(e) => setConfig({ ...config, lockout_duration_secs: parseInt(e.target.value) || 900 })}
                    className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">告警阈值</label>
                  <input
                    type="number"
                    value={config.alert_threshold}
                    onChange={(e) => setConfig({ ...config, alert_threshold: parseInt(e.target.value) || 3 })}
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
            <h3 className="text-lg font-semibold text-white mb-3">已锁定 IP</h3>
            <div className="bg-gray-700 rounded-lg p-4 max-h-80 overflow-auto">
              {lockedIps.length === 0 ? (
                <div className="text-gray-500 text-center py-4">无锁定 IP</div>
              ) : (
                <div className="space-y-2">
                  {lockedIps.map((ip) => (
                    <div key={ip.ip} className="flex justify-between items-center bg-gray-600 rounded p-3">
                      <div>
                        <div className="text-white font-mono">{ip.ip}</div>
                        <div className="text-xs text-gray-400">
                          失败: {ip.failed_attempts} | 锁定至: {ip.locked_until ? new Date(ip.locked_until).toLocaleString() : 'N/A'}
                        </div>
                      </div>
                      <button
                        onClick={() => handleUnblock(ip.ip)}
                        className="bg-green-600 hover:bg-green-700 px-3 py-1 rounded text-sm"
                      >
                        解除
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

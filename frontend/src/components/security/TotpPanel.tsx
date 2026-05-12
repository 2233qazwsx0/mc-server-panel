import { useState, useEffect } from 'react';
import type { TotpStats, TotpUserInfo, TotpSetup, TotpVerifyResponse } from '../../types/security';

interface Props {
  apiBase: string;
}

export function TotpPanel({ apiBase }: Props) {
  const [stats, setStats] = useState<TotpStats | null>(null);
  const [userId, setUserId] = useState('');
  const [setupData, setSetupData] = useState<TotpSetup | null>(null);
  const [token, setToken] = useState('');
  const [verifyResult, setVerifyResult] = useState<TotpVerifyResponse | null>(null);
  const [userInfo, setUserInfo] = useState<TotpUserInfo | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchStats = async () => {
    try {
      const res = await fetch(`${apiBase}/security/totp/stats`);
      setStats(await res.json());
    } catch (err) {
      console.error('Failed to fetch stats:', err);
    }
  };

  useEffect(() => {
    fetchStats();
  }, []);

  const handleSetup = async () => {
    if (!userId) return;
    setLoading(true);
    try {
      const res = await fetch(`${apiBase}/security/totp/setup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_id: userId }),
      });
      const data = await res.json();
      setSetupData(data);
    } catch (err) {
      console.error('Failed to setup TOTP:', err);
    }
    setLoading(false);
  };

  const handleEnable = async () => {
    if (!userId || !token) return;
    try {
      const res = await fetch(`${apiBase}/security/totp/enable/${userId}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token }),
      });
      setVerifyResult(await res.json());
      if ((await res.json()).success) {
        fetchStats();
      }
    } catch (err) {
      console.error('Failed to enable TOTP:', err);
    }
  };

  const handleVerify = async () => {
    if (!userId || !token) return;
    try {
      const res = await fetch(`${apiBase}/security/totp/verify/${userId}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token }),
      });
      setVerifyResult(await res.json());
    } catch (err) {
      console.error('Failed to verify TOTP:', err);
    }
  };

  const handleCheckUser = async () => {
    if (!userId) return;
    try {
      const res = await fetch(`${apiBase}/security/totp/info/${userId}`);
      setUserInfo(await res.json());
    } catch (err) {
      console.error('Failed to get user info:', err);
    }
  };

  const handleDisable = async () => {
    if (!userId) return;
    try {
      await fetch(`${apiBase}/security/totp/disable/${userId}`, { method: 'POST' });
      setVerifyResult({ success: true, message: 'TOTP 已禁用' });
      fetchStats();
    } catch (err) {
      console.error('Failed to disable TOTP:', err);
    }
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">双因素认证 (2FA)</h2>

        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.total_users || 0}</div>
            <div className="text-sm text-gray-400">总用户数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-400">{stats?.totp_enabled || 0}</div>
            <div className="text-sm text-gray-400">已启用 2FA</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-gray-400">{stats?.totp_disabled || 0}</div>
            <div className="text-sm text-gray-400">未启用 2FA</div>
          </div>
        </div>

        <div className="bg-gray-700 rounded-lg p-4 mb-6">
          <h3 className="text-lg font-semibold text-white mb-4">用户管理</h3>
          <div className="flex gap-4 mb-4">
            <input
              type="text"
              value={userId}
              onChange={(e) => setUserId(e.target.value)}
              placeholder="输入用户 ID"
              className="flex-1 bg-gray-600 text-white rounded px-4 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
              onClick={handleSetup}
              disabled={loading || !userId}
              className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white px-4 py-2 rounded"
            >
              设置 2FA
            </button>
            <button
              onClick={handleCheckUser}
              className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded"
            >
              查询状态
            </button>
          </div>

          {userInfo && (
            <div className="bg-gray-600 rounded p-3 mb-4">
              <div className="grid grid-cols-3 gap-4">
                <div>
                  <span className="text-gray-400 text-sm">状态: </span>
                  <span className={userInfo.enabled ? 'text-green-400' : 'text-red-400'}>
                    {userInfo.enabled ? '已启用' : '未启用'}
                  </span>
                </div>
                <div>
                  <span className="text-gray-400 text-sm">创建时间: </span>
                  <span className="text-white">{new Date(userInfo.created_at).toLocaleString()}</span>
                </div>
                <div>
                  <span className="text-gray-400 text-sm">备用码剩余: </span>
                  <span className="text-white">{userInfo.backup_codes_remaining}</span>
                </div>
              </div>
            </div>
          )}

          {setupData && (
            <div className="bg-gray-600 rounded p-4 mb-4">
              <h4 className="text-white font-semibold mb-3">📱 TOTP 设置</h4>
              <div className="space-y-3">
                <div>
                  <div className="text-sm text-gray-400">密钥 (手动输入):</div>
                  <div className="text-white font-mono bg-gray-700 p-2 rounded break-all">{setupData.manual_entry_key}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-400">备用码 (请妥善保存):</div>
                  <div className="grid grid-cols-2 gap-2">
                    {setupData.backup_codes.map((code, i) => (
                      <div key={i} className="text-white font-mono bg-gray-700 p-1 rounded text-center">{code}</div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}

          <div className="flex gap-4 items-center">
            <input
              type="text"
              value={token}
              onChange={(e) => setToken(e.target.value.replace(/\D/g, '').slice(0, 6))}
              placeholder="输入 6 位验证码"
              maxLength={6}
              className="flex-1 bg-gray-600 text-white rounded px-4 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
              onClick={handleEnable}
              className="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded"
            >
              启用
            </button>
            <button
              onClick={handleVerify}
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded"
            >
              验证
            </button>
            <button
              onClick={handleDisable}
              className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded"
            >
              禁用
            </button>
          </div>

          {verifyResult && (
            <div className={`mt-4 p-3 rounded ${verifyResult.success ? 'bg-green-900/30' : 'bg-red-900/30'}`}>
              <span className={verifyResult.success ? 'text-green-400' : 'text-red-400'}>
                {verifyResult.success ? '✅' : '❌'} {verifyResult.message}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

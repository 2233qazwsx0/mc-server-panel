import { useState, useEffect } from 'react';
import type { CertificateInfo, SslCertStats, CertificateExpiryStatus } from '../../types/security';

interface Props {
  apiBase: string;
}

export function SslCertPanel({ apiBase }: Props) {
  const [cert, setCert] = useState<CertificateInfo | null>(null);
  const [stats, setStats] = useState<SslCertStats | null>(null);
  const [expiryStatus, setExpiryStatus] = useState<CertificateExpiryStatus | null>(null);
  const [renewing, setRenewing] = useState(false);

  const fetchData = async () => {
    try {
      const [certRes, statsRes, expiryRes] = await Promise.all([
        fetch(`${apiBase}/security/ssl/cert`),
        fetch(`${apiBase}/security/ssl/stats`),
        fetch(`${apiBase}/security/ssl/check`),
      ]);
      const certData = await certRes.json();
      setCert(certData);
      setStats(await statsRes.json());
      setExpiryStatus(await expiryRes.json());
    } catch (err) {
      console.error('Failed to fetch SSL data:', err);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleRenew = async () => {
    setRenewing(true);
    try {
      await fetch(`${apiBase}/security/ssl/renew`, { method: 'POST' });
      setTimeout(fetchData, 2000);
    } catch (err) {
      console.error('Failed to renew certificate:', err);
    }
    setRenewing(false);
  };

  const getExpiryInfo = () => {
    if (!expiryStatus) return null;
    if ('Valid' in expiryStatus) {
      return { status: 'valid', days: expiryStatus.Valid.days_remaining };
    }
    if ('NeedsRenewal' in expiryStatus) {
      return { status: 'warning', days: expiryStatus.NeedsRenewal.days_remaining };
    }
    if (expiryStatus === 'Expired') {
      return { status: 'expired', days: 0 };
    }
    return null;
  };

  const expiryInfo = getExpiryInfo();

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">SSL 证书自动续期</h2>

        <div className="grid grid-cols-4 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className={`text-2xl font-bold ${
              expiryInfo?.status === 'valid' ? 'text-green-400' :
              expiryInfo?.status === 'warning' ? 'text-yellow-400' : 'text-red-400'
            }`}>
              {cert?.days_until_expiry ?? 'N/A'}
            </div>
            <div className="text-sm text-gray-400">剩余天数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.total_renewals || 0}</div>
            <div className="text-sm text-gray-400">续期总数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-400">{stats?.successful_renewals || 0}</div>
            <div className="text-sm text-gray-400">成功续期</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-red-400">{stats?.failed_renewals || 0}</div>
            <div className="text-sm text-gray-400">失败续期</div>
          </div>
        </div>

        {cert && (
          <div className="bg-gray-700 rounded-lg p-6 mb-6">
            <h3 className="text-lg font-semibold text-white mb-4">证书信息</h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="text-sm text-gray-400">主题</div>
                <div className="text-white">{cert.subject}</div>
              </div>
              <div>
                <div className="text-sm text-gray-400">颁发者</div>
                <div className="text-white">{cert.issuer}</div>
              </div>
              <div>
                <div className="text-sm text-gray-400">序列号</div>
                <div className="text-white font-mono text-sm">{cert.serial_number}</div>
              </div>
              <div>
                <div className="text-sm text-gray-400">有效期至</div>
                <div className="text-white">{new Date(cert.valid_until * 1000).toLocaleString()}</div>
              </div>
              <div className="col-span-2">
                <div className="text-sm text-gray-400">SHA-256 指纹</div>
                <div className="text-white font-mono text-xs break-all">{cert.fingerprint_sha256}</div>
              </div>
            </div>
          </div>
        )}

        <div className="flex gap-4">
          <button
            onClick={handleRenew}
            disabled={renewing}
            className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white px-6 py-2 rounded transition-colors"
          >
            {renewing ? '续期中...' : '手动续期证书'}
          </button>
          <button
            onClick={fetchData}
            className="bg-gray-600 hover:bg-gray-700 text-white px-6 py-2 rounded transition-colors"
          >
            刷新
          </button>
        </div>

        {expiryInfo?.status === 'expired' && (
          <div className="mt-4 p-4 bg-red-900/30 border border-red-700 rounded-lg">
            <div className="text-red-400 font-semibold">⚠️ 证书已过期！</div>
            <div className="text-sm text-gray-400 mt-1">请立即续期证书以确保安全连接。</div>
          </div>
        )}

        {expiryInfo?.status === 'warning' && (
          <div className="mt-4 p-4 bg-yellow-900/30 border border-yellow-700 rounded-lg">
            <div className="text-yellow-400 font-semibold">⚠️ 证书即将过期</div>
            <div className="text-sm text-gray-400 mt-1">证书将在 {expiryInfo.days} 天后过期，建议尽快续期。</div>
          </div>
        )}
      </div>
    </div>
  );
}

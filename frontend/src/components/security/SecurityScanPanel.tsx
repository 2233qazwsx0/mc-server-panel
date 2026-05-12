import { useState, useEffect } from 'react';
import type { SecurityBaseline, ScanReport, SecurityScanStats } from '../../types/security';

interface Props {
  apiBase: string;
}

export function SecurityScanPanel({ apiBase }: Props) {
  const [baselines, setBaselines] = useState<SecurityBaseline[]>([]);
  const [report, setReport] = useState<ScanReport | null>(null);
  const [stats, setStats] = useState<SecurityScanStats | null>(null);
  const [loading, setLoading] = useState(false);
  const [scanning, setScanning] = useState(false);

  const fetchData = async () => {
    setLoading(true);
    try {
      const [baselinesRes, reportRes, statsRes] = await Promise.all([
        fetch(`${apiBase}/security/scan/baselines`),
        fetch(`${apiBase}/security/scan/latest`),
        fetch(`${apiBase}/security/scan/stats`),
      ]);
      setBaselines(await baselinesRes.json());
      setReport(await reportRes.json());
      setStats(await statsRes.json());
    } catch (err) {
      console.error('Failed to fetch scan data:', err);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchData();
  }, []);

  const handleRunScan = async () => {
    setScanning(true);
    try {
      const res = await fetch(`${apiBase}/security/scan`, { method: 'POST' });
      setReport(await res.json());
      fetchData();
    } catch (err) {
      console.error('Failed to run scan:', err);
    }
    setScanning(false);
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'Critical': return 'text-red-400 bg-red-900/30 border-red-700';
      case 'High': return 'text-orange-400 bg-orange-900/30 border-orange-700';
      case 'Medium': return 'text-yellow-400 bg-yellow-900/30 border-yellow-700';
      case 'Low': return 'text-blue-400 bg-blue-900/30 border-blue-700';
      default: return 'text-gray-400 bg-gray-900/30 border-gray-700';
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 80) return 'text-green-400';
    if (score >= 60) return 'text-yellow-400';
    return 'text-red-400';
  };

  const getCategoryLabel = (category: string): string => {
    const labels: Record<string, string> = {
      Authentication: '认证安全',
      Authorization: '授权管理',
      Network: '网络安全',
      DataProtection: '数据保护',
      Configuration: '配置安全',
      Logging: '日志审计',
      Cryptography: '加密安全',
      Server: '服务器安全',
    };
    return labels[category] || category;
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">安全基线扫描</h2>

        <div className="grid grid-cols-4 gap-4 mb-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-blue-400">{stats?.total_baselines || 0}</div>
            <div className="text-sm text-gray-400">基线总数</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="text-2xl font-bold text-green-400">{stats?.enabled_baselines || 0}</div>
            <div className="text-sm text-gray-400">已启用</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className={`text-2xl font-bold ${getScoreColor(report?.overall_score || 0)}`}>
              {report?.overall_score || 0}%
            </div>
            <div className="text-sm text-gray-400">安全评分</div>
          </div>
          <div className="bg-gray-700 rounded-lg p-4 flex items-center justify-center">
            <button
              onClick={handleRunScan}
              disabled={scanning}
              className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white px-6 py-2 rounded transition-colors"
            >
              {scanning ? '扫描中...' : '运行扫描'}
            </button>
          </div>
        </div>

        {report && (
          <div className="bg-gray-700 rounded-lg p-4 mb-6">
            <h3 className="text-lg font-semibold text-white mb-4">扫描结果概览</h3>
            <div className="grid grid-cols-4 gap-4 mb-4">
              <div className="bg-green-900/30 border border-green-700 rounded p-3 text-center">
                <div className="text-2xl font-bold text-green-400">{report.passed}</div>
                <div className="text-sm text-gray-400">通过</div>
              </div>
              <div className="bg-red-900/30 border border-red-700 rounded p-3 text-center">
                <div className="text-2xl font-bold text-red-400">{report.failed}</div>
                <div className="text-sm text-gray-400">失败</div>
              </div>
              <div className="bg-yellow-900/30 border border-yellow-700 rounded p-3 text-center">
                <div className="text-2xl font-bold text-yellow-400">{report.warnings}</div>
                <div className="text-sm text-gray-400">警告</div>
              </div>
              <div className="bg-gray-600 rounded p-3 text-center">
                <div className="text-2xl font-bold text-gray-400">{report.total_checks}</div>
                <div className="text-sm text-gray-400">总检查项</div>
              </div>
            </div>

            {report.critical_findings.length > 0 && (
              <div className="space-y-3">
                <h4 className="text-red-400 font-semibold">⚠️ 严重问题</h4>
                {report.critical_findings.slice(0, 5).map((finding) => (
                  <div key={finding.id} className={`p-3 rounded border ${getSeverityColor(finding.severity)}`}>
                    <div className="font-semibold">{finding.baseline_name}</div>
                    <div className="text-sm mt-1">{finding.description}</div>
                    <div className="text-sm mt-2 bg-gray-800/50 p-2 rounded">
                      <span className="font-semibold">修复建议:</span> {finding.remediation}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        <div className="bg-gray-700 rounded-lg overflow-hidden">
          <div className="overflow-auto max-h-80">
            <table className="w-full text-sm">
              <thead className="bg-gray-600 sticky top-0">
                <tr>
                  <th className="px-4 py-2 text-left text-gray-300">基线名称</th>
                  <th className="px-4 py-2 text-left text-gray-300">分类</th>
                  <th className="px-4 py-2 text-left text-gray-300">严重性</th>
                  <th className="px-4 py-2 text-left text-gray-300">状态</th>
                  <th className="px-4 py-2 text-left text-gray-300">修复建议</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-600">
                {loading ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-8 text-center text-gray-500">加载中...</td>
                  </tr>
                ) : baselines.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-8 text-center text-gray-500">暂无基线</td>
                  </tr>
                ) : (
                  baselines.map((baseline) => (
                    <tr key={baseline.id} className="hover:bg-gray-600">
                      <td className="px-4 py-2 text-white">{baseline.name}</td>
                      <td className="px-4 py-2 text-gray-400">{getCategoryLabel(baseline.category)}</td>
                      <td className="px-4 py-2">
                        <span className={`px-2 py-1 rounded text-xs ${getSeverityColor(baseline.severity)}`}>
                          {baseline.severity === 'Critical' ? '严重' :
                           baseline.severity === 'High' ? '高' :
                           baseline.severity === 'Medium' ? '中' :
                           baseline.severity === 'Low' ? '低' : '信息'}
                        </span>
                      </td>
                      <td className="px-4 py-2">
                        <span className={`px-2 py-1 rounded text-xs ${baseline.enabled ? 'bg-green-900 text-green-400' : 'bg-gray-700 text-gray-400'}`}>
                          {baseline.enabled ? '已启用' : '已禁用'}
                        </span>
                      </td>
                      <td className="px-4 py-2 text-gray-400 text-xs max-w-xs truncate">{baseline.remediation}</td>
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

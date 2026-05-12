import { useState } from 'react';
import type { EncryptedData, HashResult } from '../../types/security';

interface Props {
  apiBase: string;
}

export function EncryptionPanel({ apiBase }: Props) {
  const [plaintext, setPlaintext] = useState('');
  const [encrypted, setEncrypted] = useState<EncryptedData | null>(null);
  const [decrypted, setDecrypted] = useState('');
  const [password, setPassword] = useState('');
  const [hashResult, setHashResult] = useState<HashResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleEncrypt = async () => {
    if (!plaintext) return;
    setLoading(true);
    setError('');
    try {
      const res = await fetch(`${apiBase}/security/encrypt`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ data: plaintext }),
      });
      setEncrypted(await res.json());
    } catch (err) {
      setError('加密失败');
    }
    setLoading(false);
  };

  const handleDecrypt = async () => {
    if (!encrypted) return;
    setLoading(true);
    setError('');
    try {
      const res = await fetch(`${apiBase}/security/decrypt`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ data: encrypted }),
      });
      const data = await res.json();
      if (data.success) {
        setDecrypted(data.data);
      } else {
        setError('解密失败');
      }
    } catch (err) {
      setError('解密失败');
    }
    setLoading(false);
  };

  const handleHash = async () => {
    if (!password) return;
    setLoading(true);
    setError('');
    try {
      const res = await fetch(`${apiBase}/security/hash`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ password }),
      });
      setHashResult(await res.json());
    } catch (err) {
      setError('哈希失败');
    }
    setLoading(false);
  };

  return (
    <div className="space-y-6">
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl font-bold text-white mb-4">敏感数据加密</h2>

        <div className="grid grid-cols-2 gap-6">
          <div className="bg-gray-700 rounded-lg p-4">
            <h3 className="text-lg font-semibold text-white mb-4">🔐 加密数据</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">要加密的文本</label>
                <textarea
                  value={plaintext}
                  onChange={(e) => setPlaintext(e.target.value)}
                  placeholder="输入要加密的敏感数据..."
                  rows={4}
                  className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                />
              </div>
              <button
                onClick={handleEncrypt}
                disabled={loading || !plaintext}
                className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white py-2 rounded transition-colors"
              >
                加密
              </button>
              {encrypted && (
                <div className="bg-gray-600 rounded p-3">
                  <div className="text-sm text-gray-400 mb-1">加密结果 (Base64):</div>
                  <div className="text-white font-mono text-xs break-all bg-gray-700 p-2 rounded">
                    {encrypted.ciphertext.slice(0, 64)}...
                  </div>
                  <div className="text-xs text-gray-500 mt-1">
                    算法: {encrypted.algorithm} | Nonce: {encrypted.nonce.slice(0, 16)}...
                  </div>
                </div>
              )}
            </div>
          </div>

          <div className="bg-gray-700 rounded-lg p-4">
            <h3 className="text-lg font-semibold text-white mb-4">🔓 解密数据</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">加密数据 (JSON)</label>
                <textarea
                  value={encrypted ? JSON.stringify(encrypted, null, 2) : ''}
                  onChange={(e) => {
                    try {
                      setEncrypted(JSON.parse(e.target.value));
                    } catch {}
                  }}
                  placeholder='{"ciphertext":"...", "nonce":"...", "algorithm":"...", "version":1}'
                  rows={4}
                  className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500 font-mono text-xs resize-none"
                />
              </div>
              <button
                onClick={handleDecrypt}
                disabled={loading || !encrypted}
                className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 text-white py-2 rounded transition-colors"
              >
                解密
              </button>
              {decrypted && (
                <div className="bg-gray-600 rounded p-3">
                  <div className="text-sm text-gray-400 mb-1">解密结果:</div>
                  <div className="text-white break-all bg-gray-700 p-2 rounded">{decrypted}</div>
                </div>
              )}
            </div>
          </div>
        </div>

        <div className="mt-6 bg-gray-700 rounded-lg p-4">
          <h3 className="text-lg font-semibold text-white mb-4">🔑 密码哈希</h3>
          <div className="flex gap-4">
            <div className="flex-1">
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="输入要哈希的密码"
                className="w-full bg-gray-600 text-white rounded px-3 py-2 focus outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <button
              onClick={handleHash}
              disabled={loading || !password}
              className="bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 text-white px-6 py-2 rounded"
            >
              生成哈希
            </button>
          </div>
          {hashResult && (
            <div className="mt-4 bg-gray-600 rounded p-3 space-y-2">
              <div>
                <div className="text-sm text-gray-400">哈希值:</div>
                <div className="text-white font-mono text-xs break-all bg-gray-700 p-2 rounded">
                  {hashResult.hash}
                </div>
              </div>
              <div>
                <div className="text-sm text-gray-400">盐值:</div>
                <div className="text-white font-mono text-xs break-all bg-gray-700 p-2 rounded">
                  {hashResult.salt}
                </div>
              </div>
              <div className="text-xs text-gray-500">
                算法: {hashResult.algorithm} | 版本: {hashResult.version}
              </div>
            </div>
          )}
        </div>

        {error && (
          <div className="mt-4 p-3 bg-red-900/30 border border-red-700 rounded">
            <span className="text-red-400">❌ {error}</span>
          </div>
        )}
      </div>
    </div>
  );
}

import React, { useState, useEffect } from 'react';
import { X, Minus, Plus, FileText, RefreshCw } from 'lucide-react';
import { DiffResult, DiffLine, InlineDiffResult, ApiResponse } from './types';
import { useFileManager } from './index';

const DiffViewer: React.FC = () => {
  const { showDiff, setShowDiff, diffResult, setDiffResult } = useFileManager();
  const [mode, setMode] = useState<'inline' | 'split'>('inline');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [path1, setPath1] = useState('');
  const [path2, setPath2] = useState('');
  const [diffLines, setDiffLines] = useState<DiffLine[]>([]);
  const [stats, setStats] = useState<{ additions: number; deletions: number; unchanged: number } | null>(null);

  useEffect(() => {
    if (diffResult) {
      setStats(diffResult.stats);
    }
  }, [diffResult]);

  const handleCompare = async () => {
    if (!path1.trim() || !path2.trim()) return;

    setLoading(true);
    setError(null);

    try {
      const url = mode === 'inline' 
        ? `/api/files/diff-inline?path1=${encodeURIComponent(path1)}&path2=${encodeURIComponent(path2)}`
        : `/api/files/diff?path1=${encodeURIComponent(path1)}&path2=${encodeURIComponent(path2)}`;

      const response = await fetch(url);
      const data: ApiResponse<DiffResult | InlineDiffResult> = await response.json();

      if (data.success && data.data) {
        if ('lines' in data.data) {
          setDiffLines((data.data as InlineDiffResult).lines);
        }
        if ('stats' in data.data) {
          setStats((data.data as DiffResult).stats);
        }
      } else {
        setError(data.error || 'Failed to compare files');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setShowDiff(false);
    setDiffResult(null);
    setDiffLines([]);
    setStats(null);
    setPath1('');
    setPath2('');
  };

  if (!showDiff) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50">
      <div className="bg-gray-900 rounded-lg shadow-xl w-full max-w-6xl h-full max-h-[90vh] flex flex-col m-8">
        <div className="flex items-center justify-between px-4 py-3 bg-gray-800 border-b border-gray-700">
          <div className="flex items-center space-x-4">
            <FileText className="w-5 h-5 text-purple-400" />
            <h2 className="text-lg font-semibold text-white">File Comparison</h2>
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={handleClose}
              className="p-1 hover:bg-gray-700 rounded"
            >
              <X className="w-5 h-5 text-gray-400" />
            </button>
          </div>
        </div>

        <div className="p-4 bg-gray-800 border-b border-gray-700">
          <div className="flex items-center space-x-4 mb-3">
            <div className="flex-1">
              <label className="block text-xs text-gray-400 mb-1">Original File</label>
              <input
                type="text"
                value={path1}
                onChange={(e) => setPath1(e.target.value)}
                placeholder="path/to/file1"
                className="w-full px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
              />
            </div>
            <div className="flex-1">
              <label className="block text-xs text-gray-400 mb-1">Modified File</label>
              <input
                type="text"
                value={path2}
                onChange={(e) => setPath2(e.target.value)}
                placeholder="path/to/file2"
                className="w-full px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
              />
            </div>
            <div className="flex items-end space-x-2">
              <button
                onClick={handleCompare}
                disabled={loading || !path1.trim() || !path2.trim()}
                className="px-4 py-2 bg-blue-600 text-white rounded text-sm disabled:opacity-50"
              >
                {loading ? <RefreshCw className="w-4 h-4 animate-spin" /> : 'Compare'}
              </button>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <label className="flex items-center space-x-2 text-sm">
                <input
                  type="radio"
                  name="mode"
                  checked={mode === 'inline'}
                  onChange={() => setMode('inline')}
                  className="text-blue-600"
                />
                <span className="text-gray-300">Inline</span>
              </label>
              <label className="flex items-center space-x-2 text-sm">
                <input
                  type="radio"
                  name="mode"
                  checked={mode === 'split'}
                  onChange={() => setMode('split')}
                  className="text-blue-600"
                />
                <span className="text-gray-300">Side by Side</span>
              </label>
            </div>

            {stats && (
              <div className="flex items-center space-x-3 text-sm">
                <span className="text-green-400">+{stats.additions}</span>
                <span className="text-red-400">-{stats.deletions}</span>
                <span className="text-gray-400">{stats.unchanged} unchanged</span>
              </div>
            )}
          </div>
        </div>

        {error && (
          <div className="px-4 py-2 bg-red-900 text-red-200 text-sm">
            {error}
          </div>
        )}

        <div className="flex-1 overflow-auto">
          {diffLines.length === 0 && !loading && (
            <div className="flex items-center justify-center h-full text-gray-400">
              Select two files to compare
            </div>
          )}

          {diffLines.length > 0 && (
            <div className="font-mono text-sm">
              {diffLines.map((line, idx) => (
                <div
                  key={idx}
                  className={`flex ${
                    line.change_type === 'delete' ? 'bg-red-900 bg-opacity-30' :
                    line.change_type === 'insert' ? 'bg-green-900 bg-opacity-30' :
                    'hover:bg-gray-800'
                  }`}
                >
                  <div className="w-12 px-2 py-0.5 text-right text-gray-500 border-r border-gray-700 select-none">
                    {line.line_number || ''}
                  </div>
                  <div className="w-8 px-2 py-0.5 text-center select-none">
                    {line.change_type === 'delete' ? <Minus className="w-3 h-3 text-red-400 inline" /> :
                     line.change_type === 'insert' ? <Plus className="w-3 h-3 text-green-400 inline" /> :
                     null}
                  </div>
                  <div className="flex-1 px-2 py-0.5 text-gray-300 whitespace-pre-wrap break-all">
                    {line.content}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default DiffViewer;

import React, { useState, useEffect } from 'react';
import { GitBranch, GitCommit, GitPullRequest, GitMerge, Plus, RefreshCw } from 'lucide-react';
import { GitRepository, GitStatus, GitCommit as GitCommitType, GitDiff, ApiResponse } from './types';

const GitPanel: React.FC = () => {
  const [repository, setRepository] = useState<GitRepository | null>(null);
  const [status, setStatus] = useState<GitStatus | null>(null);
  const [commits, setCommits] = useState<GitCommitType[]>([]);
  const [diffs, setDiffs] = useState<GitDiff[]>([]);
  const [loading, setLoading] = useState(false);
  const [commitMessage, setCommitMessage] = useState('');
  const [repoPath, setRepoPath] = useState('');
  const [activeTab, setActiveTab] = useState<'status' | 'commits' | 'diff' | 'branches'>('status');

  const initRepository = async () => {
    if (!repoPath.trim()) return;

    setLoading(true);
    try {
      const response = await fetch('/api/files/git/init', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          path: repoPath,
          name: repoPath.split('/').pop() || 'repository'
        })
      });
      
      const data: ApiResponse<GitRepository> = await response.json();
      if (data.success && data.data) {
        setRepository(data.data);
        loadStatus();
      }
    } catch (err) {
      console.error('Failed to init repository:', err);
    } finally {
      setLoading(false);
    }
  };

  const loadStatus = async () => {
    if (!repository) return;

    try {
      const response = await fetch(`/api/files/git/status?path=${encodeURIComponent(repository.path)}`);
      const data: ApiResponse<GitStatus> = await response.json();
      if (data.success && data.data) {
        setStatus(data.data);
      }
    } catch (err) {
      console.error('Failed to load status:', err);
    }
  };

  const loadCommits = async () => {
    if (!repository) return;

    try {
      const response = await fetch(`/api/files/git/log?path=${encodeURIComponent(repository.path)}&limit=20`);
      const data: ApiResponse<GitCommitType[]> = await response.json();
      if (data.success && data.data) {
        setCommits(data.data);
        setActiveTab('commits');
      }
    } catch (err) {
      console.error('Failed to load commits:', err);
    }
  };

  const loadDiffs = async () => {
    if (!repository) return;

    try {
      const response = await fetch(`/api/files/git/diff?path=${encodeURIComponent(repository.path)}`);
      const data: ApiResponse<GitDiff[]> = await response.json();
      if (data.success && data.data) {
        setDiffs(data.data);
        setActiveTab('diff');
      }
    } catch (err) {
      console.error('Failed to load diffs:', err);
    }
  };

  const handleCommit = async () => {
    if (!repository || !commitMessage.trim()) return;

    setLoading(true);
    try {
      const response = await fetch('/api/files/git/commit', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          path: repository.path,
          message: commitMessage
        })
      });
      
      const data: ApiResponse<GitCommitType> = await response.json();
      if (data.success) {
        setCommitMessage('');
        loadStatus();
        loadCommits();
      }
    } catch (err) {
      console.error('Failed to commit:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-2 mb-4">
          <GitBranch className="w-5 h-5 text-orange-400" />
          <h2 className="text-lg font-semibold text-white">Git Integration</h2>
        </div>

        {!repository && (
          <div className="flex space-x-2">
            <input
              type="text"
              value={repoPath}
              onChange={(e) => setRepoPath(e.target.value)}
              placeholder="path/to/repository"
              className="flex-1 px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
            />
            <button
              onClick={initRepository}
              disabled={loading || !repoPath.trim()}
              className="px-4 py-2 bg-orange-600 text-white rounded text-sm hover:bg-orange-700 disabled:opacity-50"
            >
              {loading ? 'Initializing...' : 'Initialize'}
            </button>
          </div>
        )}

        {repository && (
          <>
            <div className="flex items-center justify-between mb-4">
              <div className="text-sm">
                <span className="text-gray-400">Repository:</span>
                <span className="text-white ml-2">{repository.path}</span>
                <span className="ml-2 px-2 py-0.5 bg-gray-700 rounded text-xs text-gray-300">
                  {repository.branch}
                </span>
              </div>
              <button
                onClick={loadStatus}
                className="px-3 py-1 bg-gray-700 text-white rounded text-xs hover:bg-gray-600"
              >
                <RefreshCw className="w-3 h-3 inline mr-1" />
                Refresh
              </button>
            </div>

            <div className="flex space-x-2">
              <button
                onClick={() => setActiveTab('status')}
                className={`px-4 py-2 rounded text-sm ${
                  activeTab === 'status' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
                }`}
              >
                Status
              </button>
              <button
                onClick={loadCommits}
                className={`px-4 py-2 rounded text-sm ${
                  activeTab === 'commits' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
                }`}
              >
                Commits
              </button>
              <button
                onClick={loadDiffs}
                className={`px-4 py-2 rounded text-sm ${
                  activeTab === 'diff' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
                }`}
              >
                Diff
              </button>
            </div>
          </>
        )}
      </div>

      <div className="flex-1 overflow-auto p-4">
        {!repository && (
          <div className="text-center text-gray-400 py-8">
            Initialize a Git repository to start tracking changes
          </div>
        )}

        {repository && activeTab === 'status' && status && (
          <div className="space-y-4">
            <div className="bg-gray-800 rounded-lg p-4">
              <div className="flex items-center space-x-2 mb-3">
                <GitPullRequest className="w-4 h-4 text-green-400" />
                <h3 className="text-white font-medium">Staged Changes</h3>
              </div>
              {status.staged.length === 0 ? (
                <div className="text-gray-400 text-sm">No staged changes</div>
              ) : (
                <div className="space-y-1">
                  {status.staged.map((file, idx) => (
                    <div key={idx} className="flex items-center space-x-2 text-sm text-gray-300">
                      <span className="text-green-400">{file.status}</span>
                      <span>{file.path}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="text-white font-medium mb-3">Modified Files</h3>
              {status.modified.length === 0 ? (
                <div className="text-gray-400 text-sm">No modified files</div>
              ) : (
                <div className="space-y-1">
                  {status.modified.map((file, idx) => (
                    <div key={idx} className="flex items-center space-x-2 text-sm text-gray-300">
                      <span className="text-yellow-400">{file.status}</span>
                      <span>{file.path}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="text-white font-medium mb-3">Untracked Files</h3>
              {status.untracked.length === 0 ? (
                <div className="text-gray-400 text-sm">No untracked files</div>
              ) : (
                <div className="space-y-1">
                  {status.untracked.map((file, idx) => (
                    <div key={idx} className="text-sm text-gray-300">{file}</div>
                  ))}
                </div>
              )}
            </div>

            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="text-white font-medium mb-3">Commit</h3>
              <textarea
                value={commitMessage}
                onChange={(e) => setCommitMessage(e.target.value)}
                placeholder="Commit message..."
                className="w-full px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm mb-2"
                rows={3}
              />
              <button
                onClick={handleCommit}
                disabled={loading || !commitMessage.trim() || (status.staged.length === 0 && status.modified.length === 0 && status.untracked.length === 0)}
                className="px-4 py-2 bg-green-600 text-white rounded text-sm hover:bg-green-700 disabled:opacity-50"
              >
                <GitCommit className="w-4 h-4 inline mr-1" />
                Commit Changes
              </button>
            </div>
          </div>
        )}

        {repository && activeTab === 'commits' && (
          <div className="space-y-2">
            {commits.map(commit => (
              <div key={commit.hash} className="bg-gray-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-blue-400 font-mono text-sm">{commit.short_hash}</span>
                  <span className="text-gray-400 text-sm">{commit.date}</span>
                </div>
                <div className="text-white mb-2">{commit.message}</div>
                <div className="text-gray-400 text-sm">
                  {commit.author} ({commit.email})
                </div>
              </div>
            ))}
          </div>
        )}

        {repository && activeTab === 'diff' && (
          <div className="space-y-4">
            {diffs.map((diff, idx) => (
              <div key={idx} className="bg-gray-800 rounded-lg p-4">
                <h3 className="text-white font-medium mb-2">{diff.file}</h3>
                <div className="font-mono text-sm">
                  {diff.hunks.map((hunk, hIdx) => (
                    <div key={hIdx} className="mb-2">
                      <div className="text-gray-500 text-xs mb-1">
                        @@ -{hunk.old_start},{hunk.old_lines} +{hunk.new_start},{hunk.new_lines} @@
                      </div>
                      {hunk.lines.slice(0, 20).map((line, lIdx) => (
                        <div
                          key={lIdx}
                          className={`${
                            line.line_type === 'add' ? 'bg-green-900 bg-opacity-30 text-green-300' :
                            line.line_type === 'delete' ? 'bg-red-900 bg-opacity-30 text-red-300' :
                            'text-gray-300'
                          }`}
                        >
                          {line.line_type === 'add' ? '+' : line.line_type === 'delete' ? '-' : ' '}
                          {line.content}
                        </div>
                      ))}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default GitPanel;

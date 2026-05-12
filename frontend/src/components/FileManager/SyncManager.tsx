import React, { useState, useEffect } from 'react';
import { RefreshCw, Play, Eye, Trash2, Plus } from 'lucide-react';
import { SyncConfig, SyncPreview, SyncEvent, ApiResponse } from './types';

const SyncManager: React.FC = () => {
  const [configs, setConfigs] = useState<SyncConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedConfig, setSelectedConfig] = useState<SyncConfig | null>(null);
  const [preview, setPreview] = useState<SyncPreview | null>(null);
  const [history, setHistory] = useState<SyncEvent[]>([]);
  const [activeTab, setActiveTab] = useState<'configs' | 'preview' | 'history'>('configs');

  useEffect(() => {
    loadConfigs();
  }, []);

  const loadConfigs = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/files/sync/configs');
      const data: ApiResponse<SyncConfig[]> = await response.json();
      if (data.success && data.data) {
        setConfigs(data.data);
      }
    } catch (err) {
      console.error('Failed to load sync configs:', err);
    } finally {
      setLoading(false);
    }
  };

  const handlePreview = async (configId: string) => {
    try {
      const response = await fetch(`/api/files/sync/preview/${configId}`);
      const data: ApiResponse<SyncPreview> = await response.json();
      if (data.success && data.data) {
        setPreview(data.data);
        setActiveTab('preview');
      }
    } catch (err) {
      console.error('Failed to preview sync:', err);
    }
  };

  const handleSync = async (configId: string) => {
    try {
      const response = await fetch(`/api/files/sync/execute/${configId}`, {
        method: 'POST'
      });
      const data: ApiResponse<SyncEvent> = await response.json();
      if (data.success) {
        loadConfigs();
        loadHistory(configId);
      }
    } catch (err) {
      console.error('Failed to execute sync:', err);
    }
  };

  const loadHistory = async (configId: string) => {
    try {
      const response = await fetch(`/api/files/sync/history/${configId}`);
      const data: ApiResponse<SyncEvent[]> = await response.json();
      if (data.success && data.data) {
        setHistory(data.data);
        setActiveTab('history');
      }
    } catch (err) {
      console.error('Failed to load sync history:', err);
    }
  };

  const handleDelete = async (configId: string) => {
    try {
      const response = await fetch(`/api/files/sync/configs/${configId}`, {
        method: 'DELETE'
      });
      const data: ApiResponse<boolean> = await response.json();
      if (data.success) {
        loadConfigs();
      }
    } catch (err) {
      console.error('Failed to delete sync config:', err);
    }
  };

  if (loading) {
    return <div className="flex items-center justify-center h-full text-gray-400">Loading...</div>;
  }

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Remote File Sync</h2>
          <button className="px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700">
            <Plus className="w-4 h-4 inline mr-1" />
            New Config
          </button>
        </div>
        
        <div className="flex space-x-2">
          <button
            onClick={() => setActiveTab('configs')}
            className={`px-4 py-2 rounded text-sm ${
              activeTab === 'configs' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
            }`}
          >
            Configs
          </button>
          <button
            onClick={() => setActiveTab('preview')}
            disabled={!preview}
            className="px-4 py-2 rounded text-sm bg-gray-700 text-gray-300 disabled:opacity-50"
          >
            Preview
          </button>
          <button
            onClick={() => setActiveTab('history')}
            disabled={history.length === 0}
            className="px-4 py-2 rounded text-sm bg-gray-700 text-gray-300 disabled:opacity-50"
          >
            History
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {activeTab === 'configs' && (
          <div className="space-y-2">
            {configs.map(config => (
              <div key={config.id} className="bg-gray-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <h3 className="text-white font-medium">{config.name}</h3>
                  <span className={`px-2 py-1 rounded text-xs ${
                    config.status === 'Idle' ? 'bg-gray-600' :
                    config.status === 'Syncing' ? 'bg-blue-600' :
                    config.status === 'Completed' ? 'bg-green-600' : 'bg-red-600'
                  } text-white`}>
                    {config.status}
                  </span>
                </div>
                <div className="text-sm text-gray-400 mb-3">
                  {config.source.host} → {config.target.host} ({config.direction})
                </div>
                <div className="flex space-x-2">
                  <button
                    onClick={() => handlePreview(config.id)}
                    className="px-3 py-1 bg-gray-700 text-white rounded text-xs hover:bg-gray-600"
                  >
                    <Eye className="w-3 h-3 inline mr-1" />
                    Preview
                  </button>
                  <button
                    onClick={() => handleSync(config.id)}
                    disabled={config.status === 'Syncing'}
                    className="px-3 py-1 bg-blue-600 text-white rounded text-xs hover:bg-blue-700 disabled:opacity-50"
                  >
                    <Play className="w-3 h-3 inline mr-1" />
                    Sync
                  </button>
                  <button
                    onClick={() => handleDelete(config.id)}
                    className="px-3 py-1 bg-red-600 text-white rounded text-xs hover:bg-red-700"
                  >
                    <Trash2 className="w-3 h-3 inline mr-1" />
                    Delete
                  </button>
                </div>
              </div>
            ))}
            
            {configs.length === 0 && (
              <div className="text-center text-gray-400 py-8">
                No sync configurations
              </div>
            )}
          </div>
        )}

        {activeTab === 'preview' && preview && (
          <div className="space-y-4">
            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="text-white font-medium mb-2">To Upload ({preview.to_upload.length})</h3>
              {preview.to_upload.length === 0 ? (
                <div className="text-gray-400 text-sm">No files to upload</div>
              ) : (
                <div className="space-y-1">
                  {preview.to_upload.map((file, idx) => (
                    <div key={idx} className="text-sm text-gray-300">+ {file.path}</div>
                  ))}
                </div>
              )}
            </div>

            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="text-white font-medium mb-2">To Download ({preview.to_download.length})</h3>
              {preview.to_download.length === 0 ? (
                <div className="text-gray-400 text-sm">No files to download</div>
              ) : (
                <div className="space-y-1">
                  {preview.to_download.map((file, idx) => (
                    <div key={idx} className="text-sm text-gray-300">- {file.path}</div>
                  ))}
                </div>
              )}
            </div>

            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="text-white font-medium mb-2">Conflicts ({preview.conflicts.length})</h3>
              {preview.conflicts.length === 0 ? (
                <div className="text-green-400 text-sm">No conflicts</div>
              ) : (
                <div className="space-y-2">
                  {preview.conflicts.map((conflict, idx) => (
                    <div key={idx} className="bg-red-900 bg-opacity-30 rounded p-2 text-sm">
                      <div className="text-red-300 font-medium">{conflict.path}</div>
                      <div className="text-gray-400 text-xs">Both modified</div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'history' && (
          <div className="space-y-2">
            {history.map(event => (
              <div key={event.id} className="bg-gray-800 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <span className={`px-2 py-1 rounded text-xs ${
                    event.success ? 'bg-green-600' : 'bg-red-600'
                  } text-white`}>
                    {event.event_type}
                  </span>
                  <span className="text-gray-400 text-sm">{event.timestamp}</span>
                </div>
                <div className="text-sm text-gray-300">
                  Duration: {(event.duration_ms / 1000).toFixed(2)}s
                </div>
                {event.error_message && (
                  <div className="text-red-400 text-sm mt-1">{event.error_message}</div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default SyncManager;

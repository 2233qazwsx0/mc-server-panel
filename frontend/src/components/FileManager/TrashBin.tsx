import React, { useState, useEffect } from 'react';
import { Trash2, RotateCcw, AlertTriangle, RefreshCw, Search, X } from 'lucide-react';
import { TrashItem, TrashList, RestoreResult, ApiResponse } from './types';

const TrashBin: React.FC = () => {
  const [trashList, setTrashList] = useState<TrashList | null>(null);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [filteredItems, setFilteredItems] = useState<TrashItem[]>([]);

  useEffect(() => {
    loadTrash();
  }, []);

  useEffect(() => {
    if (trashList) {
      if (searchQuery.trim()) {
        setFilteredItems(
          trashList.items.filter(item =>
            item.original_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            item.original_path.toLowerCase().includes(searchQuery.toLowerCase())
          )
        );
      } else {
        setFilteredItems(trashList.items);
      }
    }
  }, [trashList, searchQuery]);

  const loadTrash = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/files/trash/list');
      const data: ApiResponse<TrashList> = await response.json();
      if (data.success && data.data) {
        setTrashList(data.data);
      }
    } catch (err) {
      console.error('Failed to load trash:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleRestore = async (itemId: string) => {
    try {
      const response = await fetch(`/api/files/trash/restore/${itemId}`, {
        method: 'POST'
      });
      const data: ApiResponse<RestoreResult> = await response.json();
      if (data.success && data.data) {
        if (data.data.warnings.length > 0) {
          alert(`Restored with warnings:\n${data.data.warnings.join('\n')}`);
        }
        loadTrash();
      }
    } catch (err) {
      console.error('Failed to restore:', err);
    }
  };

  const handlePermanentDelete = async (itemId: string) => {
    if (!confirm('This action cannot be undone. Are you sure?')) return;

    try {
      const response = await fetch(`/api/files/trash/purge/${itemId}`, {
        method: 'DELETE'
      });
      const data: ApiResponse<boolean> = await response.json();
      if (data.success) {
        loadTrash();
      }
    } catch (err) {
      console.error('Failed to permanently delete:', err);
    }
  };

  const handleEmptyTrash = async () => {
    if (!confirm('This will permanently delete ALL items in trash. This cannot be undone!')) return;

    try {
      const response = await fetch('/api/files/trash/empty', {
        method: 'POST'
      });
      const data: ApiResponse<TrashList> = await response.json();
      if (data.success) {
        loadTrash();
      }
    } catch (err) {
      console.error('Failed to empty trash:', err);
    }
  };

  const handleCleanupExpired = async () => {
    try {
      const response = await fetch('/api/files/trash/cleanup', {
        method: 'POST'
      });
      if (response.ok) {
        loadTrash();
      }
    } catch (err) {
      console.error('Failed to cleanup expired:', err);
    }
  };

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  const daysUntilExpiry = (dateStr: string): number => {
    const expiry = new Date(dateStr);
    const now = new Date();
    const diff = expiry.getTime() - now.getTime();
    return Math.ceil(diff / (1000 * 60 * 60 * 24));
  };

  if (loading) {
    return <div className="flex items-center justify-center h-full text-gray-400">Loading...</div>;
  }

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-2">
            <Trash2 className="w-5 h-5 text-yellow-400" />
            <h2 className="text-lg font-semibold text-white">Trash</h2>
            {trashList && (
              <span className="px-2 py-1 bg-gray-700 rounded text-xs text-gray-300">
                {trashList.item_count} items
              </span>
            )}
          </div>
          <div className="flex space-x-2">
            <button
              onClick={handleCleanupExpired}
              className="px-3 py-1.5 bg-gray-700 text-white rounded text-xs hover:bg-gray-600"
            >
              <RefreshCw className="w-3 h-3 inline mr-1" />
              Cleanup Expired
            </button>
            <button
              onClick={handleEmptyTrash}
              disabled={!trashList || trashList.item_count === 0}
              className="px-3 py-1.5 bg-red-600 text-white rounded text-xs hover:bg-red-700 disabled:opacity-50"
            >
              <AlertTriangle className="w-3 h-3 inline mr-1" />
              Empty Trash
            </button>
          </div>
        </div>

        {trashList && (
          <div className="flex items-center space-x-4 text-sm text-gray-400 mb-4">
            <span>Total: {formatSize(trashList.total_size)}</span>
            {trashList.oldest_item && (
              <span>Oldest: {formatDate(trashList.oldest_item)}</span>
            )}
            {trashList.newest_item && (
              <span>Newest: {formatDate(trashList.newest_item)}</span>
            )}
          </div>
        )}

        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search in trash..."
            className="w-full pl-10 pr-4 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
          />
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {filteredItems.length === 0 ? (
          <div className="text-center text-gray-400 py-8">
            {searchQuery ? 'No items match your search' : 'Trash is empty'}
          </div>
        ) : (
          <div className="space-y-2">
            {filteredItems.map(item => {
              const daysLeft = daysUntilExpiry(item.expires_at);
              const isExpiringSoon = daysLeft <= 3;

              return (
                <div
                  key={item.id}
                  className={`bg-gray-800 rounded-lg p-4 ${
                    isExpiringSoon ? 'border-l-4 border-yellow-500' : ''
                  }`}
                >
                  <div className="flex items-start justify-between mb-2">
                    <div>
                      <h3 className="text-white font-medium">{item.original_name}</h3>
                      <p className="text-gray-400 text-sm">{item.original_path}</p>
                    </div>
                    <div className="flex space-x-2">
                      <button
                        onClick={() => handleRestore(item.id)}
                        className="px-3 py-1 bg-green-600 text-white rounded text-xs hover:bg-green-700"
                      >
                        <RotateCcw className="w-3 h-3 inline mr-1" />
                        Restore
                      </button>
                      <button
                        onClick={() => handlePermanentDelete(item.id)}
                        className="px-3 py-1 bg-red-600 text-white rounded text-xs hover:bg-red-700"
                      >
                        <X className="w-3 h-3 inline mr-1" />
                        Delete
                      </button>
                    </div>
                  </div>
                  <div className="flex items-center justify-between text-sm text-gray-400">
                    <div className="flex space-x-4">
                      <span>Deleted: {formatDate(item.deleted_at)}</span>
                      <span>Size: {formatSize(item.size)}</span>
                      <span>Type: {item.file_type}</span>
                    </div>
                    <div className={`flex items-center space-x-1 ${
                      isExpiringSoon ? 'text-yellow-400' : 'text-gray-400'
                    }`}>
                      <AlertTriangle className="w-3 h-3" />
                      <span>
                        {daysLeft > 0
                          ? `Expires in ${daysLeft} days`
                          : 'Expiring soon'}
                      </span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

export default TrashBin;

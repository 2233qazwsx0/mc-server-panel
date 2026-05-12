import React, { useState, useEffect } from 'react';
import { Folder, File, ChevronRight, ChevronDown, RefreshCw, Trash2, Copy, Edit, Download, Upload } from 'lucide-react';
import { FileEntry, ApiResponse } from './types';

interface FileListProps {
  currentPath: string;
  onPathChange: (path: string) => void;
  onFileSelect: (file: FileEntry) => void;
}

const FileList: React.FC<FileListProps> = ({ currentPath, onPathChange, onFileSelect }) => {
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; file: FileEntry } | null>(null);

  useEffect(() => {
    fetchFiles();
  }, [currentPath]);

  const fetchFiles = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await fetch('/api/files/list', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: currentPath, include_hidden: false })
      });
      
      const data: ApiResponse<FileEntry[]> = await response.json();
      
      if (data.success && data.data) {
        setFiles(data.data);
      } else {
        setError(data.error || 'Failed to load files');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleFileClick = (file: FileEntry) => {
    if (file.is_directory) {
      onPathChange(file.path);
    } else {
      onFileSelect(file);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, file: FileEntry) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, file });
  };

  const handleSelect = (e: React.MouseEvent, file: FileEntry) => {
    e.stopPropagation();
    
    if (e.ctrlKey || e.metaKey) {
      const newSelected = new Set(selectedFiles);
      if (newSelected.has(file.path)) {
        newSelected.delete(file.path);
      } else {
        newSelected.add(file.path);
      }
      setSelectedFiles(newSelected);
    } else {
      setSelectedFiles(new Set([file.path]));
    }
  };

  const navigateUp = () => {
    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    onPathChange(parts.join('/'));
  };

  const getFileIcon = (file: FileEntry) => {
    if (file.is_directory) {
      return <Folder className="w-5 h-5 text-yellow-400" />;
    }
    
    const ext = file.name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'yml':
      case 'yaml':
        return <File className="w-5 h-5 text-pink-400" />;
      case 'json':
        return <File className="w-5 h-5 text-yellow-300" />;
      case 'properties':
        return <File className="w-5 h-5 text-blue-300" />;
      case 'log':
        return <File className="w-5 h-5 text-gray-400" />;
      default:
        return <File className="w-5 h-5 text-gray-300" />;
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

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <RefreshCw className="w-8 h-8 text-blue-500 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-red-400">{error}</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center space-x-2 px-4 py-2 bg-gray-800 border-b border-gray-700">
        <button
          onClick={navigateUp}
          disabled={!currentPath}
          className="px-3 py-1 bg-gray-700 text-white rounded text-sm disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Parent
        </button>
        <div className="flex-1 px-3 py-1 bg-gray-900 text-gray-300 rounded text-sm font-mono">
          /{currentPath}
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        <table className="w-full">
          <thead className="bg-gray-800 sticky top-0">
            <tr className="text-left text-gray-400 text-sm">
              <th className="px-4 py-2 w-8"></th>
              <th className="px-4 py-2">Name</th>
              <th className="px-4 py-2 w-24">Size</th>
              <th className="px-4 py-2 w-48">Modified</th>
              <th className="px-4 py-2 w-20">Permissions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800">
            {files.map((file) => (
              <tr
                key={file.path}
                onClick={() => handleFileClick(file)}
                onContextMenu={(e) => handleContextMenu(e, file)}
                onDoubleClick={() => handleFileClick(file)}
                className={`hover:bg-gray-800 cursor-pointer transition-colors ${
                  selectedFiles.has(file.path) ? 'bg-blue-900 bg-opacity-30' : ''
                }`}
              >
                <td className="px-4 py-2">
                  {file.is_directory ? (
                    expandedDirs.has(file.path) ? (
                      <ChevronDown className="w-4 h-4 text-gray-400" />
                    ) : (
                      <ChevronRight className="w-4 h-4 text-gray-400" />
                    )
                  ) : null}
                </td>
                <td className="px-4 py-2">
                  <div className="flex items-center space-x-2">
                    {getFileIcon(file)}
                    <span className="text-white">{file.name}</span>
                  </div>
                </td>
                <td className="px-4 py-2 text-gray-400 text-sm">
                  {file.is_directory ? '--' : formatSize(file.size)}
                </td>
                <td className="px-4 py-2 text-gray-400 text-sm">
                  {formatDate(file.modified)}
                </td>
                <td className="px-4 py-2 text-gray-400 text-sm font-mono">
                  {file.permissions}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {contextMenu && (
        <>
          <div
            className="fixed inset-0"
            onClick={() => setContextMenu(null)}
          />
          <div
            className="fixed bg-gray-800 border border-gray-700 rounded-lg shadow-xl py-1 z-50"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            <button
              onClick={() => {
                onFileSelect(contextMenu.file);
                setContextMenu(null);
              }}
              className="w-full px-4 py-2 text-left text-gray-300 hover:bg-gray-700 flex items-center space-x-2"
            >
              <Edit className="w-4 h-4" />
              <span>Edit</span>
            </button>
            <button
              onClick={() => {
                navigator.clipboard.writeText(contextMenu.file.path);
                setContextMenu(null);
              }}
              className="w-full px-4 py-2 text-left text-gray-300 hover:bg-gray-700 flex items-center space-x-2"
            >
              <Copy className="w-4 h-4" />
              <span>Copy Path</span>
            </button>
            <button
              className="w-full px-4 py-2 text-left text-gray-300 hover:bg-gray-700 flex items-center space-x-2"
            >
              <Download className="w-4 h-4" />
              <span>Download</span>
            </button>
            <button
              onClick={() => {
                deleteFile(contextMenu.file.path);
                setContextMenu(null);
              }}
              className="w-full px-4 py-2 text-left text-red-400 hover:bg-gray-700 flex items-center space-x-2"
            >
              <Trash2 className="w-4 h-4" />
              <span>Delete</span>
            </button>
          </div>
        </>
      )}
    </div>
  );
};

const deleteFile = async (path: string) => {
  try {
    const response = await fetch(`/api/files/delete?path=${encodeURIComponent(path)}`, {
      method: 'DELETE'
    });
    const data: ApiResponse<boolean> = await response.json();
    if (!data.success) {
      console.error('Delete failed:', data.error);
    }
  } catch (err) {
    console.error('Delete error:', err);
  }
};

export default FileList;

import React, { createContext, useContext, useState, useCallback, useEffect } from 'react';
import FileList from './FileList';
import FileEditor from './FileEditor';
import DiffViewer from './DiffViewer';
import SearchPanel from './SearchPanel';
import ArchiveManager from './ArchiveManager';
import AclManager from './AclManager';
import FileUploader from './FileUploader';
import TrashBin from './TrashBin';
import SyncManager from './SyncManager';
import GitPanel from './GitPanel';
import FileValidator from './FileValidator';
import { FileEntry, SearchResult, DiffResult, ValidationResult, ApiResponse } from '../types';

interface FileManagerContextType {
  currentPath: string;
  setCurrentPath: (path: string) => void;
  selectedFiles: string[];
  setSelectedFiles: (files: string[]) => void;
  refreshFiles: () => void;
  files: FileEntry[];
  loading: boolean;
  error: string | null;
  searchResults: SearchResult[];
  setSearchResults: (results: SearchResult[]) => void;
  showSearch: boolean;
  setShowSearch: (show: boolean) => void;
  showDiff: boolean;
  setShowDiff: (show: boolean) => void;
  diffResult: DiffResult | null;
  setDiffResult: (result: DiffResult | null) => void;
  editingFile: string | null;
  setEditingFile: (path: string | null) => void;
}

const FileManagerContext = createContext<FileManagerContextType | null>(null);

export const useFileManager = () => {
  const context = useContext(FileManagerContext);
  if (!context) {
    throw new Error('useFileManager must be used within a FileManagerProvider');
  }
  return context;
};

export const FileManagerProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [currentPath, setCurrentPath] = useState<string>('');
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [showSearch, setShowSearch] = useState(false);
  const [showDiff, setShowDiff] = useState(false);
  const [diffResult, setDiffResult] = useState<DiffResult | null>(null);
  const [editingFile, setEditingFile] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  const refreshFiles = useCallback(() => {
    setRefreshKey(k => k + 1);
  }, []);

  useEffect(() => {
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

    fetchFiles();
  }, [currentPath, refreshKey]);

  return (
    <FileManagerContext.Provider
      value={{
        currentPath,
        setCurrentPath,
        selectedFiles,
        setSelectedFiles,
        refreshFiles,
        files,
        loading,
        error,
        searchResults,
        setSearchResults,
        showSearch,
        setShowSearch,
        showDiff,
        setShowDiff,
        diffResult,
        setDiffResult,
        editingFile,
        setEditingFile,
      }}
    >
      {children}
    </FileManagerContext.Provider>
  );
};

const FileManager: React.FC = () => {
  const {
    currentPath,
    setCurrentPath,
    showSearch,
    setShowSearch,
    showDiff,
    editingFile,
    setEditingFile,
    refreshFiles,
  } = useFileManager();

  const [activeTab, setActiveTab] = useState<'files' | 'search' | 'archives' | 'acl' | 'sync' | 'git' | 'trash' | 'validator'>('files');
  const [showUploader, setShowUploader] = useState(false);

  const handlePathChange = (newPath: string) => {
    setCurrentPath(newPath);
  };

  const handleFileSelect = (file: FileEntry) => {
    if (file.is_directory) {
      setCurrentPath(file.path);
    } else {
      setEditingFile(file.path);
    }
  };

  const handleCloseEditor = () => {
    setEditingFile(null);
    refreshFiles();
  };

  return (
    <div className="h-full flex flex-col bg-gray-900">
      <div className="flex items-center justify-between px-4 py-3 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-4">
          <h1 className="text-xl font-bold text-white">File Manager</h1>
          <div className="flex items-center space-x-1">
            <button
              onClick={() => setActiveTab('files')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'files'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-700'
              }`}
            >
              Files
            </button>
            <button
              onClick={() => setActiveTab('search')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'search'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-700'
              }`}
            >
              Search
            </button>
            <button
              onClick={() => setActiveTab('archives')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'archives'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-700'
              }`}
            >
              Archives
            </button>
            <button
              onClick={() => setActiveTab('acl')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'acl'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-700'
              }`}
            >
              Permissions
            </button>
            <button
              onClick={() => setActiveTab('sync')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'sync'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-700'
              }`}
            >
              Sync
            </button>
            <button
              onClick={() => setActiveTab('git')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'git'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-700'
              }`}
            >
              Git
            </button>
            <button
              onClick={() => setActiveTab('trash')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'trash'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-700'
              }`}
            >
              Trash
            </button>
            <button
              onClick={() => setActiveTab('validator')}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                activeTab === 'validator'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-700'
              }`}
            >
              Validator
            </button>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          <button
            onClick={() => setShowUploader(true)}
            className="px-3 py-1.5 bg-green-600 text-white rounded-md text-sm font-medium hover:bg-green-700 transition-colors"
          >
            Upload
          </button>
          <button
            onClick={refreshFiles}
            className="px-3 py-1.5 bg-gray-700 text-white rounded-md text-sm font-medium hover:bg-gray-600 transition-colors"
          >
            Refresh
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-hidden">
        {activeTab === 'files' && !editingFile && (
          <FileList
            currentPath={currentPath}
            onPathChange={handlePathChange}
            onFileSelect={handleFileSelect}
          />
        )}
        {activeTab === 'files' && editingFile && (
          <FileEditor
            filePath={editingFile}
            onClose={handleCloseEditor}
          />
        )}
        {activeTab === 'search' && <SearchPanel />}
        {activeTab === 'archives' && <ArchiveManager />}
        {activeTab === 'acl' && <AclManager />}
        {activeTab === 'sync' && <SyncManager />}
        {activeTab === 'git' && <GitPanel />}
        {activeTab === 'trash' && <TrashBin />}
        {activeTab === 'validator' && <FileValidator />}
      </div>

      {showUploader && (
        <FileUploader onClose={() => setShowUploader(false)} />
      )}

      {showDiff && <DiffViewer />}
    </div>
  );
};

export default FileManager;

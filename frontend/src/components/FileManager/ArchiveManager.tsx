import React, { useState } from 'react';
import { Archive, FileText, Download, Upload, Folder } from 'lucide-react';
import { ArchiveRequest, ArchiveResult, ArchiveEntry, ExtractResult, ApiResponse } from './types';

const ArchiveManager: React.FC = () => {
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [outputName, setOutputName] = useState('');
  const [format, setFormat] = useState<'Zip' | 'Tar' | 'TarGz'>('Zip');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ArchiveResult | null>(null);
  const [extractPath, setExtractPath] = useState('');
  const [archiveList, setArchiveList] = useState<ArchiveEntry[]>([]);
  const [activeTab, setActiveTab] = useState<'create' | 'extract'>('create');

  const handleCreateArchive = async () => {
    if (selectedFiles.length === 0 || !outputName.trim()) return;

    setLoading(true);
    
    const request: ArchiveRequest = {
      files: selectedFiles,
      output_name: outputName,
      format: format
    };

    try {
      const response = await fetch('/api/files/archive/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request)
      });
      
      const data: ApiResponse<ArchiveResult> = await response.json();
      
      if (data.success && data.data) {
        setResult(data.data);
      }
    } catch (err) {
      console.error('Archive creation error:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleExtract = async () => {
    if (!extractPath.trim()) return;

    setLoading(true);
    
    try {
      const response = await fetch('/api/files/archive/extract', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          archive_path: extractPath,
          destination: extractPath.replace(/\.(zip|tar|tar\.gz)$/i, '_extracted')
        })
      });
      
      const data: ApiResponse<ExtractResult> = await response.json();
      
      if (data.success) {
        alert('Extraction completed successfully');
      }
    } catch (err) {
      console.error('Extraction error:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-2 mb-4">
          <Archive className="w-5 h-5 text-purple-400" />
          <h2 className="text-lg font-semibold text-white">Archive Manager</h2>
        </div>

        <div className="flex space-x-2">
          <button
            onClick={() => setActiveTab('create')}
            className={`px-4 py-2 rounded text-sm font-medium ${
              activeTab === 'create' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
            }`}
          >
            Create Archive
          </button>
          <button
            onClick={() => setActiveTab('extract')}
            className={`px-4 py-2 rounded text-sm font-medium ${
              activeTab === 'extract' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
            }`}
          >
            Extract Archive
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {activeTab === 'create' && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Selected Files</label>
              <div className="bg-gray-800 rounded p-4 min-h-[150px]">
                {selectedFiles.length === 0 ? (
                  <div className="text-gray-500">No files selected</div>
                ) : (
                  <div className="space-y-1">
                    {selectedFiles.map((file, idx) => (
                      <div key={idx} className="flex items-center space-x-2 text-sm text-gray-300">
                        <FileText className="w-4 h-4" />
                        <span>{file}</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
              <p className="text-xs text-gray-500 mt-1">
                Enter file paths separated by commas
              </p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-gray-400 mb-2">Output Name</label>
                <input
                  type="text"
                  value={outputName}
                  onChange={(e) => setOutputName(e.target.value)}
                  placeholder="backup"
                  className="w-full px-3 py-2 bg-gray-800 text-white rounded border border-gray-700"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-2">Format</label>
                <select
                  value={format}
                  onChange={(e) => setFormat(e.target.value as 'Zip' | 'Tar' | 'TarGz')}
                  className="w-full px-3 py-2 bg-gray-800 text-white rounded border border-gray-700"
                >
                  <option value="Zip">ZIP (.zip)</option>
                  <option value="Tar">TAR (.tar)</option>
                  <option value="TarGz">TAR.GZ (.tar.gz)</option>
                </select>
              </div>
            </div>

            <button
              onClick={handleCreateArchive}
              disabled={loading || selectedFiles.length === 0 || !outputName.trim()}
              className="w-full px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50"
            >
              {loading ? 'Creating...' : 'Create Archive'}
            </button>

            {result && (
              <div className="bg-green-900 bg-opacity-30 border border-green-700 rounded p-4">
                <h3 className="text-green-400 font-medium mb-2">Archive Created</h3>
                <div className="text-sm text-gray-300 space-y-1">
                  <div>Path: {result.archive_path}</div>
                  <div>Size: {formatSize(result.total_size)}</div>
                  <div>Files: {result.files_included}</div>
                  <div>Format: {result.format}</div>
                </div>
              </div>
            )}
          </div>
        )}

        {activeTab === 'extract' && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-2">Archive Path</label>
              <input
                type="text"
                value={extractPath}
                onChange={(e) => setExtractPath(e.target.value)}
                placeholder="path/to/archive.zip"
                className="w-full px-3 py-2 bg-gray-800 text-white rounded border border-gray-700"
              />
            </div>

            <button
              onClick={handleExtract}
              disabled={loading || !extractPath.trim()}
              className="w-full px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700 disabled:opacity-50"
            >
              {loading ? 'Extracting...' : 'Extract Archive'}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default ArchiveManager;

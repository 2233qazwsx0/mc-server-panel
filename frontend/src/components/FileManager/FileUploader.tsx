import React, { useState, useRef, useCallback } from 'react';
import { X, Upload, CheckCircle, AlertCircle, File } from 'lucide-react';
import { ChunkUpload, UploadProgress, ApiResponse } from './types';

interface FileUploaderProps {
  onClose: () => void;
}

const CHUNK_SIZE = 5 * 1024 * 1024;

const FileUploader: React.FC<FileUploaderProps> = ({ onClose }) => {
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [uploads, setUploads] = useState<Map<string, UploadProgress>>(new Map());
  const [uploadPath, setUploadPath] = useState('');
  const [dragging, setDragging] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = useCallback((files: FileList | null) => {
    if (!files) return;
    setSelectedFiles(Array.from(files));
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
    handleFileSelect(e.dataTransfer.files);
  }, [handleFileSelect]);

  const handleUpload = async () => {
    if (selectedFiles.length === 0) return;

    for (const file of selectedFiles) {
      await uploadFile(file);
    }
  };

  const uploadFile = async (file: File) => {
    const totalChunks = Math.ceil(file.size / CHUNK_SIZE);
    const uploadId = `${file.name}-${Date.now()}`;

    try {
      const initResponse = await fetch('/api/files/upload/init', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          filename: file.name,
          path: uploadPath || '/',
          total_chunks: totalChunks,
          total_size: file.size,
          checksum: await calculateChecksum(file)
        })
      });

      const initData: ApiResponse<ChunkUpload> = await initResponse.json();
      if (!initData.success || !initData.data) {
        throw new Error(initData.error || 'Failed to initialize upload');
      }

      const uploadIdFromServer = initData.data.id;

      for (let i = 0; i < totalChunks; i++) {
        const start = i * CHUNK_SIZE;
        const end = Math.min(start + CHUNK_SIZE, file.size);
        const chunk = file.slice(start, end);

        const chunkData = await chunk.arrayBuffer();
        const chunkUint8 = new Uint8Array(chunkData);

        const chunkResponse = await fetch('/api/files/upload/chunk', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            upload_id: uploadIdFromServer,
            chunk_index: i,
            data: Array.from(chunkUint8),
            checksum: await calculateChunkChecksum(chunkUint8)
          })
        });

        const chunkDataResp: ApiResponse<UploadProgress> = await chunkResponse.json();
        if (chunkDataResp.success && chunkDataResp.data) {
          setUploads(prev => {
            const newMap = new Map(prev);
            newMap.set(uploadIdFromServer, chunkDataResp.data!);
            return newMap;
          });
        }
      }

      const completeResponse = await fetch(`/api/files/upload/complete/${uploadIdFromServer}`, {
        method: 'POST'
      });

      const completeData: ApiResponse<{ path: string }> = await completeResponse.json();
      if (completeData.success) {
        setUploads(prev => {
          const newMap = new Map(prev);
          newMap.delete(uploadIdFromServer);
          return newMap;
        });
      }
    } catch (err) {
      console.error('Upload error:', err);
    }
  };

  const calculateChecksum = async (file: File): Promise<string> => {
    const buffer = await file.arrayBuffer();
    const hashBuffer = await crypto.subtle.digest('SHA-256', buffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  };

  const calculateChunkChecksum = async (data: Uint8Array): Promise<string> => {
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  };

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50">
      <div className="bg-gray-900 rounded-lg shadow-xl w-full max-w-2xl max-h-[80vh] flex flex-col m-8">
        <div className="flex items-center justify-between px-4 py-3 bg-gray-800 border-b border-gray-700">
          <div className="flex items-center space-x-2">
            <Upload className="w-5 h-5 text-green-400" />
            <h2 className="text-lg font-semibold text-white">Upload Files</h2>
          </div>
          <button onClick={onClose} className="p-1 hover:bg-gray-700 rounded">
            <X className="w-5 h-5 text-gray-400" />
          </button>
        </div>

        <div className="p-4 space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Destination Path</label>
            <input
              type="text"
              value={uploadPath}
              onChange={(e) => setUploadPath(e.target.value)}
              placeholder="/plugins"
              className="w-full px-3 py-2 bg-gray-800 text-white rounded border border-gray-700"
            />
          </div>

          <div
            onDragOver={(e) => { e.preventDefault(); setDragging(true); }}
            onDragLeave={() => setDragging(false)}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
            className={`border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors ${
              dragging
                ? 'border-blue-500 bg-blue-900 bg-opacity-20'
                : 'border-gray-600 hover:border-gray-500'
            }`}
          >
            <input
              ref={fileInputRef}
              type="file"
              multiple
              onChange={(e) => handleFileSelect(e.target.files)}
              className="hidden"
            />
            <Upload className="w-12 h-12 text-gray-400 mx-auto mb-2" />
            <p className="text-gray-300">Drag and drop files here or click to select</p>
            <p className="text-gray-500 text-sm mt-1">Max chunk size: 5MB</p>
          </div>

          {selectedFiles.length > 0 && (
            <div className="bg-gray-800 rounded p-4">
              <h3 className="text-white font-medium mb-2">Selected Files ({selectedFiles.length})</h3>
              <div className="space-y-1 max-h-40 overflow-auto">
                {selectedFiles.map((file, idx) => (
                  <div key={idx} className="flex items-center justify-between text-sm">
                    <div className="flex items-center space-x-2 text-gray-300">
                      <File className="w-4 h-4" />
                      <span>{file.name}</span>
                    </div>
                    <span className="text-gray-500">{formatSize(file.size)}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {uploads.size > 0 && (
            <div className="bg-gray-800 rounded p-4">
              <h3 className="text-white font-medium mb-2">Uploading</h3>
              <div className="space-y-2">
                {Array.from(uploads.entries()).map(([id, upload]) => (
                  <div key={id}>
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span className="text-gray-300">{upload.filename}</span>
                      <span className="text-gray-500">{upload.progress_percent.toFixed(0)}%</span>
                    </div>
                    <div className="w-full bg-gray-700 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full transition-all"
                        style={{ width: `${upload.progress_percent}%` }}
                      />
                    </div>
                    <div className="text-xs text-gray-500 mt-1">
                      {upload.received_chunks}/{upload.total_chunks} chunks
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="flex justify-end space-x-2 px-4 py-3 bg-gray-800 border-t border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-gray-700 text-white rounded text-sm hover:bg-gray-600"
          >
            Close
          </button>
          <button
            onClick={handleUpload}
            disabled={selectedFiles.length === 0 || uploads.size > 0}
            className="px-4 py-2 bg-green-600 text-white rounded text-sm hover:bg-green-700 disabled:opacity-50"
          >
            Upload {selectedFiles.length > 0 && `(${selectedFiles.length})`}
          </button>
        </div>
      </div>
    </div>
  );
};

export default FileUploader;

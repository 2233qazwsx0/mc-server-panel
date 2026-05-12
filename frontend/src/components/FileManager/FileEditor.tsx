import React, { useState, useEffect, useRef } from 'react';
import { X, Save, RotateCcw, FileText, Check } from 'lucide-react';
import { ApiResponse, FileSaveResult } from './types';

interface FileEditorProps {
  filePath: string;
  onClose: () => void;
}

const FileEditor: React.FC<FileEditorProps> = ({ filePath, onClose }) => {
  const [content, setContent] = useState<string>('');
  const [originalContent, setOriginalContent] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [fileInfo, setFileInfo] = useState<{ extension: string; size: number } | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    loadFile();
  }, [filePath]);

  useEffect(() => {
    setHasChanges(content !== originalContent);
  }, [content, originalContent]);

  const loadFile = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await fetch(`/api/files/read?path=${encodeURIComponent(filePath)}`);
      const data: ApiResponse<[string, string]> = await response.json();
      
      if (data.success && data.data) {
        const [fileContent, extension] = data.data;
        setContent(fileContent);
        setOriginalContent(fileContent);
        setFileInfo({ extension, size: fileContent.length });
      } else {
        setError(data.error || 'Failed to load file');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSaved(false);
    
    try {
      const response = await fetch('/api/files/write', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          path: filePath,
          content: content
        })
      });
      
      const data: ApiResponse<FileSaveResult> = await response.json();
      
      if (data.success) {
        setOriginalContent(content);
        setSaved(true);
        setTimeout(() => setSaved(false), 2000);
      } else {
        setError(data.error || 'Failed to save file');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setSaving(false);
    }
  };

  const handleRevert = () => {
    setContent(originalContent);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      handleSave();
    }
    
    if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
      e.preventDefault();
      handleRevert();
    }
  };

  const getLanguage = (ext: string): string => {
    const languageMap: Record<string, string> = {
      'yml': 'yaml',
      'yaml': 'yaml',
      'json': 'json',
      'properties': 'properties',
      'xml': 'xml',
      'html': 'html',
      'css': 'css',
      'js': 'javascript',
      'ts': 'typescript',
      'md': 'markdown',
      'sh': 'shell',
      'bat': 'batch',
      'ps1': 'powershell',
      'toml': 'toml',
    };
    return languageMap[ext] || 'plaintext';
  };

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center bg-gray-900">
        <div className="text-gray-400">Loading file...</div>
      </div>
    );
  }

  if (error && !content) {
    return (
      <div className="h-full flex items-center justify-center bg-gray-900">
        <div className="text-red-400">{error}</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-gray-900">
      <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-3">
          <FileText className="w-5 h-5 text-blue-400" />
          <div>
            <div className="text-white font-medium">{filePath.split('/').pop()}</div>
            <div className="text-gray-400 text-xs">
              {filePath} • {fileInfo?.extension.toUpperCase()} • {content.length} characters
              {hasChanges && <span className="text-yellow-400 ml-2">(modified)</span>}
            </div>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          {saved && (
            <span className="flex items-center text-green-400 text-sm">
              <Check className="w-4 h-4 mr-1" />
              Saved
            </span>
          )}
          <button
            onClick={handleRevert}
            disabled={!hasChanges || saving}
            className="px-3 py-1.5 bg-gray-700 text-white rounded text-sm disabled:opacity-50 hover:bg-gray-600"
          >
            <RotateCcw className="w-4 h-4 inline mr-1" />
            Revert
          </button>
          <button
            onClick={handleSave}
            disabled={!hasChanges || saving}
            className="px-3 py-1.5 bg-blue-600 text-white rounded text-sm disabled:opacity-50 hover:bg-blue-700"
          >
            <Save className="w-4 h-4 inline mr-1" />
            {saving ? 'Saving...' : 'Save'}
          </button>
          <button
            onClick={onClose}
            className="px-3 py-1.5 bg-gray-700 text-white rounded text-sm hover:bg-gray-600"
          >
            <X className="w-4 h-4 inline" />
          </button>
        </div>
      </div>
      
      {error && (
        <div className="px-4 py-2 bg-red-900 text-red-200 text-sm">
          {error}
        </div>
      )}

      <div className="flex-1 overflow-hidden">
        <textarea
          ref={textareaRef}
          value={content}
          onChange={(e) => setContent(e.target.value)}
          onKeyDown={handleKeyDown}
          className="w-full h-full p-4 bg-gray-900 text-gray-100 font-mono text-sm resize-none focus:outline-none"
          spellCheck={false}
        />
      </div>
    </div>
  );
};

export default FileEditor;

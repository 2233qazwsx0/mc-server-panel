import { useState, useEffect } from 'react';
import { Folder, FileText, ArrowLeft, Upload, MoreVertical, Trash2, Edit3, Download } from 'lucide-react';
import { FileItem } from '@/types';
import { FileTableSkeleton } from '@/components/LoadingSkeleton';
import { NoFilesState } from '@/components/EmptyState';

const mockFiles: FileItem[] = [
  { name: 'world', type: 'directory', size: 1024 * 1024 * 250, modified: new Date() },
  { name: 'plugins', type: 'directory', size: 1024 * 1024 * 50, modified: new Date() },
  { name: 'server.properties', type: 'file', size: 1024 * 2, modified: new Date() },
  { name: 'bukkit.yml', type: 'file', size: 1024 * 3, modified: new Date() },
  { name: 'spigot.yml', type: 'file', size: 1024 * 4, modified: new Date() },
  { name: 'eula.txt', type: 'file', size: 1024, modified: new Date() },
  { name: 'logs', type: 'directory', size: 1024 * 1024 * 100, modified: new Date() },
];

const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

export function Files() {
  const [path, setPath] = useState(['/']);
  const [files, setFiles] = useState<FileItem[]>(mockFiles);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsLoading(false);
    }, 800);
    return () => clearTimeout(timer);
  }, []);

  const navigateTo = (folderName: string) => {
    if (folderName === '..' && path.length > 1) {
      setPath(path.slice(0, -1));
    } else if (folderName !== '..') {
      setPath([...path, folderName]);
    }
  };

  const goBack = () => {
    if (path.length > 1) {
      setPath(path.slice(0, -1));
    }
  };

  if (isLoading) {
    return (
      <div className="h-full flex flex-col">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between mb-4 gap-2">
          <div className="flex items-center gap-3">
            <Folder className="w-6 h-6 text-chart-purple" aria-hidden="true" />
            <h2 className="font-display text-xl text-text-primary">文件管理</h2>
          </div>
        </div>
        <FileTableSkeleton />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between mb-4 gap-2">
        <div className="flex items-center gap-3">
          <Folder className="w-6 h-6 text-chart-purple" aria-hidden="true" />
          <h2 className="font-display text-xl text-text-primary">文件管理</h2>
        </div>
        <button 
          className="game-button game-button-primary flex items-center gap-2 hover-lift"
          aria-label="上传文件"
        >
          <Upload className="w-4 h-4" aria-hidden="true" />
          <span className="font-mono text-sm hidden sm:inline">上传</span>
        </button>
      </div>

      {files.length === 0 ? (
        <div className="game-card flex-1">
          <NoFilesState />
        </div>
      ) : (
        <div className="game-card flex flex-col flex-1 overflow-hidden">
          <div className="border-b border-nether-600 p-4 bg-gradient-to-r from-nether-800 to-nether-900">
            <div className="flex items-center gap-2 overflow-x-auto">
              {path.length > 1 && (
                <button
                  onClick={goBack}
                  className="game-button p-2 flex-shrink-0 hover-lift"
                  aria-label="返回上一级目录"
                >
                  <ArrowLeft className="w-4 h-4" aria-hidden="true" />
                </button>
              )}
              <div className="flex items-center gap-1 text-sm font-mono flex-1 min-w-0" role="navigation" aria-label="目录路径">
                {path.map((segment, index) => (
                  <div key={index} className="flex items-center gap-1 flex-shrink-0">
                    {index > 0 && <span className="text-text-muted">/</span>}
                    <button
                      onClick={() => setPath(path.slice(0, index + 1))}
                      className="text-mc-green hover:underline truncate"
                      aria-label={`导航到 ${segment}`}
                    >
                      {segment}
                    </button>
                  </div>
                ))}
              </div>
            </div>
          </div>

          <div className="flex-1 overflow-auto">
            {/* Desktop table view */}
            <div className="hidden md:block">
              <table className="file-table" role="table">
                <thead className="file-table-header">
                  <tr>
                    <th scope="col" className="file-table-header-cell">
                      名称
                    </th>
                    <th scope="col" className="file-table-header-cell hidden lg:table-cell">
                      大小
                    </th>
                    <th scope="col" className="file-table-header-cell hidden sm:table-cell">
                      修改时间
                    </th>
                    <th scope="col" className="file-table-header-cell text-right">
                      操作
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-nether-600">
                  {files.map((file, index) => (
                    <tr key={index} className="file-table-row">
                      <td className="file-table-cell">
                        {file.type === 'directory' ? (
                          <button
                            onClick={() => navigateTo(file.name)}
                            className="flex items-center gap-3 text-left w-full hover:text-mc-green transition-colors"
                            aria-label={`打开目录 ${file.name}`}
                          >
                            <Folder className="w-5 h-5 text-chart-purple flex-shrink-0" aria-hidden="true" />
                            <span className="truncate">{file.name}</span>
                          </button>
                        ) : (
                          <div className="flex items-center gap-3">
                            <FileText className="w-5 h-5 text-mc-green flex-shrink-0" aria-hidden="true" />
                            <span className="truncate">{file.name}</span>
                          </div>
                        )}
                      </td>
                      <td className="file-table-cell hidden lg:table-cell">
                        {file.type === 'directory' ? '-' : formatSize(file.size)}
                      </td>
                      <td className="file-table-cell hidden sm:table-cell">
                        {file.modified.toLocaleDateString()} {file.modified.toLocaleTimeString()}
                      </td>
                      <td className="file-table-cell text-right">
                        <div className="flex items-center justify-end gap-1">
                          {file.type === 'file' && (
                            <>
                              <button 
                                className="game-button p-2 hover:text-mc-green hover:bg-mc-green/10"
                                aria-label={`编辑 ${file.name}`}
                              >
                                <Edit3 className="w-4 h-4" aria-hidden="true" />
                              </button>
                              <button 
                                className="game-button p-2 hover:text-rust hover:bg-rust/10"
                                aria-label={`下载 ${file.name}`}
                              >
                                <Download className="w-4 h-4" aria-hidden="true" />
                              </button>
                            </>
                          )}
                          <button 
                            className="game-button p-2 hover:text-status-error hover:bg-status-error/10"
                            aria-label={`删除 ${file.name}`}
                          >
                            <Trash2 className="w-4 h-4" aria-hidden="true" />
                          </button>
                          <button 
                            className="game-button p-2 hover:text-text-primary"
                            aria-label={`更多选项 ${file.name}`}
                          >
                            <MoreVertical className="w-4 h-4" aria-hidden="true" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {/* Mobile card view */}
            <div className="md:hidden divide-y divide-nether-600" role="list">
              {files.map((file, index) => (
                <div
                  key={index}
                  className="p-4 hover:bg-nether-700 transition-colors"
                  role="listitem"
                >
                  {file.type === 'directory' ? (
                    <button
                      onClick={() => navigateTo(file.name)}
                      className="flex items-center justify-between w-full text-left"
                      aria-label={`打开目录 ${file.name}`}
                    >
                      <div className="flex items-center gap-3">
                        <Folder className="w-6 h-6 text-chart-purple flex-shrink-0" aria-hidden="true" />
                        <span className="text-text-primary">{file.name}</span>
                      </div>
                      <ArrowLeft className="w-4 h-4 text-text-muted rotate-180 flex-shrink-0" aria-hidden="true" />
                    </button>
                  ) : (
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <FileText className="w-6 h-6 text-mc-green flex-shrink-0" aria-hidden="true" />
                        <div className="min-w-0">
                          <p className="text-text-primary truncate">{file.name}</p>
                          <p className="text-text-muted text-xs font-mono">
                            {formatSize(file.size)} • {file.modified.toLocaleDateString()}
                          </p>
                        </div>
                      </div>
                      <div className="flex items-center gap-1">
                        <button 
                          className="game-button p-2 hover:text-mc-green hover:bg-mc-green/10"
                          aria-label={`编辑 ${file.name}`}
                        >
                          <Edit3 className="w-4 h-4" aria-hidden="true" />
                        </button>
                        <button 
                          className="game-button p-2 hover:text-status-error hover:bg-status-error/10"
                          aria-label={`删除 ${file.name}`}
                        >
                          <Trash2 className="w-4 h-4" aria-hidden="true" />
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

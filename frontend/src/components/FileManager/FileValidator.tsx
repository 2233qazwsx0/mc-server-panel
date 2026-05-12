import React, { useState, useEffect } from 'react';
import { FileCheck, AlertCircle, CheckCircle, AlertTriangle, X, RefreshCw } from 'lucide-react';
import { ValidationResult, ApiResponse } from './types';

const FileValidator: React.FC = () => {
  const [filePath, setFilePath] = useState('');
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleValidate = async () => {
    if (!filePath.trim()) return;

    setLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/files/validate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: filePath })
      });

      const data: ApiResponse<ValidationResult> = await response.json();

      if (data.success && data.data) {
        setValidationResult(data.data);
      } else {
        setError(data.error || 'Failed to validate file');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const getSeverityColor = (severity: string): string => {
    switch (severity) {
      case 'error':
        return 'text-red-400 bg-red-900 bg-opacity-30';
      case 'warning':
        return 'text-yellow-400 bg-yellow-900 bg-opacity-30';
      default:
        return 'text-gray-400 bg-gray-800';
    }
  };

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-2 mb-4">
          <FileCheck className="w-5 h-5 text-blue-400" />
          <h2 className="text-lg font-semibold text-white">Configuration Validator</h2>
        </div>

        <div className="flex space-x-2">
          <input
            type="text"
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleValidate()}
            placeholder="path/to/config.yml"
            className="flex-1 px-3 py-2 bg-gray-900 text-white rounded border border-gray-700 text-sm"
          />
          <button
            onClick={handleValidate}
            disabled={loading || !filePath.trim()}
            className="px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? <RefreshCw className="w-4 h-4 animate-spin" /> : 'Validate'}
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {error && (
          <div className="bg-red-900 bg-opacity-30 border border-red-700 rounded-lg p-4 mb-4">
            <div className="flex items-center space-x-2 text-red-400">
              <AlertCircle className="w-5 h-5" />
              <span className="font-medium">Error</span>
            </div>
            <p className="text-red-300 mt-2">{error}</p>
          </div>
        )}

        {validationResult && (
          <div className="space-y-4">
            <div className={`rounded-lg p-4 ${
              validationResult.valid
                ? 'bg-green-900 bg-opacity-30 border border-green-700'
                : 'bg-red-900 bg-opacity-30 border border-red-700'
            }`}>
              <div className="flex items-center space-x-2">
                {validationResult.valid ? (
                  <>
                    <CheckCircle className="w-6 h-6 text-green-400" />
                    <span className="text-green-400 font-medium text-lg">Valid Configuration</span>
                  </>
                ) : (
                  <>
                    <AlertCircle className="w-6 h-6 text-red-400" />
                    <span className="text-red-400 font-medium text-lg">Invalid Configuration</span>
                  </>
                )}
              </div>
              <div className="mt-2 text-sm text-gray-300">
                <p>File: {validationResult.file_path}</p>
                <p>Type: {validationResult.file_type}</p>
              </div>
            </div>

            {validationResult.errors.length > 0 && (
              <div className="bg-gray-800 rounded-lg p-4">
                <h3 className="text-white font-medium mb-3 flex items-center space-x-2">
                  <AlertCircle className="w-5 h-5 text-red-400" />
                  <span>Errors ({validationResult.errors.length})</span>
                </h3>
                <div className="space-y-2">
                  {validationResult.errors.map((error, idx) => (
                    <div
                      key={idx}
                      className={`p-3 rounded ${getSeverityColor(error.severity)}`}
                    >
                      <div className="flex items-start justify-between">
                        <div>
                          <span className="font-mono text-xs">{error.code}</span>
                          <p className="mt-1">{error.message}</p>
                        </div>
                        {error.line && (
                          <span className="text-sm opacity-75">
                            Line {error.line}
                            {error.column && `:${error.column}`}
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {validationResult.warnings.length > 0 && (
              <div className="bg-gray-800 rounded-lg p-4">
                <h3 className="text-white font-medium mb-3 flex items-center space-x-2">
                  <AlertTriangle className="w-5 h-5 text-yellow-400" />
                  <span>Warnings ({validationResult.warnings.length})</span>
                </h3>
                <div className="space-y-2">
                  {validationResult.warnings.map((warning, idx) => (
                    <div
                      key={idx}
                      className="p-3 rounded bg-yellow-900 bg-opacity-30"
                    >
                      <div className="flex items-start justify-between">
                        <div>
                          <span className="font-mono text-xs">{warning.code}</span>
                          <p className="mt-1">{warning.message}</p>
                        </div>
                        {warning.line && (
                          <span className="text-sm text-yellow-300 opacity-75">
                            Line {warning.line}
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {validationResult.suggestions.length > 0 && (
              <div className="bg-gray-800 rounded-lg p-4">
                <h3 className="text-white font-medium mb-3">Suggestions</h3>
                <ul className="space-y-2">
                  {validationResult.suggestions.map((suggestion, idx) => (
                    <li key={idx} className="flex items-start space-x-2 text-gray-300">
                      <span className="text-blue-400">•</span>
                      <span>{suggestion}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {validationResult.errors.length === 0 && validationResult.warnings.length === 0 && validationResult.suggestions.length === 0 && (
              <div className="bg-gray-800 rounded-lg p-4 text-center text-gray-400">
                No issues found. Your configuration file looks good!
              </div>
            )}
          </div>
        )}

        {!validationResult && !error && (
          <div className="text-center text-gray-400 py-8">
            Enter a configuration file path to validate
          </div>
        )}
      </div>
    </div>
  );
};

export default FileValidator;

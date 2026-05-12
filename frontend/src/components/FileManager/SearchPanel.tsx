import React, { useState } from 'react';
import { Search, File, Folder, X } from 'lucide-react';
import { SearchQuery, SearchResult, ApiResponse } from './types';
import { useFileManager } from './index';

const SearchPanel: React.FC = () => {
  const { setSearchResults, setShowSearch } = useFileManager();
  const [query, setQuery] = useState('');
  const [path, setPath] = useState('');
  const [fileTypes, setFileTypes] = useState<string[]>([]);
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [useRegex, setUseRegex] = useState(false);
  const [contentSearch, setContentSearch] = useState(true);
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);

  const handleSearch = async () => {
    if (!query.trim()) return;

    setLoading(true);
    setSearched(true);

    const searchQuery: SearchQuery = {
      query: query,
      path: path || undefined,
      file_types: fileTypes.length > 0 ? fileTypes : undefined,
      case_sensitive: caseSensitive,
      regex_enabled: useRegex,
      content_search: contentSearch,
      max_results: 100
    };

    try {
      const response = await fetch('/api/files/search', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(searchQuery)
      });
      
      const data: ApiResponse<SearchResult[]> = await response.json();
      
      if (data.success && data.data) {
        setResults(data.data);
        setSearchResults(data.data);
      }
    } catch (err) {
      console.error('Search error:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleFileTypeToggle = (type: string) => {
    setFileTypes(prev => 
      prev.includes(type) 
        ? prev.filter(t => t !== type)
        : [...prev, type]
    );
  };

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 bg-gray-800 border-b border-gray-700">
        <div className="space-y-3">
          <div className="flex items-center space-x-2">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                placeholder="Search files and content..."
                className="w-full pl-10 pr-4 py-2 bg-gray-900 text-white rounded border border-gray-700 focus:border-blue-500 focus:outline-none"
              />
            </div>
            <button
              onClick={handleSearch}
              disabled={loading || !query.trim()}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
            >
              {loading ? 'Searching...' : 'Search'}
            </button>
          </div>

          <div className="flex items-center space-x-4">
            <input
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              placeholder="Path (optional)"
              className="flex-1 px-3 py-1.5 bg-gray-900 text-white rounded border border-gray-700 text-sm"
            />
            
            <label className="flex items-center space-x-2 text-sm text-gray-300">
              <input
                type="checkbox"
                checked={caseSensitive}
                onChange={(e) => setCaseSensitive(e.target.checked)}
                className="rounded"
              />
              <span>Case Sensitive</span>
            </label>
            
            <label className="flex items-center space-x-2 text-sm text-gray-300">
              <input
                type="checkbox"
                checked={useRegex}
                onChange={(e) => setUseRegex(e.target.checked)}
                className="rounded"
              />
              <span>Regex</span>
            </label>
            
            <label className="flex items-center space-x-2 text-sm text-gray-300">
              <input
                type="checkbox"
                checked={contentSearch}
                onChange={(e) => setContentSearch(e.target.checked)}
                className="rounded"
              />
              <span>Content</span>
            </label>
          </div>

          <div className="flex items-center space-x-2">
            <span className="text-sm text-gray-400">Types:</span>
            {['yml', 'yaml', 'json', 'properties', 'txt', 'log'].map(type => (
              <button
                key={type}
                onClick={() => handleFileTypeToggle(type)}
                className={`px-2 py-1 text-xs rounded ${
                  fileTypes.includes(type)
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-700 text-gray-300'
                }`}
              >
                .{type}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {!searched && (
          <div className="text-center text-gray-400 py-8">
            Enter a search query to find files
          </div>
        )}

        {searched && results.length === 0 && !loading && (
          <div className="text-center text-gray-400 py-8">
            No results found
          </div>
        )}

        {results.length > 0 && (
          <div className="space-y-2">
            <div className="text-sm text-gray-400 mb-4">
              Found {results.length} results
            </div>
            
            {results.map((result, idx) => (
              <div
                key={idx}
                className="p-3 bg-gray-800 rounded-lg hover:bg-gray-750 cursor-pointer"
              >
                <div className="flex items-center space-x-2 mb-2">
                  {result.file.extension ? (
                    <File className="w-4 h-4 text-blue-400" />
                  ) : (
                    <Folder className="w-4 h-4 text-yellow-400" />
                  )}
                  <span className="text-white font-medium">{result.file.name}</span>
                  <span className="text-gray-400 text-sm">({result.file.path})</span>
                </div>
                
                {result.matches.length > 0 && (
                  <div className="ml-6 space-y-1">
                    {result.matches.slice(0, 3).map((match, mIdx) => (
                      <div key={mIdx} className="text-sm">
                        <span className="text-gray-500">Line {match.line_number}: </span>
                        <span className="text-gray-300 font-mono">
                          {match.line_content.substring(0, 100)}
                          {match.line_content.length > 100 && '...'}
                        </span>
                      </div>
                    ))}
                    {result.matches.length > 3 && (
                      <div className="text-gray-500 text-sm">
                        +{result.matches.length - 3} more matches
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default SearchPanel;

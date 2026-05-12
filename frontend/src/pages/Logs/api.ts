import type {
  LogEntry,
  SearchResult,
  LogStatistics,
  DashboardSummary,
  DiagnosticResult,
  KnowledgeBaseEntry,
  CustomParseRule,
  AnalysisResult,
  BottleneckReport,
  TrendAnalysis,
  ExportOptions,
  ApiResponse,
} from './types';

const API_BASE = '/api/logs';

async function fetchApi<T>(
  url: string,
  options?: RequestInit
): Promise<T> {
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API Error: ${response.statusText}`);
  }

  const data: ApiResponse<T> = await response.json();
  
  if (!data.success) {
    throw new Error(data.error || 'Unknown error');
  }

  return data.data as T;
}

export async function searchLogs(params: {
  query?: string;
  page?: number;
  per_page?: number;
  levels?: string;
  sources?: string;
  start_time?: string;
  end_time?: string;
  keywords?: string;
  exclude?: string;
  stack_trace_only?: boolean;
  performance_only?: boolean;
  sort_field?: string;
  sort_direction?: string;
}): Promise<SearchResult> {
  const searchParams = new URLSearchParams();
  
  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== '') {
      searchParams.append(key, String(value));
    }
  });

  return fetchApi<SearchResult>(`${API_BASE}/search?${searchParams}`);
}

export async function getLogEntry(id: string): Promise<LogEntry> {
  return fetchApi<LogEntry>(`${API_BASE}/${id}`);
}

export async function analyzeLog(id: string): Promise<AnalysisResult> {
  return fetchApi<AnalysisResult>(`${API_BASE}/${id}/analyze`);
}

export async function parseStackTrace(id: string): Promise<{ 
  error_type: string;
  error_message: string;
  frames: Array<{
    file: string;
    line: number;
    method: string;
    class?: string;
  }>;
  likely_cause: string;
  plugin_culprit?: string;
}> {
  return fetchApi(`${API_BASE}/${id}/stacktrace`);
}

export async function getStatistics(): Promise<LogStatistics> {
  return fetchApi<LogStatistics>(`${API_BASE}/statistics`);
}

export async function getDashboard(): Promise<DashboardSummary> {
  return fetchApi<DashboardSummary>(`${API_BASE}/dashboard`);
}

export async function analyzeTrend(hours: number = 24): Promise<TrendAnalysis> {
  return fetchApi<TrendAnalysis>(`${API_BASE}/trend?hours=${hours}`);
}

export async function detectBottlenecks(): Promise<BottleneckReport[]> {
  return fetchApi<BottleneckReport[]>(`${API_BASE}/analyze`);
}

export async function exportLogs(options: ExportOptions): Promise<string> {
  const response = await fetch(`${API_BASE}/export`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(options),
  });

  if (!response.ok) {
    throw new Error(`Export failed: ${response.statusText}`);
  }

  const data: ApiResponse<{ content: string }> = await response.json();
  
  if (!data.success) {
    throw new Error(data.error || 'Export failed');
  }

  return data.data?.content || '';
}

export async function getKnowledgeBase(): Promise<KnowledgeBaseEntry[]> {
  return fetchApi<KnowledgeBaseEntry[]>(`${API_BASE}/knowledge`);
}

export async function searchKnowledge(query: string): Promise<KnowledgeBaseEntry[]> {
  return fetchApi<KnowledgeBaseEntry[]>(
    `${API_BASE}/knowledge/search?query=${encodeURIComponent(query)}`
  );
}

export async function lookupKnowledge(errorMessage: string): Promise<KnowledgeBaseEntry | null> {
  const response = await fetch(`${API_BASE}/knowledge/lookup`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ error_message: errorMessage }),
  });

  const data: ApiResponse<KnowledgeBaseEntry | null> = await response.json();
  return data.data || null;
}

export async function getDiagnostic(): Promise<DiagnosticResult> {
  return fetchApi<DiagnosticResult>(`${API_BASE}/diagnostic`);
}

export async function getCustomRules(): Promise<CustomParseRule[]> {
  return fetchApi<CustomParseRule[]>(`${API_BASE}/rules`);
}

export async function addCustomRule(rule: CustomParseRule): Promise<void> {
  await fetchApi<void>(`${API_BASE}/rules`, {
    method: 'POST',
    body: JSON.stringify(rule),
  });
}

export async function deleteCustomRule(id: string): Promise<void> {
  await fetchApi<void>(`${API_BASE}/rules/${id}`, {
    method: 'DELETE',
  });
}

export async function toggleCustomRule(id: string, enabled: boolean): Promise<void> {
  await fetchApi<void>(`${API_BASE}/rules/${id}/${enabled}`, {
    method: 'PUT',
  });
}

export async function clearLogs(): Promise<void> {
  await fetchApi<void>(`${API_BASE}/clear`, {
    method: 'POST',
  });
}

export async function getLogCount(): Promise<number> {
  const response = await fetchApi<{ count: number }>(`${API_BASE}/count`);
  return response.count;
}

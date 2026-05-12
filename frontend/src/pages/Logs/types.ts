export interface LogEntry {
  id: string;
  timestamp: string;
  level: LogLevel;
  source: string;
  message: string;
  raw: string;
  highlights: HighlightRange[];
  metadata: LogMetadata;
}

export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error' | 'fatal';

export interface HighlightRange {
  start: number;
  end: number;
  highlight_type: HighlightType;
}

export type HighlightType = 
  | 'error' 
  | 'warning' 
  | 'info' 
  | 'keyword' 
  | 'stack_trace' 
  | 'performance' 
  | 'entity' 
  | 'player' 
  | 'command';

export interface LogMetadata {
  player_name?: string;
  entity_id?: number;
  world_name?: string;
  dimension?: string;
  position?: Position;
  stack_trace?: StackFrame[];
  performance_data?: PerformanceData;
}

export interface Position {
  x: number;
  y: number;
  z: number;
}

export interface StackFrame {
  file: string;
  line: number;
  method: string;
  class?: string;
}

export interface PerformanceData {
  tps: number;
  mspt: number;
  cpu_usage: number;
  memory_usage: number;
  entity_count: number;
  tick_time: number;
}

export interface SearchQuery {
  query: string;
  filters: SearchFilters;
  pagination: Pagination;
  sort: SortOptions;
}

export interface SearchFilters {
  levels?: LogLevel[];
  sources?: string[];
  time_range?: TimeRange;
  keywords?: string[];
  exclude_keywords?: string[];
  player_names?: string[];
  stack_trace_only: boolean;
  performance_issues_only: boolean;
}

export interface TimeRange {
  start: string;
  end: string;
}

export interface Pagination {
  page: number;
  per_page: number;
}

export interface SortOptions {
  field: 'timestamp' | 'level' | 'source' | 'relevance';
  direction: 'asc' | 'desc';
}

export interface SearchResult {
  entries: LogEntry[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
  query_time_ms: number;
  highlights: SearchHighlight[];
}

export interface SearchHighlight {
  log_id: string;
  matches: HighlightMatch[];
}

export interface HighlightMatch {
  field: string;
  start: number;
  end: number;
  context: string;
}

export interface LogStatistics {
  total_entries: number;
  by_level: LevelCounts;
  by_source: SourceCounts;
  time_series: TimeSeriesPoint[];
  error_rate: ErrorRateStats;
  performance_trends: PerformanceTrendStats;
  top_errors: ErrorSummary[];
  top_players: PlayerActivity[];
}

export interface LevelCounts {
  trace: number;
  debug: number;
  info: number;
  warn: number;
  error: number;
  fatal: number;
}

export interface SourceCounts {
  counts: SourceCount[];
}

export interface SourceCount {
  source: string;
  count: number;
}

export interface TimeSeriesPoint {
  timestamp: string;
  count: number;
  error_count: number;
  warn_count: number;
}

export interface ErrorRateStats {
  current_rate: number;
  average_rate: number;
  peak_rate: number;
  peak_time?: string;
  trend: 'up' | 'down' | 'stable';
}

export interface PerformanceTrendStats {
  average_tps: number;
  average_mspt: number;
  tps_trend: 'up' | 'down' | 'stable';
  memory_trend: 'up' | 'down' | 'stable';
  predicted_tps_24h?: number;
  predicted_mspt_24h?: number;
}

export interface ErrorSummary {
  error_type: string;
  count: number;
  first_occurrence: string;
  last_occurrence: string;
  description?: string;
  solution?: string;
}

export interface PlayerActivity {
  player_name: string;
  action_count: number;
  first_seen: string;
  last_seen: string;
}

export interface ExportOptions {
  format: 'json' | 'csv' | 'text' | 'html';
  filters: SearchFilters;
  include_metadata: boolean;
  include_context: boolean;
  max_entries?: number;
}

export interface CustomParseRule {
  id: string;
  name: string;
  pattern: string;
  fields: ParseField[];
  enabled: boolean;
  priority: number;
}

export interface ParseField {
  name: string;
  field_type: 'string' | 'integer' | 'float' | 'timestamp' | 'level' | 'regex';
  capture_group: number;
}

export interface KnowledgeBaseEntry {
  id: string;
  error_pattern: string;
  error_type: string;
  title: string;
  description: string;
  cause: string;
  solution: string;
  prevention: string[];
  related_errors: string[];
  severity: number;
  tags: string[];
}

export interface DiagnosticResult {
  issues: DiagnosticIssue[];
  recommendations: string[];
  overall_health: HealthStatus;
}

export interface DiagnosticIssue {
  id: string;
  title: string;
  description: string;
  severity: 'info' | 'warning' | 'error' | 'critical';
  affected_logs: number;
  first_occurrence: string;
  last_occurrence: string;
  related_rule?: string;
}

export interface HealthStatus {
  score: number;
  status: 'healthy' | 'degraded' | 'unhealthy' | 'critical';
  summary: string;
}

export interface AnalysisResult {
  entry_id: string;
  anomaly_score: number;
  issues: DetectedIssue[];
  performance_impact?: PerformanceImpact;
  recommendations: string[];
}

export interface DetectedIssue {
  issue_type: IssueType;
  description: string;
  severity: 'info' | 'warning' | 'error' | 'critical';
}

export type IssueType = 
  | 'memory' 
  | 'network' 
  | 'entity' 
  | 'performance' 
  | 'exception' 
  | 'world' 
  | 'security' 
  | 'configuration';

export interface PerformanceImpact {
  tps_degradation: number;
  cpu_usage: number;
  memory_pressure: number;
  entity_count: number;
  severity: 'low' | 'medium' | 'high';
}

export interface BottleneckReport {
  bottleneck_type: BottleneckType;
  description: string;
  severity: 'info' | 'warning' | 'error' | 'critical';
  affected_time_range?: [string, string];
  possible_causes: string[];
  suggested_fixes: string[];
}

export type BottleneckType = 
  | 'tick_performance' 
  | 'tick_processing' 
  | 'memory' 
  | 'entity_density' 
  | 'disk_io' 
  | 'network_io';

export interface TrendAnalysis {
  error_count: number;
  warn_count: number;
  error_rate: number;
  average_tps: number;
  average_mspt: number;
  predicted_tps_24h?: number;
  predicted_mspt_24h?: number;
  trend_direction: 'up' | 'down' | 'stable';
}

export interface DashboardSummary {
  total_logs: number;
  error_count: number;
  warn_count: number;
  info_count: number;
  debug_count: number;
  error_rate: number;
  average_tps: number;
  average_mspt: number;
  top_error?: ErrorSummary;
  uptime_hours: number;
  log_sources: string[];
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

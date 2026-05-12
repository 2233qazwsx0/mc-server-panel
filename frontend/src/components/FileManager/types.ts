export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: string;
  permissions: string;
}

export interface SearchQuery {
  query: string;
  path?: string;
  file_types?: string[];
  case_sensitive: boolean;
  regex_enabled: boolean;
  content_search: boolean;
  max_results: number;
}

export interface SearchMatch {
  line_number: number;
  line_content: string;
  match_start: number;
  match_end: number;
  context_before: string[];
  context_after: string[];
}

export interface SearchResult {
  file: {
    path: string;
    name: string;
    extension?: string;
    size: number;
    modified: string;
    directory: string;
  };
  matches: SearchMatch[];
  score: number;
}

export interface DiffResult {
  original_file: string;
  modified_file: string;
  hunks: DiffHunk[];
  stats: DiffStats;
}

export interface DiffHunk {
  original_start: number;
  original_count: number;
  modified_start: number;
  modified_count: number;
  changes: DiffChange[];
}

export interface DiffChange {
  change_type: string;
  old_line?: number;
  new_line?: number;
  content: string;
}

export interface DiffStats {
  additions: number;
  deletions: number;
  unchanged: number;
}

export interface DiffLine {
  index: number;
  line_number?: number;
  change_type: string;
  content: string;
}

export interface InlineDiffResult {
  original_file: string;
  modified_file: string;
  lines: DiffLine[];
}

export interface ValidationResult {
  valid: boolean;
  file_path: string;
  file_type: string;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  suggestions: string[];
}

export interface ValidationError {
  line?: number;
  column?: number;
  message: string;
  code: string;
  severity: string;
}

export interface ValidationWarning {
  line?: number;
  column?: number;
  message: string;
  code: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface FileSaveResult {
  path: string;
  checksum_before?: string;
  checksum_after: string;
  backup_path?: string;
  bytes_written: number;
}

export interface FileInfo {
  path: string;
  name: string;
  is_directory: boolean;
  size: number;
  created?: string;
  modified?: string;
  extension?: string;
  line_count?: number;
  content_hash?: string;
}

export interface ArchiveRequest {
  files: string[];
  output_name: string;
  format: 'Zip' | 'Tar' | 'TarGz';
}

export interface ArchiveResult {
  archive_path: string;
  total_size: number;
  files_included: number;
  format: string;
}

export interface ArchiveEntry {
  name: string;
  is_dir: boolean;
  size: number;
  compressed_size: number;
}

export interface ExtractResult {
  destination: string;
  files_extracted: number;
}

export interface AclRule {
  id: string;
  path: string;
  principal: AclPrincipal;
  permissions: string[];
  recursive: boolean;
  created_at: string;
  modified_at: string;
}

export interface AclPrincipal {
  kind: string;
  id: string;
  name: string;
}

export interface SyncConfig {
  id: string;
  name: string;
  source: SyncEndpoint;
  target: SyncEndpoint;
  direction: string;
  auto_sync: boolean;
  sync_interval_seconds: number;
  last_sync?: string;
  status: string;
}

export interface SyncEndpoint {
  protocol: string;
  host: string;
  port: number;
  base_path: string;
  credentials?: {
    username: string;
    password?: string;
    key_path?: string;
  };
}

export interface SyncPreview {
  to_upload: SyncFileChange[];
  to_download: SyncFileChange[];
  to_delete: string[];
  conflicts: SyncConflict[];
}

export interface SyncFileChange {
  path: string;
  change_type: string;
  size: number;
  checksum?: string;
}

export interface SyncConflict {
  path: string;
  source_modified: string;
  target_modified: string;
  source_size: number;
  target_size: number;
}

export interface SyncEvent {
  id: string;
  config_id: string;
  timestamp: string;
  event_type: string;
  files_affected: SyncFileChange[];
  duration_ms: number;
  success: boolean;
  error_message?: string;
}

export interface ChunkUpload {
  id: string;
  filename: string;
  destination: string;
  total_chunks: number;
  received_chunks: number[];
  total_size: number;
  checksum: string;
  created_at: string;
  last_activity: string;
}

export interface UploadProgress {
  upload_id: string;
  filename: string;
  total_chunks: number;
  received_chunks: number;
  received_bytes: number;
  total_bytes: number;
  progress_percent: number;
  is_complete: boolean;
  chunks: ChunkInfo[];
}

export interface ChunkInfo {
  index: number;
  size: number;
  checksum: string;
  received: boolean;
}

export interface TrashItem {
  id: string;
  original_path: string;
  original_name: string;
  trash_path: string;
  deleted_at: string;
  deleted_by: string;
  size: number;
  file_type: string;
  expires_at: string;
}

export interface TrashList {
  items: TrashItem[];
  total_size: number;
  item_count: number;
  oldest_item?: string;
  newest_item?: string;
}

export interface RestoreResult {
  success: boolean;
  restored_path: string;
  item_id: string;
  warnings: string[];
}

export interface GitRepository {
  id: string;
  name: string;
  path: string;
  remote_url?: string;
  branch: string;
  initialized: boolean;
}

export interface GitCommit {
  hash: string;
  short_hash: string;
  author: string;
  email: string;
  date: string;
  message: string;
}

export interface GitStatus {
  branch: string;
  is_clean: boolean;
  staged: GitFileStatus[];
  modified: GitFileStatus[];
  untracked: string[];
  conflicted: string[];
}

export interface GitFileStatus {
  path: string;
  status: string;
}

export interface GitDiff {
  file: string;
  hunks: GitDiffHunk[];
}

export interface GitDiffHunk {
  old_start: number;
  old_lines: number;
  new_start: number;
  new_lines: number;
  lines: GitDiffLine[];
}

export interface GitDiffLine {
  line_type: string;
  content: string;
}

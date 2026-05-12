use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use walkdir::WalkDir;

pub struct SearchService {
    server_root: PathBuf,
    index: RwLock<FileIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndex {
    pub files: HashMap<String, IndexedFile>,
    pub last_updated: String,
    pub total_files: usize,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: String,
    pub content_hash: String,
    pub line_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub path: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub case_sensitive: bool,
    pub regex_enabled: bool,
    pub content_search: bool,
    pub max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file: SearchResultFile,
    pub matches: Vec<SearchMatch>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultFile {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: String,
    pub directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

impl SearchService {
    pub fn new(server_root: PathBuf) -> Self {
        Self {
            server_root,
            index: RwLock::new(FileIndex {
                files: HashMap::new(),
                last_updated: chrono::Utc::now().to_rfc3339(),
                total_files: 0,
                total_size: 0,
            }),
        }
    }

    pub fn rebuild_index(&self) -> Result<FileIndex> {
        let mut files = HashMap::new();
        let mut total_size = 0u64;

        for entry in WalkDir::new(&self.server_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let relative = path.strip_prefix(&self.server_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if relative.starts_with('.') || relative.contains("/.git/") {
                continue;
            }

            let metadata = fs::metadata(path)?;
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let extension = path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());
            
            let content_hash = self.calculate_hash(path)?;

            let indexed = IndexedFile {
                path: relative.clone(),
                name,
                extension,
                size: metadata.len(),
                modified: metadata.modified()
                    .ok()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                    .unwrap_or_default(),
                content_hash,
                line_count: None,
            };

            total_size += metadata.len();
            files.insert(relative, indexed);
        }

        let total_files = files.len();
        let index = FileIndex {
            files,
            last_updated: chrono::Utc::now().to_rfc3339(),
            total_files,
            total_size,
        };

        {
            let mut guard = self.index.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            *guard = index.clone();
        }

        Ok(index)
    }

    pub fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let index = self.index.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let search_pattern = if query.regex_enabled {
            if query.case_sensitive {
                Regex::new(&query.query)
            } else {
                Regex::new(&format!("(?i){}", query.query))
            }
        } else {
            if query.case_sensitive {
                Regex::new(&regex::escape(&query.query))
            } else {
                Regex::new(&format!("(?i){}", regex::escape(&query.query)))
            }
        }.map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;

        let mut results = Vec::new();

        for (path, file) in &index.files {
            if let Some(ref filter_path) = query.path {
                if !path.starts_with(filter_path) {
                    continue;
                }
            }

            if let Some(ref types) = query.file_types {
                if let Some(ref ext) = file.extension {
                    if !types.iter().any(|t| t.eq_ignore_ascii_case(ext)) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            let mut file_matches = Vec::new();
            let mut name_score = 0.0;

            if search_pattern.is_match(&file.name) {
                name_score = self.calculate_name_score(&file.name, &query.query);
            }

            if query.content_search && file.size < 10 * 1024 * 1024 {
                if let Ok(content) = fs::read_to_string(&self.server_root.join(path)) {
                    for (line_num, line) in content.lines().enumerate() {
                        if search_pattern.is_match(line) {
                            let context: Vec<String> = content
                                .lines()
                                .skip(line_num.saturating_sub(2))
                                .take(5)
                                .map(|s| s.to_string())
                                .collect();

                            let context_before: Vec<String> = content
                                .lines()
                                .skip(line_num.saturating_sub(2))
                                .take(2)
                                .map(|s| s.to_string())
                                .collect();

                            let context_after: Vec<String> = content
                                .lines()
                                .skip(line_num + 1)
                                .take(2)
                                .map(|s| s.to_string())
                                .collect();

                            if let Some(m) = search_pattern.find(line) {
                                file_matches.push(SearchMatch {
                                    line_number: line_num + 1,
                                    line_content: line.to_string(),
                                    match_start: m.start(),
                                    match_end: m.end(),
                                    context_before,
                                    context_after,
                                });
                            }
                        }
                    }
                }
            }

            if name_score > 0.0 || !file_matches.is_empty() {
                let content_score = file_matches.len() as f32 * 0.5;
                let score = name_score + content_score;

                if score >= 0.1 {
                    let directory = PathBuf::from(path)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    results.push(SearchResult {
                        file: SearchResultFile {
                            path: path.clone(),
                            name: file.name.clone(),
                            extension: file.extension.clone(),
                            size: file.size,
                            modified: file.modified.clone(),
                            directory,
                        },
                        matches: file_matches,
                        score,
                    });
                }
            }

            if results.len() >= query.max_results {
                break;
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    pub fn search_by_name(&self, name: &str, path: Option<&str>) -> Result<Vec<SearchResultFile>> {
        let query = SearchQuery {
            query: name.to_string(),
            path: path.map(|s| s.to_string()),
            file_types: None,
            case_sensitive: false,
            regex_enabled: false,
            content_search: false,
            max_results: 100,
        };

        let results = self.search(query)?;
        Ok(results.into_iter().map(|r| r.file).collect())
    }

    pub fn search_by_extension(&self, extension: &str, path: Option<&str>) -> Result<Vec<SearchResultFile>> {
        let query = SearchQuery {
            query: format!(".{}", extension),
            path: path.map(|s| s.to_string()),
            file_types: Some(vec![extension.to_string()]),
            case_sensitive: false,
            regex_enabled: false,
            content_search: false,
            max_results: 500,
        };

        let results = self.search(query)?;
        Ok(results.into_iter().map(|r| r.file).collect())
    }

    pub fn get_index_stats(&self) -> Result<IndexStats> {
        let index = self.index.read().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        
        let mut by_type: HashMap<String, FileTypeStats> = HashMap::new();
        
        for file in index.files.values() {
            let key = file.extension.clone().unwrap_or_else(|| "no_extension".to_string());
            let entry = by_type.entry(key).or_insert_with(|| FileTypeStats {
                extension: file.extension.clone(),
                count: 0,
                total_size: 0,
            });
            entry.count += 1;
            entry.total_size += file.size;
        }

        Ok(IndexStats {
            total_files: index.total_files,
            total_size: index.total_size,
            last_updated: index.last_updated.clone(),
            by_type,
        })
    }

    fn calculate_name_score(&self, name: &str, query: &str) -> f32 {
        let name_lower = name.to_lowercase();
        let query_lower = query.to_lowercase();

        if name_lower == query_lower {
            10.0
        } else if name_lower.starts_with(&query_lower) {
            8.0
        } else if name_lower.contains(&query_lower) {
            5.0
        } else {
            0.0
        }
    }

    fn calculate_hash(&self, path: &PathBuf) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(hex::encode(hasher.finalize())[..16].to_string())
    }

    pub fn update_file(&self, path: &str) -> Result<()> {
        let full_path = self.server_root.join(path);
        
        if !full_path.exists() {
            let mut guard = self.index.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.files.remove(path);
            guard.total_files = guard.files.len();
            guard.last_updated = chrono::Utc::now().to_rfc3339();
            return Ok(());
        }

        let metadata = fs::metadata(&full_path)?;
        let name = full_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let extension = full_path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        let content_hash = self.calculate_hash(&full_path)?;

        let indexed = IndexedFile {
            path: path.to_string(),
            name,
            extension,
            size: metadata.len(),
            modified: metadata.modified()
                .ok()
                .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                .unwrap_or_default(),
            content_hash,
            line_count: None,
        };

        let mut guard = self.index.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
        guard.files.insert(path.to_string(), indexed);
        guard.total_files = guard.files.len();
        guard.last_updated = chrono::Utc::now().to_rfc3339();

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_size: u64,
    pub last_updated: String,
    pub by_type: HashMap<String, FileTypeStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeStats {
    pub extension: Option<String>,
    pub count: usize,
    pub total_size: u64,
}

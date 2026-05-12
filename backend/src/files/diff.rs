use anyhow::Result;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

pub struct DiffService {
    server_root: PathBuf,
}

impl DiffService {
    pub fn new(server_root: PathBuf) -> Self {
        Self { server_root }
    }

    pub fn compare_files(&self, path1: &str, path2: &str) -> Result<DiffResult> {
        let full_path1 = self.resolve_path(path1)?;
        let full_path2 = self.resolve_path(path2)?;

        let content1 = std::fs::read_to_string(&full_path1)?;
        let content2 = std::fs::read_to_string(&full_path2)?;

        self.compare_content(&content1, &content2, path1, path2)
    }

    pub fn compare_content(&self, original: &str, modified: &str, path1: &str, path2: &str) -> Result<DiffResult> {
        let diff = TextDiff::from_lines(original, modified);
        
        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut original_line = 0;
        let mut modified_line = 0;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    original_line += 1;
                    modified_line += 1;
                    
                    if let Some(ref mut hunk) = current_hunk {
                        hunk.changes.push(DiffChange {
                            change_type: "equal".to_string(),
                            old_line: Some(original_line),
                            new_line: Some(modified_line),
                            content: change.value().to_string(),
                        });
                        hunk.original_count += 1;
                        hunk.modified_count += 1;
                    }
                }
                ChangeTag::Delete => {
                    original_line += 1;
                    
                    if current_hunk.is_none() {
                        current_hunk = Some(DiffHunk {
                            original_start: original_line,
                            original_count: 0,
                            modified_start: modified_line,
                            modified_count: 0,
                            changes: Vec::new(),
                        });
                    }
                    
                    if let Some(ref mut hunk) = current_hunk {
                        hunk.changes.push(DiffChange {
                            change_type: "delete".to_string(),
                            old_line: Some(original_line),
                            new_line: None,
                            content: change.value().to_string(),
                        });
                        hunk.original_count += 1;
                    }
                }
                ChangeTag::Insert => {
                    modified_line += 1;
                    
                    if current_hunk.is_none() {
                        current_hunk = Some(DiffHunk {
                            original_start: original_line.saturating_sub(3),
                            original_count: 0,
                            modified_start: modified_line,
                            modified_count: 0,
                            changes: Vec::new(),
                        });
                    }
                    
                    if let Some(ref mut hunk) = current_hunk {
                        hunk.changes.push(DiffChange {
                            change_type: "insert".to_string(),
                            old_line: None,
                            new_line: Some(modified_line),
                            content: change.value().to_string(),
                        });
                        hunk.modified_count += 1;
                    }
                }
            }

            if let Some(ref hunk) = current_hunk {
                if hunk.changes.len() > 100 {
                    hunks.push(current_hunk.take().unwrap());
                }
            }
        }

        if let Some(hunk) = current_hunk {
            hunks.push(hunk);
        }

        let stats = DiffStats {
            additions: diff.iter_all_changes()
                .filter(|c| c.tag() == ChangeTag::Insert)
                .count(),
            deletions: diff.iter_all_changes()
                .filter(|c| c.tag() == ChangeTag::Delete)
                .count(),
            unchanged: diff.iter_all_changes()
                .filter(|c| c.tag() == ChangeTag::Equal)
                .count(),
        };

        Ok(DiffResult {
            original_file: path1.to_string(),
            modified_file: path2.to_string(),
            hunks,
            stats,
        })
    }

    pub fn compare_with_backup(&self, file_path: &str) -> Result<DiffResult> {
        let backups_dir = self.server_root.join(".backups");
        
        let backup_file = std::fs::read_dir(backups_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy();
                let filename = PathBuf::from(file_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                name.contains(&filename)
            })
            .max_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()))
            .ok_or_else(|| anyhow::anyhow!("No backup found for {}", file_path))?;

        self.compare_files(&backup_file.path().to_string_lossy(), file_path)
    }

    pub fn get_inline_diff(&self, path1: &str, path2: &str) -> Result<InlineDiffResult> {
        let full_path1 = self.resolve_path(path1)?;
        let full_path2 = self.resolve_path(path2)?;

        let content1 = std::fs::read_to_string(&full_path1)?;
        let content2 = std::fs::read_to_string(&full_path2)?;

        let diff = TextDiff::from_lines(&content1, &content2);
        let mut lines = Vec::new();

        for (idx, change) in diff.iter_all_changes().enumerate() {
            let line_number = match change.tag() {
                ChangeTag::Equal => {
                    let count = lines.iter().filter(|l: &&DiffLine| l.change_type == "equal").count();
                    Some(count + 1)
                }
                ChangeTag::Delete => {
                    let count = lines.iter().filter(|l| l.change_type == "delete").count();
                    Some(count + 1)
                }
                ChangeTag::Insert => None,
            };

            lines.push(DiffLine {
                index: idx,
                line_number,
                change_type: match change.tag() {
                    ChangeTag::Equal => "equal",
                    ChangeTag::Delete => "delete",
                    ChangeTag::Insert => "insert",
                }.to_string(),
                content: change.value().to_string().trim_end_matches('\n').to_string(),
            });
        }

        Ok(InlineDiffResult {
            original_file: path1.to_string(),
            modified_file: path2.to_string(),
            lines,
        })
    }

    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let requested_path = self.server_root.join(path);
        let canonical = requested_path.canonicalize()?;
        
        if !canonical.starts_with(&self.server_root) {
            anyhow::bail!("Path escape detected: {}", path);
        }

        Ok(canonical)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffResult {
    pub original_file: String,
    pub modified_file: String,
    pub hunks: Vec<DiffHunk>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffHunk {
    pub original_start: usize,
    pub original_count: usize,
    pub modified_start: usize,
    pub modified_count: usize,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffChange {
    pub change_type: String,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InlineDiffResult {
    pub original_file: String,
    pub modified_file: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffLine {
    pub index: usize,
    pub line_number: Option<usize>,
    pub change_type: String,
    pub content: String,
}

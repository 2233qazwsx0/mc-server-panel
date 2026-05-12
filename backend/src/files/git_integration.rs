use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::RwLock;
use uuid::Uuid;

pub struct GitService {
    server_root: PathBuf,
    repos: RwLock<HashMap<String, GitRepository>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepository {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote_url: Option<String>,
    pub branch: String,
    pub initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub is_clean: bool,
    pub staged: Vec<GitFileStatus>,
    pub modified: Vec<GitFileStatus>,
    pub untracked: Vec<String>,
    pub conflicted: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileStatus {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    pub file: String,
    pub hunks: Vec<GitDiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<GitDiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiffLine {
    pub line_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBlame {
    pub file: String,
    pub lines: Vec<GitBlameLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBlameLine {
    pub line_number: usize,
    pub commit: String,
    pub author: String,
    pub date: String,
    pub content: String,
}

impl GitService {
    pub fn new(server_root: PathBuf) -> Self {
        Self {
            server_root,
            repos: RwLock::new(HashMap::new()),
        }
    }

    pub fn init_repo(&self, path: &str, name: &str) -> Result<GitRepository> {
        let full_path = self.server_root.join(path);
        
        if !full_path.exists() {
            anyhow::bail!("Directory does not exist: {}", path);
        }

        let git_dir = full_path.join(".git");
        if git_dir.exists() {
            anyhow::bail!("Git repository already exists at: {}", path);
        }

        let output = Command::new("git")
            .args(["init"])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git init failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let repo = GitRepository {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            path: path.to_string(),
            remote_url: None,
            branch: "main".to_string(),
            initialized: true,
        };

        {
            let mut guard = self.repos.write().map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
            guard.insert(repo.id.clone(), repo.clone());
        }

        Ok(repo)
    }

    pub fn get_status(&self, path: &str) -> Result<GitStatus> {
        let full_path = self.server_root.join(path);
        
        if !full_path.join(".git").exists() {
            anyhow::bail!("Not a git repository: {}", path);
        }

        let branch = self.get_current_branch(path)?;
        
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git status failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let status_output = String::from_utf8_lossy(&output.stdout);
        let mut staged = Vec::new();
        let mut modified = Vec::new();
        let mut untracked = Vec::new();
        let mut conflicted = Vec::new();

        for line in status_output.lines() {
            if line.len() < 3 {
                continue;
            }
            
            let index_status = line.chars().next().unwrap();
            let worktree_status = line.chars().nth(1).unwrap();
            let file_path = line[3..].to_string();

            if index_status == '?' && worktree_status == '?' {
                untracked.push(file_path);
            } else if index_status == 'U' || worktree_status == 'U' {
                conflicted.push(file_path);
            } else {
                if index_status != ' ' && index_status != '?' {
                    staged.push(GitFileStatus {
                        path: file_path.clone(),
                        status: Self::status_char_to_string(index_status),
                    });
                }
                if worktree_status != ' ' && worktree_status != '?' {
                    modified.push(GitFileStatus {
                        path: file_path.clone(),
                        status: Self::status_char_to_string(worktree_status),
                    });
                }
            }
        }

        Ok(GitStatus {
            branch,
            is_clean: staged.is_empty() && modified.is_empty() && untracked.is_empty() && conflicted.is_empty(),
            staged,
            modified,
            untracked,
            conflicted,
        })
    }

    fn get_current_branch(&self, path: &str) -> Result<String> {
        let full_path = self.server_root.join(path);
        
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            return Ok("main".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn commit(&self, path: &str, message: &str) -> Result<GitCommit> {
        let full_path = self.server_root.join(path);

        let output = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git add failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git commit failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        self.get_latest_commit(path)
    }

    pub fn get_latest_commit(&self, path: &str) -> Result<GitCommit> {
        let full_path = self.server_root.join(path);

        let output = Command::new("git")
            .args(["log", "-1", "--format=%H%n%h%n%an%n%ae%n%ai%n%s"])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() || output.stdout.is_empty() {
            anyhow::bail!("No commits found");
        }

        let lines: Vec<&str> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .collect();

        Ok(GitCommit {
            hash: lines.get(0).unwrap_or(&"").to_string(),
            short_hash: lines.get(1).unwrap_or(&"").to_string(),
            author: lines.get(2).unwrap_or(&"").to_string(),
            email: lines.get(3).unwrap_or(&"").to_string(),
            date: lines.get(4).unwrap_or(&"").to_string(),
            message: lines.get(5).unwrap_or(&"").to_string(),
        })
    }

    pub fn get_log(&self, path: &str, limit: usize) -> Result<Vec<GitCommit>> {
        let full_path = self.server_root.join(path);

        let output = Command::new("git")
            .args(["log", &format!("-{}", limit), "--format=%H%n%h%n%an%n%ae%n%ai%n%s"])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git log failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for chunk in content.split('\n').collect::<Vec<_>>().chunks(7) {
            if chunk.len() >= 7 {
                commits.push(GitCommit {
                    hash: chunk[0].to_string(),
                    short_hash: chunk[1].to_string(),
                    author: chunk[2].to_string(),
                    email: chunk[3].to_string(),
                    date: chunk[4].to_string(),
                    message: chunk[5].to_string(),
                });
            }
        }

        Ok(commits)
    }

    pub fn diff(&self, path: &str, target: Option<&str>) -> Result<Vec<GitDiff>> {
        let full_path = self.server_root.join(path);

        let args = if let Some(t) = target {
            vec!["diff", t]
        } else {
            vec!["diff", "--cached"]
        };

        let output = Command::new("git")
            .args(&args)
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git diff failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        self.parse_diff_output(&String::from_utf8_lossy(&output.stdout))
    }

    fn parse_diff_output(&self, output: &str) -> Result<Vec<GitDiff>> {
        let mut diffs = Vec::new();
        let mut current_file = None;
        let mut current_hunks: Vec<GitDiffHunk> = Vec::new();
        let mut current_hunk_lines: Vec<GitDiffLine> = Vec::new();
        let mut old_start = 0;
        let mut old_lines = 0;
        let mut new_start = 0;
        let mut new_lines = 0;

        for line in output.lines() {
            if line.starts_with("diff --git") {
                if let Some(file) = current_file.take() {
                    if !current_hunk_lines.is_empty() || !current_hunks.is_empty() {
                        if !current_hunk_lines.is_empty() {
                            current_hunks.push(GitDiffHunk {
                                old_start,
                                old_lines,
                                new_start,
                                new_lines,
                                lines: std::mem::take(&mut current_hunk_lines),
                            });
                        }
                        diffs.push(GitDiff {
                            file,
                            hunks: std::mem::take(&mut current_hunks),
                        });
                    }
                }
                
                if let Some(file_part) = line.split(" b/").nth(1) {
                    current_file = Some(file_part.to_string());
                }
            } else if line.starts_with("@@") {
                if !current_hunk_lines.is_empty() {
                    current_hunks.push(GitDiffHunk {
                        old_start,
                        old_lines,
                        new_start,
                        new_lines,
                        lines: std::mem::take(&mut current_hunk_lines),
                    });
                }

                let parts: Vec<&str> = line[4..].split("@@").collect();
                if let Some(hunk_info) = parts.first() {
                    let nums: Vec<&str> = hunk_info.trim().split(' ').collect();
                    if nums.len() >= 2 {
                        let old_part = nums[0];
                        let new_part = nums[1];
                        
                        if old_part.starts_with('-') {
                            let range: Vec<&str> = old_part[1..].split(',').collect();
                            old_start = range.first().unwrap_or(&"0").parse().unwrap_or(0);
                            old_lines = range.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                        }
                        
                        if new_part.starts_with('+') {
                            let range: Vec<&str> = new_part[1..].split(',').collect();
                            new_start = range.first().unwrap_or(&"0").parse().unwrap_or(0);
                            new_lines = range.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                        }
                    }
                }
            } else if let Some(ref file) = current_file {
                if !line.is_empty() {
                    let (line_type, content) = if line.starts_with('+') {
                        ("add", &line[1..])
                    } else if line.starts_with('-') {
                        ("delete", &line[1..])
                    } else if line.starts_with(' ') {
                        ("context", &line[1..])
                    } else {
                        ("context", line)
                    };

                    current_hunk_lines.push(GitDiffLine {
                        line_type: line_type.to_string(),
                        content: content.to_string(),
                    });
                }
            }
        }

        if let Some(file) = current_file {
            if !current_hunk_lines.is_empty() || !current_hunks.is_empty() {
                if !current_hunk_lines.is_empty() {
                    current_hunks.push(GitDiffHunk {
                        old_start,
                        old_lines,
                        new_start,
                        new_lines,
                        lines: current_hunk_lines,
                    });
                }
                diffs.push(GitDiff {
                    file,
                    hunks: current_hunks,
                });
            }
        }

        Ok(diffs)
    }

    pub fn blame(&self, file_path: &str) -> Result<GitBlame> {
        let full_path = self.server_root.join(file_path);

        let output = Command::new("git")
            .args(["blame", "--line-porcelain", file_path])
            .current_dir(full_path.parent().unwrap_or(&full_path))
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git blame failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let content = String::from_utf8_lossy(&output.stdout);
        let mut lines = Vec::new();
        let mut current_line: Option<GitBlameLine> = None;

        for line in content.lines() {
            if line.starts_with("\t") {
                if let Some(mut l) = current_line.take() {
                    l.content = line[1..].to_string();
                    lines.push(l);
                }
            } else if line.starts_with(" ") {
                if let Some(ref mut l) = current_line {
                    l.content = line[1..].to_string();
                }
            } else {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    let key = parts[0];
                    let value = parts[1];

                    match key {
                        "author" => {
                            if let Some(ref mut l) = current_line {
                                l.author = value.to_string();
                            }
                        }
                        "author-mail" => {
                            if let Some(ref mut l) = current_line {
                                if l.author.is_empty() {
                                    l.author = value.replace("<", " (").replace(">", ")");
                                }
                            }
                        }
                        "author-time" => {
                            if let Some(ref mut l) = current_line {
                                if let Ok(ts) = value.parse::<i64>() {
                                    l.date = chrono::DateTime::from_timestamp(ts, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                        .unwrap_or_default();
                                }
                            }
                        }
                        "summary" => {
                            if let Some(ref mut l) = current_line {
                                l.commit = value.to_string();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(GitBlame {
            file: file_path.to_string(),
            lines,
        })
    }

    pub fn create_branch(&self, path: &str, branch_name: &str) -> Result<()> {
        let full_path = self.server_root.join(path);

        let output = Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git branch creation failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    pub fn checkout(&self, path: &str, ref_name: &str) -> Result<()> {
        let full_path = self.server_root.join(path);

        let output = Command::new("git")
            .args(["checkout", ref_name])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git checkout failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    pub fn get_branches(&self, path: &str) -> Result<Vec<String>> {
        let full_path = self.server_root.join(path);

        let output = Command::new("git")
            .args(["branch", "-a"])
            .current_dir(&full_path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Git branch list failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|l| l.trim_start_matches(['*', ' ']).to_string())
            .filter(|b| !b.is_empty())
            .collect();

        Ok(branches)
    }

    fn status_char_to_string(status: char) -> String {
        match status {
            'M' => "modified".to_string(),
            'A' => "added".to_string(),
            'D' => "deleted".to_string(),
            'R' => "renamed".to_string(),
            'C' => "copied".to_string(),
            'U' => "unmerged".to_string(),
            _ => format!("unknown({})", status),
        }
    }
}

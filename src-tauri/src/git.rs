use git2::{Repository, StatusOptions, StatusShow};
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: String,
    pub changed_files: Vec<ChangedFile>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangedFile {
    pub path: String,
    pub status: String,
}

fn status_label(s: git2::Status) -> &'static str {
    if s.is_index_new() || s.is_wt_new() {
        "added"
    } else if s.is_index_modified() || s.is_wt_modified() {
        "modified"
    } else if s.is_index_deleted() || s.is_wt_deleted() {
        "deleted"
    } else if s.is_index_renamed() || s.is_wt_renamed() {
        "renamed"
    } else if s.is_index_typechange() || s.is_wt_typechange() {
        "typechange"
    } else {
        "unknown"
    }
}

pub fn get_git_status(project_path: &str) -> Option<GitStatus> {
    let repo = Repository::discover(project_path).ok()?;

    // Get current branch name
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD".to_string());

    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts)).ok()?;
    let repo_root = repo.workdir()?.to_path_buf();

    let changed_files: Vec<ChangedFile> = statuses
        .iter()
        .filter_map(|entry| {
            let path_str = entry.path()?;
            let status = entry.status();
            // Skip ignored files
            if status.is_ignored() {
                return None;
            }
            // Convert to absolute path
            let abs_path = repo_root.join(path_str);
            Some(ChangedFile {
                path: abs_path.to_string_lossy().replace('\\', "/"),
                status: status_label(status).to_string(),
            })
        })
        .collect();

    Some(GitStatus {
        is_repo: true,
        branch,
        changed_files,
    })
}

/// Returns list of absolute paths of files changed in git (modified, added, etc.)
pub fn get_changed_file_paths(project_path: &str) -> Vec<String> {
    get_git_status(project_path)
        .map(|s| {
            s.changed_files
                .into_iter()
                .filter(|f| f.status != "deleted")
                .map(|f| {
                    // Normalize to OS path
                    let p = Path::new(&f.path);
                    p.to_string_lossy().to_string()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Returns unified diff for a single file relative to HEAD
pub fn get_file_diff(project_path: &str, file_path: &str) -> Option<String> {
    let repo = Repository::discover(project_path).ok()?;
    let repo_root = repo.workdir()?.to_path_buf();

    // Get relative path from repo root
    let abs = Path::new(file_path);
    let rel = abs.strip_prefix(&repo_root).ok()?;

    // Diff working tree against HEAD
    let head_tree = repo.head().ok()?.peel_to_tree().ok()?;
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(rel.to_string_lossy().as_ref());

    let diff = repo
        .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_opts))
        .ok()?;

    let mut output = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        match origin {
            '+' | '-' | ' ' => output.push(origin),
            _ => {}
        }
        output.push_str(&String::from_utf8_lossy(line.content()));
        true
    })
    .ok()?;

    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

/// Returns diffs for all given file paths as a map of relative_path -> diff_string
pub fn get_diffs_for_files(project_path: &str, file_paths: &[String]) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    let repo_root = Repository::discover(project_path)
        .ok()
        .and_then(|r| r.workdir().map(|p| p.to_path_buf()));
    let root = match repo_root {
        Some(r) => r,
        None => return result,
    };

    for path in file_paths {
        if let Some(diff) = get_file_diff(project_path, path) {
            let rel = Path::new(path)
                .strip_prefix(&root)
                .unwrap_or(Path::new(path))
                .to_string_lossy()
                .replace('\\', "/");
            result.insert(rel, diff);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_label() {
        assert_eq!(status_label(git2::Status::WT_MODIFIED), "modified");
        assert_eq!(status_label(git2::Status::INDEX_NEW), "added");
        assert_eq!(status_label(git2::Status::WT_DELETED), "deleted");
        assert_eq!(status_label(git2::Status::INDEX_RENAMED), "renamed");
    }

    #[test]
    fn test_get_git_status_non_repo() {
        // A temp dir that is not a git repo
        let dir = tempfile::TempDir::new().unwrap();
        let result = get_git_status(&dir.path().to_string_lossy());
        assert!(result.is_none());
    }

    #[test]
    fn test_get_changed_file_paths_non_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        let paths = get_changed_file_paths(&dir.path().to_string_lossy());
        assert!(paths.is_empty());
    }
}

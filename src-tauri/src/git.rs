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

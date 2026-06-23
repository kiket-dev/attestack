use std::path::Path;
use std::process::Command;

use attestack_core::{sha256_digest, GitSnapshotPayload};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("not inside a git repository")]
    NotARepo,

    #[error("git command failed: {0}")]
    CommandFailed(String),
}

pub type Result<T> = std::result::Result<T, GitError>;

pub fn is_git_repo(repo_root: &Path) -> bool {
    git_output(repo_root, &["rev-parse", "--is-inside-work-tree"])
        .map(|output| output.trim() == "true")
        .unwrap_or(false)
}

pub fn capture_snapshot(repo_root: &Path, _include_diff: bool) -> Result<GitSnapshotPayload> {
    if !is_git_repo(repo_root) {
        return Err(GitError::NotARepo);
    }

    let head = git_output(repo_root, &["rev-parse", "HEAD"]).ok();
    let branch = git_output(repo_root, &["rev-parse", "--abbrev-ref", "HEAD"]).ok();
    let status = git_output(repo_root, &["status", "--porcelain"]).unwrap_or_default();
    let dirty = !status.trim().is_empty();

    let staged_diff_hash = hash_git_output(repo_root, &["diff", "--staged"]);
    let unstaged_diff_hash = hash_git_output(repo_root, &["diff"]);
    let untracked_files_hash = hash_untracked_files(repo_root);

    Ok(GitSnapshotPayload {
        repo_root_hash: Some(sha256_digest(
            repo_root
                .canonicalize()
                .unwrap_or_else(|_| repo_root.to_path_buf())
                .to_string_lossy()
                .as_bytes(),
        )),
        head,
        branch,
        dirty,
        staged_diff_hash,
        unstaged_diff_hash,
        untracked_files_hash,
        diff_artifact: None, // filled by caller after storing artifact
    })
}

pub fn snapshot_diff_bytes(repo_root: &Path) -> Result<Option<Vec<u8>>> {
    if !is_git_repo(repo_root) {
        return Err(GitError::NotARepo);
    }

    let mut diff = String::new();
    if let Ok(staged) = git_output(repo_root, &["diff", "--staged"]) {
        diff.push_str(&staged);
    }
    if let Ok(unstaged) = git_output(repo_root, &["diff"]) {
        diff.push_str(&unstaged);
    }
    if diff.is_empty() {
        Ok(None)
    } else {
        Ok(Some(diff.into_bytes()))
    }
}

fn hash_git_output(repo_root: &Path, args: &[&str]) -> Option<String> {
    git_output(repo_root, args)
        .ok()
        .filter(|output| !output.is_empty())
        .map(|output| sha256_digest(output.as_bytes()))
}

fn hash_untracked_files(repo_root: &Path) -> Option<String> {
    let output = git_output(repo_root, &["ls-files", "--others", "--exclude-standard"]).ok()?;
    let mut files: Vec<&str> = output.lines().filter(|line| !line.is_empty()).collect();
    if files.is_empty() {
        return None;
    }
    files.sort_unstable();
    Some(sha256_digest(files.join("\n").as_bytes()))
}

fn git_output(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|err| GitError::CommandFailed(err.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GitError::CommandFailed(if stderr.is_empty() {
            format!("git {} failed", args.join(" "))
        } else {
            stderr
        }));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn init_git_repo(path: &Path) {
        Command::new("git").args(["init", "-b", "main"]).current_dir(path).output().unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(path)
            .output()
            .unwrap();
        std::fs::write(path.join("README.md"), "hello\n").unwrap();
        Command::new("git").args(["add", "README.md"]).current_dir(path).output().unwrap();
        Command::new("git").args(["commit", "-m", "init"]).current_dir(path).output().unwrap();
    }

    #[test]
    fn capture_snapshot_in_git_repo() {
        let dir = tempdir().unwrap();
        init_git_repo(dir.path());
        let snapshot = capture_snapshot(dir.path(), false).unwrap();
        assert!(snapshot.head.is_some());
        assert_eq!(snapshot.branch.as_deref(), Some("main"));
        assert!(!snapshot.dirty);
    }

    #[test]
    fn not_a_repo_returns_error() {
        let dir = tempdir().unwrap();
        assert!(capture_snapshot(dir.path(), false).is_err());
    }
}

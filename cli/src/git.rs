use anyhow::{anyhow, Context, Result};
use git2::{Repository, StatusOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Wrapper around git2 Repository with helper methods
pub struct GitRepo {
    repo: Repository,
    path: PathBuf,
}

#[allow(dead_code)]
impl GitRepo {
    /// Open a repository at the given path or find one in parent directories
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path)
            .with_context(|| format!("Not a git repository: {}", path.display()))?;
        let path = repo
            .workdir()
            .ok_or_else(|| anyhow!("Bare repositories are not supported"))?
            .to_path_buf();
        Ok(Self { repo, path })
    }

    /// Get the repository path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head().context("Failed to get HEAD")?;
        if head.is_branch() {
            let name = head
                .shorthand()
                .ok_or_else(|| anyhow!("Invalid branch name"))?;
            Ok(name.to_string())
        } else {
            // Detached HEAD
            let commit = head.peel_to_commit()?;
            Ok(commit.id().to_string()[..8].to_string())
        }
    }

    /// Get list of staged files
    pub fn get_staged_files(&self) -> Result<Vec<String>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut staged = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_index_renamed()
                || status.is_index_typechange()
            {
                if let Some(path) = entry.path() {
                    staged.push(path.to_string());
                }
            }
        }

        Ok(staged)
    }

    /// Get list of unstaged modified files (for interactive selection)
    pub fn get_unstaged_files(&self) -> Result<Vec<String>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_wt_new()
                || status.is_wt_modified()
                || status.is_wt_deleted()
                || status.is_wt_renamed()
                || status.is_wt_typechange()
            {
                if let Some(path) = entry.path() {
                    files.push(path.to_string());
                }
            }
        }

        Ok(files)
    }

    /// Create a patch from staged changes
    /// Uses git diff --cached to capture exactly what's staged
    pub fn create_patch_from_staged(&self) -> Result<String> {
        // Use git command for more reliable patch format
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["diff", "--cached", "--binary"])
            .output()
            .context("Failed to run git diff")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("git diff failed: {}", stderr));
        }

        let patch = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in patch")?;

        if patch.trim().is_empty() {
            return Err(anyhow!("No staged changes to create patch from"));
        }

        Ok(patch)
    }

    /// Apply a patch to the working directory
    pub fn apply_patch(&self, patch_path: &Path) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["apply", "--index"])
            .arg(patch_path)
            .output()
            .context("Failed to run git apply")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("git apply failed: {}", stderr));
        }

        Ok(())
    }

    /// Stage specific files
    pub fn stage_files(&self, files: &[String]) -> Result<()> {
        if files.is_empty() {
            return Ok(());
        }

        let mut index = self.repo.index()?;
        for file in files {
            index.add_path(Path::new(file))?;
        }
        index.write()?;
        Ok(())
    }

    /// Unstage all staged files (git reset)
    pub fn unstage_all(&self) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["reset", "HEAD"])
            .output()
            .context("Failed to run git reset")?;

        // git reset returns 0 even if there's nothing to reset
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore "nothing to commit" type errors
            if !stderr.contains("nothing to commit") {
                return Err(anyhow!("git reset failed: {}", stderr));
            }
        }

        Ok(())
    }

    /// Create a commit with the staged changes
    pub fn commit(&self, message: &str) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["commit", "-m", message])
            .output()
            .context("Failed to run git commit")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("git commit failed: {}", stderr));
        }

        // Get the commit hash
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["rev-parse", "HEAD"])
            .output()
            .context("Failed to get commit hash")?;

        let hash = String::from_utf8(output.stdout)?
            .trim()
            .to_string();

        Ok(hash[..8].to_string())
    }

    /// Push to remote
    pub fn push(&self) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["push"])
            .output()
            .context("Failed to run git push")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("git push failed: {}", stderr));
        }

        Ok(())
    }

    /// Check if the current branch matches the expected branch
    pub fn verify_branch(&self, expected: &str) -> Result<bool> {
        let current = self.current_branch()?;
        Ok(current == expected)
    }
}

/// Read a patch file and return its contents
pub fn read_patch(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read patch file: {}", path.display()))
}

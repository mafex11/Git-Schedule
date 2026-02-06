use anyhow::{anyhow, Context, Result};
use git2::Repository;
use git_schedule_shared::Schedule;
use std::process::Command;
use tracing::{info, warn};

/// Execute a scheduled commit
pub async fn execute_schedule(schedule: &Schedule) -> Result<()> {
    // Verify repository exists
    if !schedule.repo_path.exists() {
        return Err(anyhow!(
            "Repository not found: {}",
            schedule.repo_path.display()
        ));
    }

    // Verify branch matches
    let current_branch = get_current_branch(&schedule.repo_path)?;
    if current_branch != schedule.branch {
        return Err(anyhow!(
            "Branch mismatch: expected '{}', found '{}'",
            schedule.branch,
            current_branch
        ));
    }

    // Verify patch file exists
    if !schedule.patch_file.exists() {
        return Err(anyhow!(
            "Patch file not found: {}",
            schedule.patch_file.display()
        ));
    }

    // Apply the patch
    apply_patch(&schedule.repo_path, &schedule.patch_file)?;

    // Create the commit
    let commit_hash = create_commit(&schedule.repo_path, &schedule.message)?;
    info!("Created commit {} in {}", commit_hash, schedule.repo_path.display());

    // Push if requested
    if schedule.push_after {
        push(&schedule.repo_path)?;
        info!("Pushed to remote");
    }

    // Clean up patch file
    if let Err(e) = std::fs::remove_file(&schedule.patch_file) {
        warn!("Failed to remove patch file: {}", e);
    }

    Ok(())
}

/// Get the current branch name
fn get_current_branch(repo_path: &std::path::Path) -> Result<String> {
    let repo = Repository::open(repo_path)?;
    let head = repo.head()?;

    if head.is_branch() {
        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    } else {
        // Detached HEAD
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string()[..8].to_string())
    }
}

/// Apply a patch file to the repository
fn apply_patch(repo_path: &std::path::Path, patch_path: &std::path::Path) -> Result<()> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["apply", "--index"])
        .arg(patch_path)
        .output()
        .context("Failed to run git apply")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Try to restore the patch to working directory for recovery
        let _ = restore_patch_to_staging(repo_path, patch_path);

        return Err(anyhow!("git apply failed: {}", stderr.trim()));
    }

    Ok(())
}

/// Restore patch to staging area on failure
fn restore_patch_to_staging(repo_path: &std::path::Path, patch_path: &std::path::Path) -> Result<()> {
    // Try applying without --index first (to working directory)
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["apply"])
        .arg(patch_path)
        .output()?;

    if output.status.success() {
        // Stage all changes
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["add", "-A"])
            .output()?;

        if output.status.success() {
            info!("Restored patch to staging area for retry");
            return Ok(());
        }
    }

    Err(anyhow!("Failed to restore patch"))
}

/// Create a commit with the staged changes
fn create_commit(repo_path: &std::path::Path, message: &str) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["commit", "-m", message])
        .output()
        .context("Failed to run git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git commit failed: {}", stderr.trim()));
    }

    // Get the commit hash
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["rev-parse", "HEAD"])
        .output()?;

    let hash = String::from_utf8(output.stdout)?
        .trim()
        .to_string();

    Ok(hash[..8].to_string())
}

/// Push to remote
fn push(repo_path: &std::path::Path) -> Result<()> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["push"])
        .output()
        .context("Failed to run git push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git push failed: {}", stderr.trim()));
    }

    Ok(())
}

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

    // Stash any working changes first to avoid conflicts
    let had_stash = stash_changes(&schedule.repo_path)?;

    // Apply the patch
    let apply_result = apply_patch_internal(&schedule.repo_path, &schedule.patch_file);

    if let Err(e) = apply_result {
        // Restore stash if we had one
        if had_stash {
            let _ = stash_pop(&schedule.repo_path);
        }
        return Err(e);
    }

    // Create the commit
    let commit_hash = create_commit(&schedule.repo_path, &schedule.message)?;
    info!("Created commit {} in {}", commit_hash, schedule.repo_path.display());

    // Push if requested
    if schedule.push_after {
        push(&schedule.repo_path)?;
        info!("Pushed to remote");
    }

    // Restore stash if we had one
    if had_stash {
        stash_pop(&schedule.repo_path)?;
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

/// Apply a patch file to the repository (internal, called after stash)
fn apply_patch_internal(repo_path: &std::path::Path, patch_path: &std::path::Path) -> Result<()> {
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

/// Stash working directory changes, returns true if there was something to stash
fn stash_changes(repo_path: &std::path::Path) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["stash", "push", "-m", "git-schedule-temp"])
        .output()
        .context("Failed to run git stash")?;

    // Check if anything was stashed
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(!stdout.contains("No local changes to save"))
}

/// Pop the stash after commit
fn stash_pop(repo_path: &std::path::Path) -> Result<()> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["stash", "pop"])
        .output()
        .context("Failed to run git stash pop")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("git stash pop had issues: {}", stderr.trim());
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

/// Execute a scheduled PR
pub async fn execute_pr_schedule(schedule: &Schedule) -> Result<()> {
    // Verify repository exists
    if !schedule.repo_path.exists() {
        return Err(anyhow!(
            "Repository not found: {}",
            schedule.repo_path.display()
        ));
    }

    // Verify patch file exists
    if !schedule.patch_file.exists() {
        return Err(anyhow!(
            "Patch file not found: {}",
            schedule.patch_file.display()
        ));
    }

    let pr_target = schedule
        .pr_target
        .as_deref()
        .ok_or_else(|| anyhow!("PR target branch not set"))?;

    // Determine the working branch
    let work_branch = if let Some(ref new_branch) = schedule.pr_branch {
        // Create a new branch
        let output = Command::new("git")
            .current_dir(&schedule.repo_path)
            .args(["checkout", "-b", new_branch])
            .output()
            .context("Failed to create new branch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create branch '{}': {}", new_branch, stderr.trim()));
        }
        new_branch.clone()
    } else {
        // Use current branch — verify it matches
        let current = get_current_branch(&schedule.repo_path)?;
        if current != schedule.branch {
            return Err(anyhow!(
                "Branch mismatch: expected '{}', found '{}'",
                schedule.branch,
                current
            ));
        }
        schedule.branch.clone()
    };

    // Stash any working changes
    let had_stash = stash_changes(&schedule.repo_path)?;

    // Apply the patch
    let apply_result = apply_patch_internal(&schedule.repo_path, &schedule.patch_file);

    if let Err(e) = apply_result {
        if had_stash {
            let _ = stash_pop(&schedule.repo_path);
        }
        // Switch back if we created a new branch
        if schedule.pr_branch.is_some() {
            let _ = Command::new("git")
                .current_dir(&schedule.repo_path)
                .args(["checkout", &schedule.branch])
                .output();
        }
        return Err(e);
    }

    // Create the commit
    let commit_hash = create_commit(&schedule.repo_path, &schedule.message)?;
    info!("Created commit {} for PR", commit_hash);

    // Push the branch
    push(&schedule.repo_path)?;
    info!("Pushed branch {} to remote", work_branch);

    // Create PR using gh CLI
    let mut gh_args = vec![
        "pr".to_string(),
        "create".to_string(),
        "--base".to_string(),
        pr_target.to_string(),
        "--head".to_string(),
        work_branch.clone(),
        "--title".to_string(),
        schedule.message.clone(),
    ];

    if let Some(ref body) = schedule.pr_body {
        gh_args.push("--body".to_string());
        gh_args.push(body.clone());
    } else {
        gh_args.push("--body".to_string());
        gh_args.push(String::new());
    }

    if schedule.pr_draft {
        gh_args.push("--draft".to_string());
    }

    let gh_output = Command::new("gh")
        .current_dir(&schedule.repo_path)
        .args(&gh_args)
        .output()
        .context("Failed to run gh CLI. Is GitHub CLI installed?")?;

    if !gh_output.status.success() {
        let stderr = String::from_utf8_lossy(&gh_output.stderr);
        // Restore stash before failing
        if had_stash {
            let _ = stash_pop(&schedule.repo_path);
        }
        return Err(anyhow!("gh pr create failed: {}", stderr.trim()));
    }

    let pr_url = String::from_utf8_lossy(&gh_output.stdout).trim().to_string();
    info!("Created PR: {}", pr_url);

    // Restore stash
    if had_stash {
        stash_pop(&schedule.repo_path)?;
    }

    // Clean up patch file
    if let Err(e) = std::fs::remove_file(&schedule.patch_file) {
        warn!("Failed to remove patch file: {}", e);
    }

    Ok(())
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

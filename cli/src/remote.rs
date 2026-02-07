use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use console::style;
use std::path::Path;
use std::process::Command;

use crate::time_parser::{format_absolute, format_relative};

const WORKFLOW_PATH: &str = ".github/workflows/git-schedule-remote.yml";

const WORKFLOW_TEMPLATE: &str = r#"name: Git Schedule Remote

on:
  schedule:
    - cron: '*/5 * * * *'
  workflow_dispatch: {}

permissions:
  contents: write
  actions: write

jobs:
  execute:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Fetch all refs
        run: |
          git fetch --tags
          git fetch origin 'refs/heads/git-schedule/*:refs/remotes/origin/git-schedule/*'

      - name: Execute due schedules
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          executed=0
          failed=0

          for tag in $(git tag -l 'git-schedule-meta/*'); do
            id="${tag#git-schedule-meta/}"
            metadata=$(git tag -l --format='%(contents)' "$tag" | head -1)

            if [ -z "$metadata" ]; then
              echo "Skipping $id: no metadata"
              continue
            fi

            scheduled_at=$(echo "$metadata" | jq -r '.scheduled_at')
            target=$(echo "$metadata" | jq -r '.target_branch')
            author_name=$(echo "$metadata" | jq -r '.author_name')
            author_email=$(echo "$metadata" | jq -r '.author_email')

            # Compare timestamps
            sched_ts=$(date -u -d "$scheduled_at" +%s 2>/dev/null) || continue
            now_ts=$(date -u +%s)

            if [ "$now_ts" -lt "$sched_ts" ]; then
              echo "Schedule $id not yet due ($(date -u -d "$scheduled_at"))"
              continue
            fi

            echo "Executing schedule $id..."

            # Configure git identity from original author
            git config user.name "$author_name"
            git config user.email "$author_email"

            branch="git-schedule/$id"
            commit=$(git rev-parse "origin/$branch" 2>/dev/null)
            if [ -z "$commit" ]; then
              echo "Branch $branch not found, skipping"
              continue
            fi

            # Cherry-pick to target branch
            git checkout "$target" 2>/dev/null || git checkout -b "$target" "origin/$target"
            git pull origin "$target" --ff-only 2>/dev/null || true

            if git cherry-pick "$commit"; then
              git push origin "$target"
              git push origin --delete "$branch" 2>/dev/null || true
              git tag -d "$tag" 2>/dev/null || true
              git push origin --delete "refs/tags/$tag" 2>/dev/null || true
              echo "Successfully executed schedule $id"
              executed=$((executed + 1))
            else
              git cherry-pick --abort 2>/dev/null || true
              echo "Failed to cherry-pick schedule $id (likely a conflict)"
              failed=$((failed + 1))
            fi
          done

          echo ""
          echo "Done: $executed executed, $failed failed"

          # Auto-disable workflow if no schedules remain
          remaining=$(git tag -l 'git-schedule-meta/*' | wc -l)
          if [ "$remaining" -eq 0 ]; then
            echo "No remaining schedules, disabling workflow"
            gh workflow disable "Git Schedule Remote"
          fi
"#;

/// Get the user's git name and email from config
fn get_git_user(repo_path: &Path) -> Result<(String, String)> {
    let name_output = Command::new("git")
        .current_dir(repo_path)
        .args(["config", "user.name"])
        .output()
        .context("Failed to run git config user.name")?;

    let name = String::from_utf8(name_output.stdout)?
        .trim()
        .to_string();

    if name.is_empty() {
        return Err(anyhow!("git user.name is not set. Run: git config user.name \"Your Name\""));
    }

    let email_output = Command::new("git")
        .current_dir(repo_path)
        .args(["config", "user.email"])
        .output()
        .context("Failed to run git config user.email")?;

    let email = String::from_utf8(email_output.stdout)?
        .trim()
        .to_string();

    if email.is_empty() {
        return Err(anyhow!("git user.email is not set. Run: git config user.email \"you@example.com\""));
    }

    Ok((name, email))
}

/// Check if a remote named 'origin' exists
fn has_remote(repo_path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to check git remote")?;

    Ok(output.status.success())
}

/// Create a remote schedule: branch + tag + push
pub fn create_remote_schedule(
    repo_path: &Path,
    branch: &str,
    message: &str,
    scheduled_at: DateTime<Utc>,
    patch_path: &Path,
) -> Result<String> {
    // Verify remote exists
    if !has_remote(repo_path)? {
        return Err(anyhow!(
            "No 'origin' remote found. Add one with: git remote add origin <url>"
        ));
    }

    let (author_name, author_email) = get_git_user(repo_path)?;

    // Generate schedule ID
    let id = uuid::Uuid::new_v4().to_string();
    let id = &id[..8];
    let temp_branch = format!("git-schedule/{}", id);

    // Stash any working tree changes so the tree is clean for applying the patch
    let stash_output = run_git(repo_path, &["stash", "push", "-m", "git-schedule-remote-temp"])?;
    let had_stash = !stash_output.contains("No local changes to save");

    // Create temp branch from current HEAD
    run_git(repo_path, &["checkout", "-b", &temp_branch])?;

    // Apply patch and commit
    let apply_result = run_git(repo_path, &["apply", "--index", &patch_path.to_string_lossy()]);

    if let Err(e) = apply_result {
        let _ = run_git(repo_path, &["checkout", branch]);
        let _ = run_git(repo_path, &["branch", "-D", &temp_branch]);
        if had_stash { let _ = run_git(repo_path, &["stash", "pop"]); }
        return Err(e).context("Failed to apply patch on temp branch");
    }

    let commit_result = run_git(repo_path, &["commit", "-m", message]);

    if let Err(e) = commit_result {
        let _ = run_git(repo_path, &["checkout", branch]);
        let _ = run_git(repo_path, &["branch", "-D", &temp_branch]);
        if had_stash { let _ = run_git(repo_path, &["stash", "pop"]); }
        return Err(e).context("Failed to create commit on temp branch");
    }

    // Create annotated tag with metadata
    let metadata = serde_json::json!({
        "target_branch": branch,
        "scheduled_at": scheduled_at.to_rfc3339(),
        "author_name": author_name,
        "author_email": author_email,
    });
    let tag_name = format!("git-schedule-meta/{}", id);

    let tag_result = run_git(
        repo_path,
        &["tag", "-a", &tag_name, "-m", &metadata.to_string()],
    );

    if let Err(e) = tag_result {
        let _ = run_git(repo_path, &["checkout", branch]);
        let _ = run_git(repo_path, &["branch", "-D", &temp_branch]);
        if had_stash { let _ = run_git(repo_path, &["stash", "pop"]); }
        return Err(e).context("Failed to create metadata tag");
    }

    // Enable the workflow in case it was auto-disabled
    let _ = Command::new("gh")
        .current_dir(repo_path)
        .args(["workflow", "enable", "Git Schedule Remote"])
        .output();

    // Push branch and tag
    let push_result = run_git(
        repo_path,
        &["push", "origin", &temp_branch, &tag_name],
    );

    if let Err(e) = push_result {
        let _ = run_git(repo_path, &["checkout", branch]);
        let _ = run_git(repo_path, &["branch", "-D", &temp_branch]);
        let _ = run_git(repo_path, &["tag", "-d", &tag_name]);
        if had_stash { let _ = run_git(repo_path, &["stash", "pop"]); }
        return Err(e).context("Failed to push to remote");
    }

    // Switch back to original branch and clean up local temp branch
    run_git(repo_path, &["checkout", branch])?;
    let _ = run_git(repo_path, &["branch", "-D", &temp_branch]);

    // Drop the stash — changes are captured on the remote branch
    if had_stash {
        let _ = run_git(repo_path, &["stash", "drop"]);
    }

    // Display confirmation
    println!();
    println!(
        "{} Scheduled remote commit for {} (in {})",
        style("✓").green().bold(),
        style(format_absolute(scheduled_at)).cyan(),
        format_relative(scheduled_at)
    );
    println!();
    println!("  {} {}", style("ID:").dim(), id);
    println!("  {} {}", style("Message:").dim(), truncate(message, 50));
    println!("  {} {}", style("Branch:").dim(), branch);
    println!("  {} remote (GitHub Actions)", style("Mode:").dim());
    println!();
    println!(
        "{}",
        style("GitHub Actions will execute this within ~5 minutes of the scheduled time.").dim()
    );

    Ok(id.to_string())
}

/// Ensure the GitHub Actions workflow file exists in the repo
pub fn ensure_workflow_exists(repo_path: &Path, branch: &str) -> Result<()> {
    let workflow_file = repo_path.join(WORKFLOW_PATH);

    if workflow_file.exists() {
        return Ok(());
    }

    println!(
        "{}",
        style("Setting up GitHub Actions workflow for remote scheduling...").yellow()
    );

    // Create the workflow directory and file
    let workflow_dir = workflow_file.parent().unwrap();
    std::fs::create_dir_all(workflow_dir)
        .context("Failed to create .github/workflows directory")?;

    std::fs::write(&workflow_file, WORKFLOW_TEMPLATE)
        .context("Failed to write workflow file")?;

    // Commit and push the workflow file
    run_git(repo_path, &["add", WORKFLOW_PATH])?;
    run_git(
        repo_path,
        &["commit", "-m", "chore: add git-schedule remote workflow"],
    )?;
    run_git(repo_path, &["push", "origin", branch])?;

    println!(
        "{} Created {}",
        style("✓").green(),
        style(WORKFLOW_PATH).cyan()
    );

    Ok(())
}

/// Cancel a remote schedule by deleting the remote branch and tag
pub fn cancel_remote_schedule(repo_path: &Path, id: &str) -> Result<()> {
    if !has_remote(repo_path)? {
        return Err(anyhow!("No 'origin' remote found"));
    }

    let branch = format!("git-schedule/{}", id);
    let tag = format!("git-schedule-meta/{}", id);

    // Delete remote branch
    let branch_result = run_git(repo_path, &["push", "origin", "--delete", &branch]);
    // Delete remote tag
    let tag_result = run_git(
        repo_path,
        &["push", "origin", "--delete", &format!("refs/tags/{}", tag)],
    );

    // Delete local tag if it exists
    let _ = run_git(repo_path, &["tag", "-d", &tag]);

    if branch_result.is_err() && tag_result.is_err() {
        return Err(anyhow!(
            "Remote schedule '{}' not found. Check the ID with: git branch -r | grep git-schedule",
            id
        ));
    }

    println!(
        "{} Cancelled remote schedule {}",
        style("✓").green(),
        style(id).cyan()
    );

    Ok(())
}

/// Run a git command and return the output
fn run_git(repo_path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run: git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.trim().to_string())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

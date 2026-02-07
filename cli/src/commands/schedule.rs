use anyhow::{anyhow, Result};
use console::style;
use git_schedule_shared::config;
use std::env;

use crate::client::{ensure_daemon_running, Client};
use crate::git::GitRepo;
use crate::interactive::select_files_to_stage;
use crate::remote;
use crate::time_parser::{format_absolute, format_relative, parse_absolute, parse_relative};

pub async fn run(
    message: String,
    in_time: Option<String>,
    at_time: Option<String>,
    push: bool,
    remote: bool,
) -> Result<()> {
    // Parse the scheduled time
    let scheduled_at = match (in_time, at_time) {
        (Some(rel), None) => parse_relative(&rel)?,
        (None, Some(abs)) => parse_absolute(&abs)?,
        (Some(_), Some(_)) => return Err(anyhow!("Use --in OR --at, not both")),
        (None, None) => return Err(anyhow!("Must specify --in or --at")),
    };

    // Open repository
    let repo_path = env::current_dir()?;
    let repo = GitRepo::open(&repo_path)?;
    let branch = repo.current_branch()?;

    // Check for staged files
    let staged = repo.get_staged_files()?;
    if staged.is_empty() {
        // Interactive file selection
        let selected = select_files_to_stage(&repo)?;
        repo.stage_files(&selected)?;
        println!("{} Staged {} file(s)", style("✓").green(), selected.len());
    }

    if remote {
        return run_remote(&repo, &repo_path, &branch, &message, scheduled_at).await;
    }

    // Local daemon path
    ensure_daemon_running().await?;

    // Connect to daemon and check queue limit
    let mut client = Client::connect().await?;
    let status = client.get_status().await?;

    if status.pending_count >= config::MAX_SCHEDULES {
        return Err(anyhow!(
            "Queue full ({} schedules). Cancel some first with: git-schedule cancel <id>",
            config::MAX_SCHEDULES
        ));
    }

    // Create patch from staged changes
    let patch_content = repo.create_patch_from_staged()?;

    // Save patch to file
    let patches_dir = config::patches_dir()?;
    std::fs::create_dir_all(&patches_dir)?;
    let patch_file = patches_dir.join(format!("{}.patch", uuid::Uuid::new_v4()));
    std::fs::write(&patch_file, &patch_content)?;

    // Unstage files (like after a real commit)
    repo.unstage_all()?;

    // Send schedule to daemon
    let schedule = client
        .create_schedule(
            message.clone(),
            scheduled_at,
            repo_path.clone(),
            branch.clone(),
            patch_file,
            push,
        )
        .await?;

    // Display confirmation
    println!();
    println!(
        "{} Scheduled commit for {} (in {})",
        style("✓").green().bold(),
        style(format_absolute(scheduled_at)).cyan(),
        format_relative(scheduled_at)
    );
    println!();
    println!("  {} {}", style("ID:").dim(), schedule.id);
    println!("  {} {}", style("Message:").dim(), truncate(&message, 50));
    println!("  {} {}", style("Branch:").dim(), branch);

    let _staged_count = repo.get_staged_files().unwrap_or_default().len();
    println!(
        "  {} {} file(s) captured",
        style("Files:").dim(),
        patch_content.matches("diff --git").count()
    );

    if push {
        println!("  {} Yes", style("Push:").dim());
    }

    println!();
    println!(
        "{}",
        style("Run 'git-schedule list' to see all scheduled commits").dim()
    );

    Ok(())
}

async fn run_remote(
    repo: &GitRepo,
    repo_path: &std::path::Path,
    branch: &str,
    message: &str,
    scheduled_at: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    // Create patch from staged changes
    let patch_content = repo.create_patch_from_staged()?;

    // Save patch to temp file
    let patches_dir = config::patches_dir()?;
    std::fs::create_dir_all(&patches_dir)?;
    let patch_file = patches_dir.join(format!("{}.patch", uuid::Uuid::new_v4()));
    std::fs::write(&patch_file, &patch_content)?;

    // Unstage files (like after a real commit)
    repo.unstage_all()?;

    // Ensure workflow exists first (commits to current branch)
    remote::ensure_workflow_exists(repo_path, branch)?;

    // Create remote schedule (branch + tag + push)
    let result = remote::create_remote_schedule(
        repo_path,
        branch,
        message,
        scheduled_at,
        &patch_file,
    );

    // Clean up local patch file regardless of result
    let _ = std::fs::remove_file(&patch_file);

    result?;

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

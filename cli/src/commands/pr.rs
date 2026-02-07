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
    title: String,
    to: String,
    branch: Option<String>,
    body: Option<String>,
    draft: bool,
    in_time: Option<String>,
    at_time: Option<String>,
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
    let current_branch = repo.current_branch()?;

    // Check for staged files
    let staged = repo.get_staged_files()?;
    if staged.is_empty() {
        let selected = select_files_to_stage(&repo)?;
        repo.stage_files(&selected)?;
        println!("{} Staged {} file(s)", style("✓").green(), selected.len());
    }

    if remote {
        return run_remote(
            &repo,
            &repo_path,
            &current_branch,
            &title,
            &to,
            branch,
            body,
            draft,
            scheduled_at,
        )
        .await;
    }

    // Local daemon path
    ensure_daemon_running().await?;

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
    let patches_dir = config::patches_dir()?;
    std::fs::create_dir_all(&patches_dir)?;
    let patch_file = patches_dir.join(format!("{}.patch", uuid::Uuid::new_v4()));
    std::fs::write(&patch_file, &patch_content)?;

    repo.unstage_all()?;

    let schedule = client
        .create_pr_schedule(
            title.clone(),
            scheduled_at,
            repo_path,
            current_branch.clone(),
            patch_file,
            to.clone(),
            body.clone(),
            draft,
            branch.clone(),
        )
        .await?;

    // Display confirmation
    println!();
    println!(
        "{} Scheduled PR for {} (in {})",
        style("✓").green().bold(),
        style(format_absolute(scheduled_at)).cyan(),
        format_relative(scheduled_at)
    );
    println!();
    println!("  {} {}", style("ID:").dim(), schedule.id);
    println!("  {} {}", style("Title:").dim(), truncate(&title, 50));
    println!(
        "  {} {} → {}",
        style("Branches:").dim(),
        branch.as_deref().unwrap_or(&current_branch),
        to
    );
    if draft {
        println!("  {} Yes", style("Draft:").dim());
    }
    println!(
        "  {} {} file(s) captured",
        style("Files:").dim(),
        patch_content.matches("diff --git").count()
    );

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
    current_branch: &str,
    title: &str,
    to: &str,
    branch: Option<String>,
    body: Option<String>,
    draft: bool,
    scheduled_at: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    let patch_content = repo.create_patch_from_staged()?;
    let patches_dir = config::patches_dir()?;
    std::fs::create_dir_all(&patches_dir)?;
    let patch_file = patches_dir.join(format!("{}.patch", uuid::Uuid::new_v4()));
    std::fs::write(&patch_file, &patch_content)?;

    repo.unstage_all()?;

    remote::ensure_workflow_exists(repo_path, current_branch)?;

    let result = remote::create_remote_pr_schedule(
        repo_path,
        current_branch,
        title,
        to,
        branch.clone(),
        body,
        draft,
        scheduled_at,
        &patch_file,
    );

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

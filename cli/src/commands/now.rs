use anyhow::Result;
use chrono::Utc;
use console::style;
use git_schedule_shared::config;
use std::env;

use crate::client::{ensure_daemon_running, Client};
use crate::git::GitRepo;
use crate::interactive::select_files_to_stage;

pub async fn run(message: String, push: bool) -> Result<()> {
    // Open repository
    let repo_path = env::current_dir()?;
    let repo = GitRepo::open(&repo_path)?;
    let branch = repo.current_branch()?;

    // Check for staged files
    let staged = repo.get_staged_files()?;
    if staged.is_empty() {
        // Interactive file selection
        let selected = select_files_to_stage(&repo)?;
        if selected.is_empty() {
            println!("{} No files selected", style("!").yellow());
            return Ok(());
        }
        repo.stage_files(&selected)?;
        println!("{} Staged {} file(s)", style("✓").green(), selected.len());
    }

    // Ensure daemon is running
    ensure_daemon_running().await?;

    // Create patch from staged changes
    let patch_content = repo.create_patch_from_staged()?;

    // Save patch to file
    let patches_dir = config::patches_dir()?;
    std::fs::create_dir_all(&patches_dir)?;
    let patch_file = patches_dir.join(format!("{}.patch", uuid::Uuid::new_v4()));
    std::fs::write(&patch_file, &patch_content)?;

    // Unstage files
    repo.unstage_all()?;

    // Connect and create schedule for now (immediate execution)
    let mut client = Client::connect().await?;
    let scheduled_at = Utc::now();

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

    println!("{} Committing now...", style("●").cyan());
    println!();
    println!("  {} {}", style(&schedule.id).dim(), message);
    println!(
        "  {} @ {}",
        repo_path.file_name().unwrap().to_string_lossy(),
        branch
    );
    if push {
        println!("  {}", style("[push]").dim());
    }

    Ok(())
}

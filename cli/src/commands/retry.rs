use anyhow::{anyhow, Result};
use console::style;
use git_schedule_shared::ScheduleStatus;

use crate::client::{ensure_daemon_running, Client};
use crate::git::{read_patch, GitRepo};

pub async fn run(id: &str) -> Result<()> {
    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;
    let schedule = client.get_schedule(id).await?;

    // Verify it's a failed/missed schedule
    if !matches!(
        schedule.status,
        ScheduleStatus::Failed | ScheduleStatus::Missed
    ) {
        return Err(anyhow!(
            "Schedule {} is not failed or missed (status: {})",
            id,
            schedule.status
        ));
    }

    // Open the repository
    let repo = GitRepo::open(&schedule.repo_path)?;

    // Verify we're in the right repo
    let current_dir = std::env::current_dir()?;
    if !current_dir.starts_with(&schedule.repo_path) {
        println!(
            "{} You're not in the repository where this commit was scheduled.",
            style("Warning:").yellow()
        );
        println!(
            "  Expected: {}",
            schedule.repo_path.display()
        );
        println!("  Current:  {}", current_dir.display());
        println!();
    }

    // Read the patch
    let patch_content = read_patch(&schedule.patch_file)?;

    // Write patch to temp file and apply it
    let temp_patch = std::env::temp_dir().join(format!("git-schedule-retry-{}.patch", id));
    std::fs::write(&temp_patch, &patch_content)?;

    // Apply patch (stages the files)
    repo.apply_patch(&temp_patch)?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_patch);

    // Remove from failed queue via daemon
    client.retry_failed(id).await?;

    println!(
        "{} Files from schedule {} have been re-staged",
        style("✓").green(),
        style(id).cyan()
    );
    println!();
    println!("  {} {}", style("Message:").dim(), schedule.message);
    println!(
        "  {} {} file(s) staged",
        style("Files:").dim(),
        patch_content.matches("diff --git").count()
    );
    println!();
    println!(
        "{}",
        style("The files are now staged. To reschedule, run:").dim()
    );
    println!(
        "  {}",
        style(format!("git-schedule \"{}\" --in <time>", schedule.message)).cyan()
    );

    Ok(())
}

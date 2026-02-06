use anyhow::Result;
use chrono::Local;
use console::style;

use crate::client::{ensure_daemon_running, Client};

pub async fn run() -> Result<()> {
    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;

    // Get recent completed schedules (last 20)
    let completed = client.get_completed(20).await?;

    println!("{}", style("Completed Commits").bold());
    println!();

    if completed.is_empty() {
        println!("{}", style("  No completed commits yet").dim());
        return Ok(());
    }

    let completed_count = completed.len();
    for schedule in &completed {
        let local_time = schedule.scheduled_at.with_timezone(&Local);
        let time_str = local_time.format("%b %d, %I:%M %p").to_string();

        let repo_name = schedule
            .repo_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!(
            "  {} {} {}",
            style("✓").green(),
            style(&schedule.id).dim(),
            schedule.message
        );
        println!(
            "      {} @ {} {}",
            repo_name,
            schedule.branch,
            if schedule.push_after {
                style("[pushed]").dim().to_string()
            } else {
                String::new()
            }
        );
        println!("      {}", style(time_str).dim());
        println!();
    }

    // Summary
    let status = client.get_status().await?;
    println!("{}", style("─".repeat(50)).dim());
    println!(
        "  Total: {} completed | {} pending | {} failed",
        style(completed_count).green(),
        style(status.pending_count).cyan(),
        if status.failed_count > 0 {
            style(status.failed_count).red().to_string()
        } else {
            style(status.failed_count).dim().to_string()
        }
    );

    Ok(())
}

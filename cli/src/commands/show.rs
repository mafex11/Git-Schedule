use anyhow::Result;
use console::style;

use crate::client::{ensure_daemon_running, Client};
use crate::git::read_patch;
use crate::time_parser::{format_absolute, format_relative};

pub async fn run(id: &str) -> Result<()> {
    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;
    let schedule = client.get_schedule(id).await?;

    // Header
    println!("{}", style("Schedule Details").bold());
    println!("{}", style("─".repeat(60)).dim());
    println!("  {} {}", style("ID:").dim(), schedule.id);
    println!("  {} {}", style("Message:").dim(), schedule.message);
    println!(
        "  {} {} ({})",
        style("Scheduled:").dim(),
        format_absolute(schedule.scheduled_at),
        format_relative(schedule.scheduled_at)
    );

    let repo_name = schedule
        .repo_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!(
        "  {} {} @ {}",
        style("Repo:").dim(),
        repo_name,
        schedule.branch
    );

    if schedule.push_after {
        println!("  {} Yes", style("Push:").dim());
    }

    // Read and display patch
    println!();
    println!("{}", style("Diff").bold());
    println!("{}", style("─".repeat(60)).dim());

    let patch = read_patch(&schedule.patch_file)?;

    // Colorize the diff output
    for line in patch.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            println!("{}", style(line).green());
        } else if line.starts_with('-') && !line.starts_with("---") {
            println!("{}", style(line).red());
        } else if line.starts_with("@@") {
            println!("{}", style(line).cyan());
        } else if line.starts_with("diff --git") {
            println!();
            println!("{}", style(line).bold());
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}

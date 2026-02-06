use anyhow::Result;
use console::style;
use git_schedule_shared::config;

use crate::client::Client;
use crate::time_parser::{format_absolute, format_relative};

pub async fn run() -> Result<()> {
    let socket_path = config::socket_path()?;

    println!("{}", style("git-schedule Status").bold());
    println!();

    // Check if daemon is running
    if !socket_path.exists() {
        println!("{} Daemon not running", style("●").red());
        println!();
        println!(
            "{}",
            style("Start it with: git-schedule daemon start").dim()
        );
        return Ok(());
    }

    // Try to connect
    let client_result = Client::connect().await;
    let mut client = match client_result {
        Ok(c) => c,
        Err(_) => {
            println!("{} Daemon not responding", style("●").red());
            println!();
            println!(
                "{}",
                style("Try restarting: git-schedule daemon restart").dim()
            );
            return Ok(());
        }
    };

    let status = client.get_status().await?;

    // Daemon status
    if status.running {
        println!(
            "{} Daemon running (PID: {})",
            style("●").green(),
            status.pid.unwrap_or(0)
        );
        if let Some(uptime) = status.uptime_seconds {
            println!("  Uptime: {}", format_duration(uptime));
        }
    } else {
        println!("{} Daemon not running", style("●").red());
    }

    println!();
    println!(
        "{}: {}",
        style("Pending").dim(),
        style(status.pending_count).cyan()
    );
    println!(
        "{}: {}",
        style("Failed").dim(),
        if status.failed_count > 0 {
            style(status.failed_count).red()
        } else {
            style(status.failed_count).dim()
        }
    );

    // Next scheduled commit
    if let Some(next) = status.next_schedule {
        println!();
        println!("{}", style("Next Commit").bold());
        println!(
            "  {} {} ({})",
            style("Time:").dim(),
            style(format_absolute(next.scheduled_at)).green(),
            format_relative(next.scheduled_at)
        );
        println!(
            "  {} {}",
            style("Message:").dim(),
            truncate(&next.message, 40)
        );
        let repo_name = next
            .repo_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        println!(
            "  {} {} @ {}",
            style("Repo:").dim(),
            repo_name,
            next.branch
        );
    }

    if status.failed_count > 0 {
        println!();
        println!(
            "{}",
            style("Run 'git-schedule failed' to see failed commits").yellow()
        );
    }

    Ok(())
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

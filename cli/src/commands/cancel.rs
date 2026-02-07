use anyhow::Result;
use console::style;
use std::env;

use crate::client::{ensure_daemon_running, Client};
use crate::remote;

pub async fn run(id: &str, remote: bool) -> Result<()> {
    if remote {
        let repo_path = env::current_dir()?;
        return remote::cancel_remote_schedule(&repo_path, id);
    }

    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;

    // First get the schedule to show details
    let schedule = client.get_schedule(id).await?;

    // Cancel it
    client.cancel_schedule(id).await?;

    // Delete the patch file
    if schedule.patch_file.exists() {
        let _ = std::fs::remove_file(&schedule.patch_file);
    }

    println!(
        "{} Cancelled schedule {}",
        style("✓").green(),
        style(id).cyan()
    );
    println!(
        "  {} {}",
        style("Message:").dim(),
        truncate(&schedule.message, 50)
    );

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

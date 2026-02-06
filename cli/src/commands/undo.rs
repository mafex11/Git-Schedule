use anyhow::Result;
use console::style;

use crate::client::{ensure_daemon_running, Client};

pub async fn run() -> Result<()> {
    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;

    // Get the most recent pending schedule
    let schedule = match client.get_most_recent().await {
        Ok(s) => s,
        Err(_) => {
            println!("{} No pending schedules to undo", style("●").dim());
            return Ok(());
        }
    };

    // Cancel it
    client.cancel_schedule(schedule.id.as_str()).await?;

    println!("{} Undone: {}", style("✓").green(), schedule.message);
    println!(
        "  {} {}",
        style("ID:").dim(),
        style(schedule.id.as_str()).dim()
    );

    Ok(())
}

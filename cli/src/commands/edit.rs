use anyhow::{anyhow, Result};
use console::style;

use crate::client::{ensure_daemon_running, Client};
use crate::time_parser::{format_absolute, format_relative, parse_absolute, parse_relative};

pub async fn run(
    id: &str,
    message: Option<String>,
    in_time: Option<String>,
    at_time: Option<String>,
) -> Result<()> {
    // Parse new time if provided
    let scheduled_at = match (in_time, at_time) {
        (Some(rel), None) => Some(parse_relative(&rel)?),
        (None, Some(abs)) => Some(parse_absolute(&abs)?),
        (Some(_), Some(_)) => return Err(anyhow!("Use --in OR --at, not both")),
        (None, None) => None,
    };

    if message.is_none() && scheduled_at.is_none() {
        return Err(anyhow!(
            "Nothing to update. Use --message or --in/--at to change the schedule."
        ));
    }

    ensure_daemon_running().await?;

    let mut client = Client::connect().await?;
    let schedule = client.update_schedule(id, message.clone(), scheduled_at).await?;

    println!(
        "{} Updated schedule {}",
        style("✓").green(),
        style(id).cyan()
    );

    if message.is_some() {
        println!(
            "  {} {}",
            style("Message:").dim(),
            truncate(&schedule.message, 50)
        );
    }

    if scheduled_at.is_some() {
        println!(
            "  {} {} ({})",
            style("Time:").dim(),
            format_absolute(schedule.scheduled_at),
            format_relative(schedule.scheduled_at)
        );
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
